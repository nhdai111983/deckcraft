//! One background check at start-up (and on demand from Help) for a newer release, and the modal
//! prompt that offers to update and restart. The flow itself is `craft_update::updater::Updater`.
//!
//! NexusOffice: Nexus Slide is installed and updated by the NexusOffice installer, never from
//! upstream's DeckCraft releases (`craft_update::suite::DECKCRAFT` downloads `storytold/deckcraft`
//! assets). So the start-up check is never turned on and Help › Check for Updates… only says where
//! updates come from: the app makes no network request of its own.

use craft_update::updater::{Action, Status, Updater};

use crate::SlideApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) struct Runtime {
    updater: Updater,
    /// The desktop host turns the check at start on; tests and the web build leave it off.
    enabled: bool,
    check_requested: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&craft_update::suite::DECKCRAFT, VERSION), enabled: false, check_requested: false }
    }
}

/// NexusOffice: never turns the start-up check on (see the module docs).
pub(crate) fn enable(_app: &mut SlideApp) {}

/// The message Help › Check for Updates… shows instead of checking upstream's releases
/// (English, Vietnamese). The fork has no interface-language switch yet, so the English text is
/// shown; the Vietnamese one is for the Vietnamese interface (nx/vi-dialogs).
pub(crate) const NEXUS_UPDATES: (&str, &str) = (
    "New versions of Nexus Slide arrive through the NexusOffice installer: install the latest NexusOffice to update.",
    "Phiên bản mới của Nexus Slide được cập nhật qua bộ cài NexusOffice: cài NexusOffice mới nhất để cập nhật.",
);

/// Help > Check for Updates…: NexusOffice: says where updates come from; checks nothing.
pub(crate) fn request_check(app: &mut SlideApp) {
    app.notify(NEXUS_UPDATES.0);
}

pub(crate) fn tick(app: &mut SlideApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if std::mem::take(&mut rt.check_requested) {
        rt.updater.check_now(ctx);
    } else if rt.enabled {
        rt.updater.check_at_start(ctx);
    }
    rt.updater.tick(ctx);
    let outcome = match rt.updater.status() {
        Status::UpToDate => Some(format!("{} {VERSION} is up to date.", deckcraft_engine::nexus::APP_NAME)),
        Status::Failed(e) => Some(format!("Could not check for updates: {e}")),
        Status::Idle | Status::Checking => None,
    };
    if let Some(message) = outcome {
        rt.updater.dismiss_status();
        app.notify(message);
    }
}

pub(crate) fn show(app: &mut SlideApp, ctx: &egui::Context) {
    // The restart helper is scheduled before the app closes, so no update starts over unsaved work.
    let clean = !app.session.documents().iter().any(|d| d.is_dirty());
    match app.updates_rt.updater.show_if(ctx, clean) {
        Action::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
        // The app's close path asks about unsaved work; the restart helper relaunches once the process exits.
        Action::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        Action::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NexusOffice: neither the start-up hook nor Help › Check for Updates… starts a check.
    #[test]
    fn nexus_never_checks_upstream_releases() {
        let mut app = SlideApp::new(deckcraft_engine::Session::new(), crate::Services::default());
        enable(&mut app);
        request_check(&mut app);
        assert!(!app.updates_rt.enabled && !app.updates_rt.check_requested);
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |ui| tick(&mut app, ui.ctx())).textures_delta.clear();
        assert!(matches!(app.updates_rt.updater.status(), Status::Idle), "no check was started");
    }

    /// The Check for Updates… message names the NexusOffice installer in both languages.
    #[test]
    fn nexus_update_message_names_the_installer_in_en_and_vi() {
        let (en, vi) = NEXUS_UPDATES;
        assert!(en.contains("NexusOffice installer") && en.contains("Nexus Slide"), "{en}");
        assert!(vi.contains("bộ cài NexusOffice") && vi.contains("Nexus Slide"), "{vi}");
        for text in [en, vi] {
            assert!(!text.contains("DeckCraft") && !text.contains("http"), "{text}");
        }
    }

    #[test]
    fn an_offered_release_opens_the_prompt() {
        let mut app = SlideApp::new(deckcraft_engine::Session::new(), crate::Services::default());
        let ctx = egui::Context::default();
        assert!(!app.updates_rt.updater.prompt_open());
        app.updates_rt.updater.offer(craft_update::Release { tag: "v99.0.0".into(), ..Default::default() });
        ctx.run_ui(egui::RawInput::default(), |ui| show(&mut app, ui.ctx())).textures_delta.clear();
        assert!(app.updates_rt.updater.prompt_open());
    }
}
