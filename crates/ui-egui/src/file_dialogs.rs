//! NexusOffice: Open and Save As dialogs a host shows without freezing its window.
//!
//! A host with [`Services::file_dialog`](crate::Services::file_dialog) (the NexusOffice window,
//! which shows the system dialog off the UI thread) answers later through a channel;
//! [`SlideApp::logic`](crate::SlideApp::logic) picks the answer up on a later frame and finishes
//! what the dialog was for (open the presentation, save it and maybe close it, export a copy,
//! insert a picture). A host with only the blocking `pick_open` / `pick_save` (the stand-alone
//! app) keeps upstream's flow. One dialog at a time: asking for another while one is open does
//! nothing.

use std::sync::mpsc::{Receiver, TryRecvError};

use serde_json::{Value, json};

use crate::SlideApp;

/// A file dialog the app asks the host to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogRequest {
    /// Pick a file to open; `purpose` says what for (`open`, `picture`, `audio`, `video`, `theme`).
    Open { purpose: String },
    /// Pick a path to save to, given a suggested file name and the types to offer (extensions,
    /// the first the default): `pptx` and `deckcraft` to save the presentation, `pdf`, `png`,
    /// `jpg`, `txt` or `gif` for a copy.
    Save { name: String, types: Vec<String> },
}

/// The host's answer: the picked path, `Ok(None)` when the user cancelled, or an error when no
/// system dialog could be shown.
pub type FileDialogAnswer = Result<Option<String>, String>;

/// The types a presentation is saved as (Save, Save As): its file stays one Nexus Slide opens.
pub const PRESENTATION_TYPES: [&str; 2] = ["pptx", "deckcraft"];

