//! Crash recovery (PowerPoint's AutoRecover): unsaved changes of every open presentation are
//! written to a recovery folder (`file.recovery.save`, called on a timer by the app and after a
//! command panics); saving or closing a presentation removes its entry; after a crash
//! `file.recovery.open` reopens what was left there as unsaved presentations that remember where
//! they were saved.
//!
//! An entry is `<uid>.deckcraft` (the presentation) plus `<uid>.json` (`{"path", "title",
//! "saved", "owner"}`: the original file, the title, when it was written (Unix seconds) and the
//! instance that owns it). The metadata is written first, so an entry is never on disk without
//! its owner.
//!
//! Several instances share the folder. Document uids are unique across processes, each instance
//! holds a lock on `owner-<instance>.lock` while it runs, and only the entries of no running
//! instance (this one's are other sessions' live entries) are offered, reopened or discarded as a
//! crash's. Deciding that and taking an entry over happen under the folder's lock
//! (`recovery.lock`, [`take`], [`discard_left`]), so two instances answering at once never both
//! take one. Opening an entry doesn't delete it: the recovered presentation keeps the entry's uid,
//! so the snapshot stays on disk until the next recovery save replaces it or a save of the
//! presentation removes it.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};

use crate::{DocState, EngineError, Result, Session};

/// The platform's per-user recovery folder.
pub fn default_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        return home.map(|h| h.join("Library/Application Support").join(crate::nexus::SETTINGS_DIR).join("Recovery"));
    }
    if cfg!(target_os = "windows") {
        return std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join(crate::nexus::SETTINGS_DIR).join("Recovery"));
    }
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".local/share")))
        .map(|d| d.join(crate::nexus::SETTINGS_DIR_XDG).join("recovery"))
}

/// Uids and instance ids stay below 2^53 so they survive JSON numbers read as `f64`.
pub(crate) const UID_MASK: u64 = (1 << 53) - 1;

/// This process's random id, with the low 24 bits clear so its document uids (id + counter) don't
/// run into another instance's.
pub fn instance_id() -> u64 {
    static ID: OnceLock<u64> = OnceLock::new();
    *ID.get_or_init(|| {
        use std::hash::{BuildHasher, Hasher};
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u32(std::process::id());
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(d) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            h.write_u128(d.as_nanos());
        }
        // Never 0: old entries are numbered from 1.
        ((h.finish() & UID_MASK) & !0xFF_FFFF).max(1 << 24)
    })
}

fn lock_path(dir: &Path, owner: u64) -> PathBuf {
    dir.join(format!("owner-{owner}.lock"))
}

/// The folder's own lock, held while an instance decides which entries a crash left and takes
/// or discards them.
const FOLDER_LOCK: &str = "recovery.lock";

/// How often a busy lock is tried before giving up (another instance probing a lock file holds
/// it for an instant), with [`pause`] between tries.
const LOCK_TRIES: u32 = 20;

fn pause() {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::sleep(std::time::Duration::from_millis(25));
}

fn open_lock_file(path: &Path) -> std::io::Result<File> {
    std::fs::OpenOptions::new().create(true).write(true).truncate(false).open(path)
}

/// Hold this instance's lock in `dir` for the rest of the process. Checked again on every call:
/// a lock file another instance removed (it found the file unlocked an instant before this one
/// locked it) is made and locked again, so this instance's entries are never taken for a crash's
/// while it runs. Returns whether the lock is held (a failure is logged).
fn claim_dir(dir: &Path) -> bool {
    static HELD: Mutex<Vec<(PathBuf, File)>> = Mutex::new(Vec::new());
    let Ok(mut held) = HELD.lock() else { return false };
    let me = instance_id();
    if let Some(i) = held.iter().position(|(d, _)| d == dir) {
        if owner_alive(dir, me) {
            return true;
        }
        // Its file is gone: let go of the old handle first (Windows can't make a file whose
        // deletion is still pending) and lock a new one.
        held.swap_remove(i);
    }
    let path = lock_path(dir, me);
    let mut why = String::from("busy");
    let mut open_failures = 0;
    for _ in 0..LOCK_TRIES {
        match open_lock_file(&path) {
            Ok(f) => match f.try_lock() {
                // Locked, and still the file at that name (not one removed in between).
                Ok(()) if owner_alive(dir, me) => {
                    held.push((dir.to_path_buf(), f));
                    return true;
                }
                Ok(()) => why = "its file was removed while it was locked".into(),
                Err(std::fs::TryLockError::WouldBlock) => why = "busy".into(),
                Err(std::fs::TryLockError::Error(e)) => {
                    why = e.to_string();
                    break;
                }
            },
            Err(e) => {
                why = e.to_string();
                open_failures += 1;
                if open_failures > 3 {
                    break;
                }
            }
        }
        if cfg!(target_arch = "wasm32") {
            break;
        }
        pause();
    }
    log::warn!("recovery: could not lock {}: {why}", path.display());
    false
}

/// Whether a running instance (this one included) holds `owner`'s lock in `dir`.
fn owner_alive(dir: &Path, owner: u64) -> bool {
    let Ok(f) = File::open(lock_path(dir, owner)) else { return false };
    matches!(f.try_lock(), Err(std::fs::TryLockError::WouldBlock))
}

/// Runs `f` holding the folder's lock ([`FOLDER_LOCK`]), so two instances deciding at once which
/// entries a crash left never both take (or one discard) an entry. Without it (a file system
/// without locks, a busy folder) `f` still runs, and that is logged.
fn exclusively<R>(dir: &Path, f: impl FnOnce() -> R) -> R {
    let lock = open_lock_file(&dir.join(FOLDER_LOCK)).ok();
    let mut locked = false;
    if let Some(l) = &lock {
        for _ in 0..LOCK_TRIES {
            match l.try_lock() {
                Ok(()) => {
                    locked = true;
                    break;
                }
                Err(std::fs::TryLockError::WouldBlock) if !cfg!(target_arch = "wasm32") => pause(),
                Err(_) => break,
            }
        }
    }
    if !locked {
        log::warn!("recovery: could not lock {}", dir.join(FOLDER_LOCK).display());
    }
    let r = f();
    // Dropping the file lets go of the lock.
    drop(lock);
    r
}

/// Write `bytes` to `path` through a temporary file and a rename, so a crash never leaves it torn.
/// The temporary file reaches the disk before the rename (and, on Unix, the rename before this
/// returns): after a power cut the entry is the old one or the new one, never an empty file the
/// rename got to the disk ahead of. A failed write leaves no temporary file behind.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let tmp = path.with_extension(format!("{}.tmp", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
    let written = File::create(&tmp)
        .and_then(|mut f| {
            f.write_all(bytes)?;
            f.sync_all()
        })
        .and_then(|()| std::fs::rename(&tmp, path));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(EngineError::Other(format!("{}: {e}", path.display())));
    }
    // The folder's entry for the rename (Unix; Windows can't open a folder as a file).
    #[cfg(unix)]
    if let Some(dir) = path.parent()
        && let Ok(d) = File::open(dir)
    {
        let _ = d.sync_all();
    }
    Ok(())
}

/// Presentations reopened per start; the rest stay on disk for the next one, so a folder
/// full of entries can't stall start-up.
pub const MAX_REOPEN: usize = 32;
/// The largest entry written or read back (512 MiB). A presentation bigger than this gets no entry
/// (it couldn't be read back), and an entry bigger than this is never read: recovering it could
/// run out of memory at every start.
pub const MAX_ENTRY_BYTES: u64 = 512 << 20;
const MAX_META_BYTES: u64 = 64 << 10;
/// A temporary file older than this is a write a crash cut short.
#[cfg(not(target_arch = "wasm32"))]
const STALE_TMP: std::time::Duration = std::time::Duration::from_secs(60 * 60);
/// An entry a crash left that nobody recovered for this long is removed.
#[cfg(not(target_arch = "wasm32"))]
const STALE_ENTRY: std::time::Duration = std::time::Duration::from_secs(30 * 24 * 60 * 60);

/// Why an entry wasn't read back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryError {
    /// Bigger than [`MAX_ENTRY_BYTES`] (its size in bytes): left on disk, never read.
    TooLarge(u64),
    /// Missing, not a regular file, or unreadable (memory for it included).
    Unreadable,
}

/// `p` as bytes when it's a regular file no bigger than `max`: reading a FIFO left where an
/// entry belongs would block start-up forever. The memory is asked for, so a copy too big for
/// what is free is an error rather than an abort.
fn read_regular(p: &Path, max: u64) -> std::result::Result<Vec<u8>, EntryError> {
    use std::io::Read;
    let m = std::fs::metadata(p).map_err(|_| EntryError::Unreadable)?;
    if !m.is_file() {
        return Err(EntryError::Unreadable);
    }
    if m.len() > max {
        return Err(EntryError::TooLarge(m.len()));
    }
    let f = File::open(p).map_err(|_| EntryError::Unreadable)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(usize::try_from(m.len()).map_err(|_| EntryError::Unreadable)?).map_err(|_| EntryError::Unreadable)?;
    f.take(max.saturating_add(1)).read_to_end(&mut bytes).map_err(|_| EntryError::Unreadable)?;
    match u64::try_from(bytes.len()) {
        Ok(n) if n <= max => Ok(bytes),
        Ok(n) => Err(EntryError::TooLarge(n)),
        Err(_) => Err(EntryError::Unreadable),
    }
}

fn files(dir: &Path, uid: u64) -> (PathBuf, PathBuf) {
    (dir.join(format!("{uid}.deckcraft")), dir.join(format!("{uid}.json")))
}

/// Entry `uid`'s metadata as it is on disk now (`{}`: none, or not an object).
fn read_meta(dir: &Path, uid: u64) -> Value {
    // Metadata is a file on disk: anything but an object (corrupt, hand-edited) counts as none.
    read_regular(&files(dir, uid).1, MAX_META_BYTES)
        .ok()
        .and_then(|t| serde_json::from_slice::<Value>(&t).ok())
        .filter(Value::is_object)
        .unwrap_or(json!({}))
}

/// Whether metadata `meta` belongs to a crash: it names no instance, or one that is neither this
/// one nor running.
fn crash_left(dir: &Path, meta: &Value) -> bool {
    match meta.get("owner").and_then(Value::as_u64) {
        Some(owner) => owner != instance_id() && !owner_alive(dir, owner),
        None => true,
    }
}

fn now_secs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

/// Remove a presentation's recovery entry (it was saved or closed).
pub fn discard(dir: &Path, uid: u64) {
    let (d, m) = files(dir, uid);
    let _ = std::fs::remove_file(d);
    let _ = std::fs::remove_file(m);
}

/// Remove the entries of the presentations open in `s` (Don't Save on quit). Nothing else: not
/// what a crash left (offered at the next start), not another window's live entries, and not those
/// of this process's other sessions (a host with one session per tab: the other tabs' live entries).
pub fn discard_session(s: &Session, dir: &Path) {
    for d in s.documents() {
        discard(dir, d.uid);
    }
}

/// Holds this instance's lock in `dir` (made if needed), as every write and take does first.
/// `false`: it can't be held (a file system without locks, no access), so no entry is written or
/// taken: crash recovery is off in that folder. Hosts tell the user.
pub fn hold_lock(dir: &Path) -> bool {
    std::fs::create_dir_all(dir).is_ok() && claim_dir(dir)
}

fn no_lock(dir: &Path) -> EngineError {
    EngineError::Other(format!("{}: the recovery folder can't be locked; AutoRecover is off", dir.display()))
}

/// What [`save_entries`] wrote.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saved {
    /// Entries written.
    pub written: usize,
    /// Changed presentations bigger than [`MAX_ENTRY_BYTES`]: no entry was written for them (an
    /// older, smaller one stays).
    pub too_large: Vec<u64>,
}