/// The file types Open offers for `purpose`, as (label, extensions).
pub fn open_filters(purpose: &str) -> Vec<(&'static str, &'static [&'static str])> {
    match purpose {
        "picture" => vec![("Pictures", &["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "svg"])],
        "audio" => vec![("Audio", &["wav", "mp3", "m4a", "aac", "flac", "ogg", "oga", "opus", "aif", "aiff", "wma", "weba", "mka"])],
        "video" => vec![("Video", &["mp4", "m4v", "mov", "webm", "mkv", "wmv"])],
        "theme" => vec![("Themes and presentations", &["thmx", "pptx", "potx", "ppsx"])],
        _ => vec![
            ("Presentations", &["pptx", "deckcraft", "potx", "ppsx"]),
            ("PowerPoint", &["pptx", "potx", "ppsx"]),
            ("Nexus Slide", &["deckcraft"]),
            ("Outline", &["txt", "md"]),
        ],
    }
}

/// The path a presentation is saved to: `.pptx` added when the name has no presentation type
/// ("Báo cáo 10.2026" has the extension "2026", and a PDF picked in the Save dialog would leave the
/// presentation unsaved).
pub(crate) fn with_presentation_extension(path: String) -> String {
    let ext = std::path::Path::new(&path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    if ext.is_some_and(|e| PRESENTATION_TYPES.contains(&e.as_str())) { path } else { format!("{path}.pptx") }
}

/// What to do with the path picked in a dialog.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AfterPick {
    /// Open the picked presentation (or outline).
    Open,
    /// Save the active presentation there; then close it when "Save changes?" asked for the save.
    Save { then_close: bool },
    /// Write a copy with `file.export` in `format` (`all`: every slide as a picture), with `extra`
    /// params (a PDF's layout…).
    Export { format: String, ext: String, all: bool, extra: Value },
    /// Run `cmd` with the picked file as `path`, added to `extra` (insert a picture, a sound…).
    Run { cmd: String, extra: Value },
    /// Insert every slide of the picked presentation after the current one.
    Reuse { keep: bool },
}

/// A dialog the host is showing; the answer arrives on `rx`.
pub(crate) struct PendingDialog {
    rx: Receiver<FileDialogAnswer>,
    pub(crate) after: AfterPick,
}

impl SlideApp {
    /// Whether a host file dialog is open (its answer hasn't arrived yet).
    pub fn file_dialog_open(&self) -> bool {
        self.file_dialog.is_some()
    }

    /// Whether this window reaches files on disk: a desktop picker, or a host's dialogs (the
    /// browser build downloads instead).
    pub fn has_file_system(&self) -> bool {
        self.services.pick_save.is_some() || self.services.file_dialog.is_some()
    }

    /// Whether this window can pick a file to open (a desktop picker or a host's dialogs).
    pub fn can_pick_files(&self) -> bool {
        self.services.pick_open.is_some() || self.services.file_dialog.is_some()
    }

    /// Ask the host's non-blocking dialog and do `after` with the answer on a later frame. `false`
    /// when the host has none (the caller uses the blocking picker). Asking while another dialog
    /// is open does nothing (and counts as asked).
    pub(crate) fn ask_file(&mut self, req: FileDialogRequest, after: AfterPick) -> bool {
        if self.file_dialog.is_some() {
            return true;
        }
        let Some(ask) = &self.services.file_dialog else { return false };
        let rx = ask(req);
        self.file_dialog = Some(PendingDialog { rx, after });
        true
    }

    /// "Save changes?" answered Save on a presentation with no file: close it once Save As has
    /// saved it.
    pub(crate) fn close_after_pending_save(&mut self) {
        if let Some(p) = &mut self.file_dialog
            && matches!(p.after, AfterPick::Save { .. })
        {
            p.after = AfterPick::Save { then_close: true };
        }
    }

    /// Picks up the host's answer, if it has come. Called every frame from `logic`.
    pub(crate) fn poll_file_dialog(&mut self) {
        let answer = match &self.file_dialog {
            None => return,
            Some(p) => match p.rx.try_recv() {
                Err(TryRecvError::Empty) => return,
                // The host went away without answering: as cancelled.
                Err(TryRecvError::Disconnected) => Ok(None),
                Ok(answer) => answer,
            },
        };
        let Some(pending) = self.file_dialog.take() else { return };
        match answer {
            Ok(Some(path)) if !path.is_empty() => self.finish_pick(path, pending.after),
            Ok(_) => {}
            Err(why) => {
                let lead = self.ui.language.tr("No file dialog could be shown");
                self.notify(format!("{lead}: {why}"));
            }
        }
    }

    fn finish_pick(&mut self, path: String, after: AfterPick) {
        match after {
            AfterPick::Open => self.open_path_in_background(&path),
            AfterPick::Save { then_close } => {
                let path = with_presentation_extension(path);
                match self.session.execute("file.saveAs", &json!({"path": path})) {
                    Ok(_) => {
                        self.note_saved(&path);
                        if then_close && self.session.active().is_some_and(|d| !d.is_dirty()) {
                            let _ = self.run("file.close", json!({}));
                        }
                    }
                    Err(e) => self.notify(e.to_string()),
                }
            }
            AfterPick::Export { format, ext, all, extra } => {
                // The dialog may keep another type's extension; the copy is written as `ext`.
                let path = std::path::Path::new(&path).with_extension(&ext).to_string_lossy().into_owned();
                let mut params = json!({"path": path, "format": format, "all": all});
                if let (Some(o), Some(x)) = (params.as_object_mut(), extra.as_object()) {
                    o.extend(x.clone());
                }
                if self.run("file.export", params).is_ok() {
                    self.set_status(format!("Exported {path}"));
                }
            }
            AfterPick::Run { cmd, mut extra } => {
                if !extra.is_object() {
                    extra = json!({});
                }
                extra["path"] = json!(path);
                if let Err(e) = self.session.execute(&cmd, &extra) {
                    self.set_status(e.to_string());
                }
            }
            AfterPick::Reuse { keep } => match self.session.execute("slide.reuse", &json!({"path": path, "keepFormatting": keep})) {
                Ok(r) => self.set_status(format!("Inserted {} slides from {path}", r["slides"])),
                Err(e) => self.set_status(e.to_string()),
            },
        }
        self.drain_requests();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_presentation_save_always_names_a_presentation_file() {
        let w = |p: &str| with_presentation_extension(p.to_string());
        assert_eq!(w("Báo cáo 10.2026"), "Báo cáo 10.2026.pptx", "a dot in the name is not a type");
        assert_eq!(w("Họp giao ban"), "Họp giao ban.pptx");
        assert_eq!(w("Họp giao ban.PPTX"), "Họp giao ban.PPTX");
        assert_eq!(w("Họp giao ban.deckcraft"), "Họp giao ban.deckcraft");
        assert_eq!(w("Họp giao ban.pdf"), "Họp giao ban.pdf.pptx", "Save keeps a presentation, not a copy");
    }

    #[test]
    fn open_offers_presentations_first() {
        assert!(open_filters("open").first().is_some_and(|(_, e)| e.contains(&"pptx") && e.contains(&"deckcraft")));
        assert!(open_filters("picture").first().is_some_and(|(_, e)| e.contains(&"png")));
    }
}