/// Write entries for the unsaved presentations (and drop those of saved ones). Returns how many
/// were written.
pub fn save(s: &Session, dir: &Path) -> Result<usize> {
    save_entries(s, dir).map(|r| r.written)
}

/// [`save`], saying which presentations were too large to get an entry.
pub fn save_entries(s: &Session, dir: &Path) -> Result<Saved> {
    save_capped(s, dir, MAX_ENTRY_BYTES)
}

fn save_capped(s: &Session, dir: &Path, max: u64) -> Result<Saved> {
    std::fs::create_dir_all(dir).map_err(|e| EngineError::Other(format!("{}: {e}", dir.display())))?;
    // Without this instance's lock its entries would look like a crash's to every other
    // instance (offered, taken, discarded while it runs): none is written.
    if !claim_dir(dir) {
        return Err(no_lock(dir));
    }
    let mut out = Saved::default();
    for d in s.documents() {
        if !d.is_dirty() {
            discard(dir, d.uid);
            continue;
        }
        let (doc, meta) = files(dir, d.uid);
        let bytes = deckcraft_format::save(&d.doc).map_err(|e| EngineError::Other(e.to_string()))?;
        if !u64::try_from(bytes.len()).is_ok_and(|n| n <= max) {
            // Never written as if it could be recovered: it couldn't be read back.
            log::warn!("recovery: {} is too large for an AutoRecover entry ({} bytes)", d.title(), bytes.len());
            out.too_large.push(d.uid);
            // An older entry of it stays, marked stale: offered as an older version, with its time.
            if std::fs::metadata(&doc).is_ok_and(|m| m.is_file()) {
                let mut m = read_meta(dir, d.uid);
                if let Some(o) = m.as_object_mut()
                    && o.get("stale") != Some(&json!(true))
                {
                    o.insert("stale".into(), json!(true));
                    if let Err(e) = write_atomic(&meta, m.to_string().as_bytes()) {
                        log::warn!("recovery: {e}");
                    }
                }
            }
            continue;
        }
        // The metadata first: an entry is never on disk without the instance that owns it, even
        // for an instant (another instance would take it for a crash's).
        let m = json!({"path": d.path, "title": d.title(), "saved": now_secs(), "owner": instance_id()});
        write_atomic(&meta, m.to_string().as_bytes())?;
        write_atomic(&doc, &bytes)?;
        out.written += 1;
    }
    Ok(out)
}

/// Entries in `dir`: (uid, metadata).
pub fn list(dir: &Path) -> Vec<(u64, Value)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("deckcraft") {
            continue;
        }
        if !std::fs::metadata(&p).is_ok_and(|m| m.is_file()) {
            continue;
        }
        let Some(uid) = p.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse::<u64>().ok()) else { continue };
        out.push((uid, read_meta(dir, uid)));
    }
    out.sort_by_key(|e| e.0);
    out
}

/// The entries a crash left in `dir`: those of no running instance, this one included (a host
/// with several sessions offers them once, before any of its own are written, and gives each to
/// a session of its own with [`take`], [`read_entry`] and [`reopen`]). Same shape as [`list`].
/// Old leftovers are swept first ([`sweep`]) and lock files of instances no longer running are
/// removed, as [`open`] does.
///
/// Each entry returned is recorded as offered (`"offered"`: when it first was): only entries that
/// were offered are ever swept, so one nobody was shown (no Slide tab opened for a month, a clock
/// that jumped) is never lost. An entry marked `"stale": true` is an older version (the newer one
/// was too large to write).
pub fn recoverable(dir: &Path) -> Vec<(u64, Value)> {
    sweep(dir);
    let mut out: Vec<(u64, Value)> = list(dir).into_iter().filter(|(_, meta)| crash_left(dir, meta)).collect();
    for (uid, meta) in &mut out {
        if meta.get("offered").and_then(Value::as_u64).is_some() {
            continue;
        }
        let when = now_secs();
        exclusively(dir, || {
            let mut m = read_meta(dir, *uid);
            if !crash_left(dir, &m) || m.get("offered").and_then(Value::as_u64).is_some() {
                return;
            }
            if let Some(o) = m.as_object_mut() {
                o.insert("offered".into(), json!(when));
            }
            if let Err(e) = write_atomic(&files(dir, *uid).1, m.to_string().as_bytes()) {
                log::warn!("recovery: {e}");
            }
        });
        if let Some(o) = meta.as_object_mut() {
            o.insert("offered".into(), json!(when));
        }
    }
    remove_stale_locks(dir);
    out
}

/// Takes entry `uid` over for this instance if a crash left it, checked now under the folder's
/// lock (an offer made earlier may be stale: another window may have recovered it since). Its
/// metadata then names this instance, so no other instance offers, recovers or discards it while
/// this one runs. `Ok(false)`: gone, or this or another running instance has it. The snapshot is
/// left as it is.
pub fn take(dir: &Path, uid: u64) -> Result<bool> {
    exclusively(dir, || {
        let (doc, meta_path) = files(dir, uid);
        let mut meta = read_meta(dir, uid);
        if !std::fs::metadata(&doc).is_ok_and(|m| m.is_file()) || !crash_left(dir, &meta) {
            return Ok(false);
        }
        // Owned by this instance only while it holds its lock; otherwise others would take it too.
        if !claim_dir(dir) {
            return Err(no_lock(dir));
        }
        if let Some(o) = meta.as_object_mut() {
            o.insert("owner".into(), json!(instance_id()));
        }
        write_atomic(&meta_path, meta.to_string().as_bytes())?;
        Ok(true)
    })
}

/// Removes entry `uid` if a crash left it, checked now under the folder's lock: an entry this or
/// another running instance has (recovered since it was offered) stays. Returns whether it went.
pub fn discard_left(dir: &Path, uid: u64) -> bool {
    exclusively(dir, || {
        if !crash_left(dir, &read_meta(dir, uid)) {
            return false;
        }
        discard(dir, uid);
        true
    })
}

/// The presentation of entry `uid` in `dir`, read back (an error: missing, not a regular file or
/// too large). The entry stays on disk.
pub fn read_entry(dir: &Path, uid: u64) -> std::result::Result<Vec<u8>, EntryError> {
    let (doc, _) = files(dir, uid);
    read_regular(&doc, MAX_ENTRY_BYTES)
}

/// Add the presentation in `bytes` (an entry's, from [`read_entry`]) to `s` as an unsaved
/// presentation that keeps the entry's `uid` (so the next recovery save replaces the entry and
/// saving or closing it removes it), remembers `path` and is called `title`. Nothing is written
/// or deleted. Returns its index; an error for bytes that aren't a presentation, or a `uid`
/// already open in `s`.
pub fn reopen(s: &mut Session, uid: u64, bytes: &[u8], path: Option<String>, title: &str) -> Result<usize> {
    if s.documents().iter().any(|d| d.uid == uid) {
        return Err(EngineError::Other(format!("recovered presentation {uid} is already open")));
    }
    let mut d = deckcraft_format::load(bytes).map_err(|e| EngineError::Other(e.to_string()))?;
    deckcraft_format::repair(&mut d);
    let mut st = DocState::new(d, path, title.to_string());
    st.uid = uid;
    // Unsaved: the copy on disk (if any) is older than what was recovered.
    st.saved_doc = std::sync::Arc::new((*st.doc).clone());
    st.revision += 1;
    Ok(s.add_document(st))
}

/// Reopen every entry a crash left (no running instance's, this one's included: in a host with
/// one session per tab those are its other sessions' live entries) as an unsaved presentation.
/// Each is taken over first ([`take`]), so another instance starting now leaves it alone; it stays
/// on disk, and the presentation keeps the entry's uid, so the next recovery save replaces the
/// snapshot and saving or closing it removes it. Unreadable entries and entries already open here
/// are left alone. Returns the opened documents' indices. At most [`MAX_REOPEN`] are opened per
/// start, the newest first; the rest wait for the next one.
pub fn open(s: &mut Session, dir: &Path) -> Result<Vec<usize>> {
    sweep(dir);
    let mut opened = Vec::new();
    let mut entries = list(dir);
    entries.sort_by_key(|(uid, meta)| std::cmp::Reverse((meta.get("saved").and_then(Value::as_u64).unwrap_or(0), *uid)));
    for (uid, meta) in entries {
        if opened.len() >= MAX_REOPEN {
            break;
        }
        if s.documents().iter().any(|d| d.uid == uid) || !crash_left(dir, &meta) {
            continue;
        }
        match take(dir, uid) {
            Ok(true) => {}
            Ok(false) => continue,
            // Its metadata couldn't be rewritten: the presentation is still worth reopening.
            Err(e) => log::warn!("recovery: {e}"),
        }
        let Ok(bytes) = read_entry(dir, uid) else { continue };
        let path = meta.get("path").and_then(Value::as_str).map(str::to_string);
        let title = meta.get("title").and_then(Value::as_str).filter(|t| !t.is_empty()).unwrap_or("Presentation").to_string();
        let Ok(i) = reopen(s, uid, &bytes, path, &format!("{title} (Recovered)")) else { continue };
        opened.push(i);
    }
    remove_stale_locks(dir);
    Ok(opened)
}

/// Delete the lock files of instances that are no longer running. Each is removed while this
/// instance holds its lock, so an instance that locks it meanwhile finds its file gone and makes
/// a new one ([`claim_dir`]) instead of holding a lock nobody can see.
fn remove_stale_locks(dir: &Path) {
    let me = instance_id();
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(owner) =
            name.to_str().and_then(|n| n.strip_prefix("owner-")).and_then(|n| n.strip_suffix(".lock")).and_then(|n| n.parse::<u64>().ok())
        else {
            continue;
        };
        if owner == me {
            continue;
        }
        let Ok(f) = File::open(e.path()) else { continue };
        if f.try_lock().is_ok() {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// Remove what nobody will recover: temporary files older than an hour (writes a crash cut
/// short), and entries a crash left that were offered 30 days ago or more and not taken since
/// (or metadata whose presentation is gone). An entry never offered stays however old (nobody
/// was shown it yet), and so does a running instance's (this one's included).
pub fn sweep(dir: &Path) {
    #[cfg(not(target_arch = "wasm32"))]
    sweep_at(dir, std::time::SystemTime::now());
    #[cfg(target_arch = "wasm32")]
    let _ = dir;
}

#[cfg(not(target_arch = "wasm32"))]
fn sweep_at(dir: &Path, now: std::time::SystemTime) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut old = std::collections::BTreeSet::new();
    let mut recent = std::collections::HashSet::new();
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        // Not followed through a link: only the folder's own files.
        let Ok(m) = e.metadata() else { continue };
        if !m.is_file() {
            continue;
        }
        let age = m.modified().ok().and_then(|t| now.duration_since(t).ok());
        if name.ends_with(".tmp") {
            if age.is_some_and(|a| a > STALE_TMP) {
                let _ = std::fs::remove_file(e.path());
            }
            continue;
        }
        let Some(uid) = name.strip_suffix(".deckcraft").or_else(|| name.strip_suffix(".json")).and_then(|n| n.parse::<u64>().ok()) else {
            continue;
        };
        if age.is_some_and(|a| a > STALE_ENTRY) {
            old.insert(uid);
        } else {
            recent.insert(uid);
        }
    }
    let now_s = now.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    for uid in old {
        if recent.contains(&uid) {
            continue;
        }
        let meta = read_meta(dir, uid);
        let has_doc = std::fs::metadata(files(dir, uid).0).is_ok_and(|m| m.is_file());
        let offered_long_ago = meta.get("offered").and_then(Value::as_u64).is_some_and(|t| now_s.saturating_sub(t) > STALE_ENTRY.as_secs());
        if crash_left(dir, &meta) && (!has_doc || offered_long_ago) {
            discard(dir, uid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deckcraft-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// What a crash leaves: every entry in `dir` made an instance's that no longer runs.
    fn crash(dir: &Path) {
        for (uid, mut meta) in list(dir) {
            if let Some(o) = meta.as_object_mut() {
                o.insert("owner".into(), json!(instance_id() ^ (1 << 45)));
            }
            std::fs::write(files(dir, uid).1, meta.to_string()).unwrap();
        }
    }

    #[test]
    fn recovers_unsaved_presentations() {
        let dir = temp("recovery");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.new", &json!({})).unwrap();
        // Only the edited presentation is written.
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        assert_eq!(save(&s, &dir).unwrap(), 1);
        assert_eq!(list(&dir).len(), 1);
        // "Crash": a new session reopens it, unsaved, with its slides.
        crash(&dir);
        let mut s2 = Session::new();
        assert_eq!(open(&mut s2, &dir).unwrap().len(), 1);
        let st = s2.doc().unwrap();
        assert!(st.is_dirty());
        assert_eq!(st.doc.slides.len(), 2);
        assert!(st.title().ends_with("(Recovered)"));
        assert_eq!(list(&dir).len(), 1, "the snapshot stays until it is replaced or saved");
        // A second "crash" before any checkpoint: the presentation is recovered again.
        crash(&dir);
        let mut s3 = Session::new();
        assert_eq!(open(&mut s3, &dir).unwrap().len(), 1);
        assert_eq!(s3.doc().unwrap().doc.slides.len(), 2);
        // Opening again in the same session doesn't duplicate it.
        assert!(open(&mut s3, &dir).unwrap().is_empty());
        // Saving the recovered presentation drops the entry.
        s3.recovery_dir = Some(dir.clone());
        let path = dir.join("saved.deckcraft");
        s3.execute("file.saveAs", &json!({"path": path.to_string_lossy()})).unwrap();
        assert!(list(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A host with several sessions (NexusOffice: one per tab) offers what a crash left, one
    /// entry per session: `recoverable` leaves out this instance's own entries and those of a
    /// running instance, and `reopen` gives an entry to a session as an unsaved presentation that
    /// keeps its uid, file and title (so saving it removes the entry).
    #[test]
    fn hosts_offer_and_reopen_one_entry_at_a_time() {
        let dir = temp("recovery-hooks");
        std::fs::create_dir_all(&dir).unwrap();
        // This instance's own entry: written by a session here, never offered here.
        let mut mine = Session::new();
        mine.execute("file.new", &json!({})).unwrap();
        mine.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        assert_eq!(save(&mine, &dir).unwrap(), 1);
        let mine_uid = mine.doc().unwrap().uid;
        // A running instance's entry (its lock held): not offered either.
        let live = instance_id() ^ (1 << 41);
        let lock = File::create(lock_path(&dir, live)).unwrap();
        lock.try_lock().unwrap();
        let bytes = deckcraft_format::save(&mine.doc().unwrap().doc).unwrap();
        let (d, m) = files(&dir, live + 1);
        std::fs::write(&d, &bytes).unwrap();
        std::fs::write(&m, json!({"title": "Live", "owner": live}).to_string()).unwrap();
        // A crashed instance's entry (no lock held), with its file: offered.
        let dead = instance_id() ^ (1 << 42);
        File::create(lock_path(&dir, dead)).unwrap();
        let (d, m) = files(&dir, dead + 1);
        std::fs::write(&d, &bytes).unwrap();
        std::fs::write(&m, json!({"title": "Báo cáo", "path": "/x/báo cáo.pptx", "owner": dead}).to_string()).unwrap();
        let offered = recoverable(&dir);
        assert_eq!(offered.iter().map(|e| e.0).collect::<Vec<_>>(), vec![dead + 1], "only the crash's entry");
        assert!(offered.iter().all(|e| e.0 != mine_uid));
        assert!(!lock_path(&dir, dead).exists(), "a crashed instance's lock file is cleaned up");
        assert!(lock_path(&dir, live).exists(), "a running one's stays");
        // Reopened in a session of its own: unsaved, its uid, file and title kept, nothing deleted.
        let mut s = Session::new();
        assert_eq!(take(&dir, dead + 1).ok(), Some(true));
        let read = read_entry(&dir, dead + 1).unwrap();
        let i = reopen(&mut s, dead + 1, &read, Some("/x/báo cáo.pptx".into()), "Báo cáo").unwrap();
        let st = &s.documents()[i];
        assert!(st.is_dirty() && st.uid == dead + 1);
        assert_eq!((st.path.as_deref(), st.title.as_str()), (Some("/x/báo cáo.pptx"), "Báo cáo"));
        assert!(files(&dir, dead + 1).0.exists(), "the entry stays until a newer save or a save of the presentation");
        // The same entry twice, or bytes that aren't a presentation: an error, nothing added.
        assert!(reopen(&mut s, dead + 1, &read, None, "x").is_err());
        assert!(reopen(&mut s, 77, b"not a presentation", None, "x").is_err());
        assert_eq!(s.documents().len(), 1);
        // The next save writes it as this instance's: no longer offered.
        save(&s, &dir).unwrap();
        assert!(recoverable(&dir).is_empty());
        drop(lock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Entries are written whole through a temporary file and a rename, and a write that fails
    /// leaves no temporary file behind. (That the temporary file is synced before the rename can't
    /// be seen from a test: only a power cut would show it.)
    #[test]
    fn entries_are_written_atomically() {
        let dir = temp("recovery-atomic");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("9.deckcraft");
        write_atomic(&p, b"first").unwrap();
        write_atomic(&p, b"second").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"second");
        // The rename fails (a folder that isn't empty is where the entry goes): an error, and the
        // temporary file written before it is removed.
        let blocked = dir.join("8.deckcraft");
        std::fs::create_dir_all(blocked.join("inside")).unwrap();
        assert!(write_atomic(&blocked, b"x").is_err());
        assert!(!dir.join("8.deckcraft.tmp").exists(), "the temporary file of a failed write is removed");
        let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
        names.sort();
        assert_eq!(names, ["8.deckcraft", "9.deckcraft"], "nothing else left");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Junk in the folder (names that aren't uids, metadata that isn't an object, an owner that
    /// isn't a number, an empty or garbage presentation) is never offered as something it isn't
    /// and never panics.
    #[test]
    fn hostile_entries_never_panic() {
        let dir = temp("recovery-junk");
        std::fs::create_dir_all(&dir).unwrap();
        let junk: [(&str, &[u8]); 7] = [
            ("1.deckcraft.tmp", b"x"),
            ("2.deckcraft", b""),
            ("2.json", b"{\"owner\": \"me\", \"title\": 7, \"path\": [1]}"),
            ("3.deckcraft", b"\xff\xfe garbage"),
            ("3.json", b"{\"owner\": 18446744073709551615}"),
            ("x.deckcraft", b"x"),
            ("18446744073709551616.deckcraft", b"x"),
        ];
        for (name, body) in junk {
            let _ = std::fs::write(dir.join(name), body);
        }
        std::fs::create_dir_all(dir.join("4.deckcraft")).unwrap();
        let offered = recoverable(&dir);
        let mut s = Session::new();
        for (uid, meta) in &offered {
            let path = meta.get("path").and_then(Value::as_str).map(str::to_string);
            if let Ok(bytes) = read_entry(&dir, *uid) {
                assert!(reopen(&mut s, *uid, &bytes, path, "x").is_err(), "{uid}");
            }
        }
        assert!(s.documents().is_empty());
        assert!(open(&mut s, &dir).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn uids_are_unique_beyond_this_process() {
        let a = DocState::new(Default::default(), None, "a".into());
        let b = DocState::new(Default::default(), None, "b".into());
        assert_ne!(a.uid, b.uid);
        // Not the small per-process numbers every instance used to start from.
        assert!(a.uid >= 1 << 24 && a.uid <= UID_MASK);
    }

    #[test]
    fn another_running_instance_keeps_its_entries() {
        let dir = temp("recovery-shared");
        std::fs::create_dir_all(&dir).unwrap();
        // Another instance (its own owner id and uid, its lock held) has an unsaved presentation.
        let other = instance_id() ^ (1 << 40);
        let lock = File::create(lock_path(&dir, other)).unwrap();
        lock.try_lock().unwrap();
        let mut o = Session::new();
        o.execute("file.new", &json!({})).unwrap();
        o.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        let bytes = deckcraft_format::save(&o.doc().unwrap().doc).unwrap();
        let uid = other + 5;
        let (d, m) = files(&dir, uid);
        std::fs::write(&d, bytes).unwrap();
        std::fs::write(&m, json!({"title": "Theirs", "owner": other}).to_string()).unwrap();
        // This instance saving its clean and dirty presentations leaves that entry alone...
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        assert_eq!(save(&s, &dir).unwrap(), 1);
        assert!(list(&dir).iter().any(|e| e.0 == uid));
        // ...and a recovery open here skips it while its owner runs.
        let mut s2 = Session::new();
        let opened = open(&mut s2, &dir).unwrap();
        assert!(s2.documents().iter().all(|d| d.uid != uid), "{opened:?}");
        assert!(lock_path(&dir, other).exists(), "a live owner's lock stays");
        // Once the owner is gone, its entry is recovered and its lock file cleaned up.
        drop(lock);
        let mut s3 = Session::new();
        open(&mut s3, &dir).unwrap();
        assert!(s3.documents().iter().any(|d| d.uid == uid && d.title().starts_with("Theirs")));
        assert!(!lock_path(&dir, other).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_entries_are_skipped() {
        let dir = temp("recovery-corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("7.deckcraft"), b"not a presentation").unwrap();
        std::fs::write(dir.join("7.json"), b"[1, 2]").unwrap();
        let mut s = Session::new();
        assert!(open(&mut s, &dir).unwrap().is_empty());
        s.recovery_dir = Some(dir.clone());
        let v = s.execute("file.recovery.list", &json!({})).unwrap();
        assert_eq!(v.as_array().map(Vec::len), Some(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A FIFO where an entry or its metadata belongs is skipped rather than read (that blocked
    /// start-up forever), and a folder full of entries reopens only [`MAX_REOPEN`] per start.
    #[cfg(unix)]
    #[test]
    fn hostile_recovery_folders_dont_stall_start_up() {
        let dir = temp("recovery-hostile");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        save(&s, &dir).unwrap();
        let (good, _) = files(&dir, s.doc().unwrap().uid);
        for i in 1..=(MAX_REOPEN as u64 + 3) {
            std::fs::copy(&good, dir.join(format!("{}.deckcraft", 100_000 + i))).unwrap();
        }
        for name in ["5.deckcraft", "6.json"] {
            let _ = std::process::Command::new("mkfifo").arg(dir.join(name)).status();
        }
        std::fs::write(dir.join("6.deckcraft"), std::fs::read(&good).unwrap()).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let d = dir.clone();
        std::thread::spawn(move || {
            let mut s2 = Session::new();
            let _ = tx.send(open(&mut s2, &d).map(|v| v.len()).unwrap_or(0));
        });
        let n = rx.recv_timeout(std::time::Duration::from_secs(60)).expect("recovery hung");
        assert_eq!(n, MAX_REOPEN);
        // Every entry stays on disk (main keeps snapshots until a newer save); the 32 opened plus
        // the rest that wait for the next start.
        assert_eq!(list(&dir).len(), MAX_REOPEN + 5, "the rest wait for the next start");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// NexusOffice: one session per tab in one process. A second session's recovery open (its
    /// Backstage "Recover Unsaved Presentations") never reopens the first session's live entry:
    /// both would write one entry, and closing one without saving would delete the other's.
    #[test]
    fn a_second_session_never_reopens_this_instances_live_entries() {
        let dir = temp("recovery-own");
        let mut a = Session::new();
        a.recovery_dir = Some(dir.clone());
        a.execute("file.new", &json!({})).unwrap();
        a.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        a.execute("file.recovery.save", &json!({})).unwrap();
        let uid = a.doc().unwrap().uid;
        let mut b = Session::new();
        b.recovery_dir = Some(dir.clone());
        b.execute("file.new", &json!({})).unwrap();
        let v = b.execute("file.recovery.open", &json!({})).unwrap();
        assert_eq!(v["opened"], json!([]), "this instance's live entry isn't a crash's");
        assert!(b.documents().iter().all(|d| d.uid != uid));
        assert!(open(&mut b, &dir).unwrap().is_empty());
        assert!(recoverable(&dir).is_empty());
        assert_eq!(read_meta(&dir, uid)["owner"], json!(instance_id()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An offer is acted on as the folder is when it is answered: an entry another window
    /// recovered since (its owner running) is neither taken nor discarded, and an entry taken is
    /// this instance's from then on.
    #[test]
    fn answers_act_on_the_folder_as_it_is_now() {
        let dir = temp("recovery-stale");
        std::fs::create_dir_all(&dir).unwrap();
        let mut o = Session::new();
        o.execute("file.new", &json!({})).unwrap();
        o.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        let bytes = deckcraft_format::save(&o.doc().unwrap().doc).unwrap();
        let dead = instance_id() ^ (1 << 43);
        let uid = dead + 1;
        let (d, m) = files(&dir, uid);
        std::fs::write(&d, &bytes).unwrap();
        std::fs::write(&m, json!({"title": "Họp", "owner": dead}).to_string()).unwrap();
        assert_eq!(recoverable(&dir).len(), 1, "offered");
        // Meanwhile another window recovers it: its metadata names that window, which runs.
        let other = instance_id() ^ (1 << 44);
        let lock = File::create(lock_path(&dir, other)).unwrap();
        lock.try_lock().unwrap();
        std::fs::write(&m, json!({"title": "Họp", "owner": other}).to_string()).unwrap();
        assert!(!discard_left(&dir, uid), "Loại bỏ leaves another window's live entry");
        assert!(d.exists() && m.exists());
        assert_eq!(take(&dir, uid).ok(), Some(false), "Khôi phục doesn't take it too");
        assert_eq!(read_meta(&dir, uid)["owner"], json!(other));
        // That window crashes as well: now it is a crash's again, and taken here.
        drop(lock);
        assert_eq!(take(&dir, uid).ok(), Some(true));
        assert_eq!(read_meta(&dir, uid)["owner"], json!(instance_id()));
        assert_eq!(read_meta(&dir, uid)["title"], json!("Họp"), "the rest of its metadata is kept");
        assert_eq!(take(&dir, uid).ok(), Some(false), "taken once");
        assert!(!discard_left(&dir, uid), "this instance's now");
        assert!(recoverable(&dir).is_empty());
        // Gone (another window discarded it): nothing to take.
        discard(&dir, uid);
        assert_eq!(take(&dir, uid).ok(), Some(false));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A presentation too large to be read back gets no entry (an older one stays), and an entry
    /// over the limit is refused without being read.
    #[test]
    fn entries_over_the_limit_are_neither_written_nor_read() {
        let dir = temp("recovery-large");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        let uid = s.doc().unwrap().uid;
        assert_eq!(save_entries(&s, &dir).unwrap(), Saved { written: 1, too_large: vec![] });
        let before = std::fs::read(files(&dir, uid).0).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        let r = save_capped(&s, &dir, 16).unwrap();
        assert_eq!(r, Saved { written: 0, too_large: vec![uid] });
        assert_eq!(std::fs::read(files(&dir, uid).0).unwrap(), before, "the older entry stays as it was");
        assert_eq!(read_meta(&dir, uid)["stale"], json!(true), "marked as an older version");
        // Small enough again: a current entry, no longer stale.
        save(&s, &dir).unwrap();
        assert_eq!(read_meta(&dir, uid).get("stale"), None);
        // An entry bigger than the limit (a sparse file) is refused by its size.
        let (big, _) = files(&dir, 77);
        File::create(&big).unwrap().set_len(MAX_ENTRY_BYTES + 1).unwrap();
        assert_eq!(read_entry(&dir, 77), Err(EntryError::TooLarge(MAX_ENTRY_BYTES + 1)));
        assert_eq!(read_entry(&dir, 78), Err(EntryError::Unreadable));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A lock file another instance removed under this one (it found the file unlocked an instant
    /// before this one locked it) is made again on the next save: this instance's entries are
    /// never left looking like a crash's.
    #[test]
    fn a_lock_file_removed_under_a_running_instance_comes_back() {
        let dir = temp("recovery-relock");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        save(&s, &dir).unwrap();
        assert!(owner_alive(&dir, instance_id()));
        let _ = std::fs::remove_file(lock_path(&dir, instance_id()));
        assert!(!owner_alive(&dir, instance_id()), "the lock is gone from the folder");
        save(&s, &dir).unwrap();
        assert!(owner_alive(&dir, instance_id()), "locked again");
        // Another instance looking for stale locks now finds this one running.
        let uid = s.doc().unwrap().uid;
        assert!(!crash_left(&dir, &read_meta(&dir, uid)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Start-up sweeps temporary files older than an hour and entries a crash left that were
    /// offered 30 days ago; entries never offered, newer ones, and a running instance's however
    /// old, stay.
    #[test]
    fn old_leftovers_are_swept() {
        let dir = temp("recovery-sweep");
        std::fs::create_dir_all(&dir).unwrap();
        let now = std::time::SystemTime::now();
        let ago = |secs: u64| now.checked_sub(std::time::Duration::from_secs(secs)).unwrap();
        let age = |p: &Path, t: std::time::SystemTime| File::options().write(true).open(p).unwrap().set_modified(t).unwrap();
        let dead = instance_id() ^ (1 << 46);
        let live = instance_id() ^ (1 << 47);
        let lock = File::create(lock_path(&dir, live)).unwrap();
        lock.try_lock().unwrap();
        let month = 31 * 24 * 60 * 60;
        let long_ago = now_secs() - month;
        let entry = |uid: u64, owner: u64, when: std::time::SystemTime, offered: Option<u64>| {
            let (d, m) = files(&dir, uid);
            std::fs::write(&d, b"x").unwrap();
            let meta = match offered {
                Some(t) => json!({"owner": owner, "offered": t}),
                None => json!({"owner": owner}),
            };
            std::fs::write(&m, meta.to_string()).unwrap();
            age(&d, when);
            age(&m, when);
        };
        // Offered a month ago and never taken: swept.
        entry(dead + 1, dead, ago(month), Some(long_ago));
        entry(dead + 2, dead, ago(60), Some(long_ago));
        // Old but never offered (nobody opened a Slide tab, or the clock jumped): kept.
        entry(dead + 4, dead, ago(month), None);
        // Offered just now, though its files are old: kept.
        entry(dead + 5, dead, ago(month), Some(now_secs()));
        entry(live + 1, live, ago(month), Some(long_ago));
        entry(instance_id() + 900, instance_id(), ago(month), Some(long_ago));
        // Half an old entry (its metadata never got its presentation).
        std::fs::write(dir.join(format!("{}.json", dead + 3)), b"{}").unwrap();
        age(&dir.join(format!("{}.json", dead + 3)), ago(month));
        for (name, when) in [("1.deckcraft.tmp", ago(2 * 60 * 60)), ("2.json.tmp", ago(60))] {
            std::fs::write(dir.join(name), b"half").unwrap();
            age(&dir.join(name), when);
        }
        sweep(&dir);
        let mut left: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        let mut want = vec![
            format!("{}.deckcraft", dead + 2),
            format!("{}.json", dead + 2),
            format!("{}.deckcraft", dead + 4),
            format!("{}.json", dead + 4),
            format!("{}.deckcraft", dead + 5),
            format!("{}.json", dead + 5),
            format!("{}.deckcraft", live + 1),
            format!("{}.json", live + 1),
            format!("{}.deckcraft", instance_id() + 900),
            format!("{}.json", instance_id() + 900),
            "2.json.tmp".to_string(),
            format!("owner-{live}.lock"),
        ];
        want.sort();
        assert_eq!(left, want);
        drop(lock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Don't Save on quit (`file.recovery.discard` without a uid) removes the entries of the
    /// presentations open in that session only: not another session's of this process (another
    /// tab's live entry), not another running window's, not what a crash left.
    #[test]
    fn discarding_without_a_uid_keeps_every_other_entry() {
        let dir = temp("recovery-discard-all");
        let mut s = Session::new();
        s.recovery_dir = Some(dir.clone());
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        s.execute("file.recovery.save", &json!({})).unwrap();
        let mine = s.doc().unwrap().uid;
        let mut tab = Session::new();
        tab.recovery_dir = Some(dir.clone());
        tab.execute("file.new", &json!({})).unwrap();
        tab.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        tab.execute("file.recovery.save", &json!({})).unwrap();
        let other_tab = tab.doc().unwrap().uid;
        let live = instance_id() ^ (1 << 48);
        let lock = File::create(lock_path(&dir, live)).unwrap();
        lock.try_lock().unwrap();
        for (uid, owner) in [(live + 1, live), (live + 2, live ^ (1 << 30))] {
            let (d, m) = files(&dir, uid);
            std::fs::write(&d, b"x").unwrap();
            std::fs::write(&m, json!({"owner": owner}).to_string()).unwrap();
        }
        s.execute("file.recovery.discard", &json!({})).unwrap();
        let mut left: Vec<u64> = list(&dir).iter().map(|e| e.0).collect();
        left.sort();
        let mut want = vec![other_tab, live + 1, live + 2];
        want.sort();
        assert_eq!(left, want);
        assert!(!files(&dir, mine).0.exists());
        drop(lock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Entries a host offers are recorded as offered (when first), and kept so.
    #[test]
    fn offered_entries_are_recorded() {
        let dir = temp("recovery-offered");
        std::fs::create_dir_all(&dir).unwrap();
        let dead = instance_id() ^ (1 << 49);
        let (d, m) = files(&dir, dead + 1);
        std::fs::write(&d, b"x").unwrap();
        std::fs::write(&m, json!({"owner": dead}).to_string()).unwrap();
        let first = recoverable(&dir);
        let t = first[0].1["offered"].as_u64().unwrap();
        assert_eq!(read_meta(&dir, dead + 1)["offered"], json!(t));
        std::fs::write(&m, json!({"owner": dead, "offered": 5}).to_string()).unwrap();
        recoverable(&dir);
        assert_eq!(read_meta(&dir, dead + 1)["offered"], json!(5), "the first offer is kept");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Without its lock an instance writes no entry and takes none (other instances would take
    /// them for a crash's): an error each time, nothing on disk.
    #[test]
    fn without_its_lock_nothing_is_written_or_taken() {
        let dir = temp("recovery-nolock");
        std::fs::create_dir_all(lock_path(&dir, instance_id())).unwrap();
        assert!(!hold_lock(&dir));
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
        assert!(save(&s, &dir).is_err());
        assert!(list(&dir).is_empty());
        let dead = instance_id() ^ (1 << 50);
        let (d, m) = files(&dir, dead + 1);
        std::fs::write(&d, b"x").unwrap();
        std::fs::write(&m, json!({"owner": dead}).to_string()).unwrap();
        assert!(take(&dir, dead + 1).is_err());
        assert_eq!(read_meta(&dir, dead + 1)["owner"], json!(dead), "not taken");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
