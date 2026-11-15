//! Bring-up stubs are an explicit unqualified profile, never a release contract.
use super::hash::HashMap;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, PoisonError},
};

/// The driver selects the profile at most once, before loading any source.
/// It is process-wide so that strict compilations carry no bring-up state.
static SELECTED: OnceLock<Stubs> = OnceLock::new();

/// Select the unqualified bring-up profile for this process.
/// # Errors
/// Rejects a second selection.
pub fn select(stubs: Stubs) -> Result<(), &'static str> {
    SELECTED
        .set(stubs)
        .map_err(|_| "the bring-up profile was already selected")
}
/// The selected stub list; absent in every strict compilation.
#[must_use]
pub fn selected() -> Option<&'static Stubs> {
    SELECTED.get()
}

/// A guest crate as the driver names it: its crate name and root source.
#[derive(Clone, Debug)]
pub struct Guest {
    name: String,
    root: PathBuf,
}
impl Guest {
    /// # Errors
    /// Rejects a name that is not a plain crate identifier.
    pub fn new(name: &str, root: PathBuf) -> Result<Self, &'static str> {
        let mut characters = name.chars();
        let plain = characters
            .next()
            .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
            && characters.all(|next| next == '_' || next.is_ascii_alphanumeric());
        if !plain || matches!(name, "_" | "self" | "Self" | "super" | "crate") {
            return Err("a guest crate name must be a plain identifier");
        }
        Ok(Self {
            name: name.to_owned(),
            root,
        })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// The selected profile's guest crates in driver order: further crate roots
/// that the session resolves beside its own.
static GUESTS: OnceLock<Vec<Guest>> = OnceLock::new();

/// Give the selected profile its guest crates.
/// # Errors
/// Refused outside the bring-up profile, and when given twice.
pub fn admit_guests(guests: Vec<Guest>) -> Result<(), &'static str> {
    if selected().is_none() {
        return Err("guest crates require the bring-up profile");
    }
    GUESTS
        .set(guests)
        .map_err(|_| "the bring-up guest crates were already given")
}
/// The session's guest crates; none in every strict compilation.
#[must_use]
pub fn guests() -> &'static [Guest] {
    GUESTS.get().map_or(&[], Vec::as_slice)
}

/// Crate-level publication fences passed over, in first-met order.
static WAIVED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Pass over a crate-level publication fence: the artifact then lacks the
/// fact that the fence protects, and the driver reports the waiver. Refused
/// outside the bring-up profile, where every fence stops publication.
pub fn waive(fence: &str) -> bool {
    if selected().is_none() {
        return false;
    }
    let mut waived = WAIVED.lock().unwrap_or_else(PoisonError::into_inner);
    if !waived.iter().any(|seen| seen == fence) {
        waived.push(fence.to_owned());
    }
    true
}
/// Withhold a library's source metadata that a publication step could not
/// produce, so that its code is still lowered. No consumer can bind the
/// artifact's declarations. Refused outside the bring-up profile.
#[must_use]
pub fn withhold(reason: &str) -> bool {
    waive(&format!("source metadata is withheld: {reason}"))
}
/// A publication check's result, or success once its fence was waived.
/// # Errors
/// Returns the check's failure outside the bring-up profile.
pub fn waiving(result: Result<(), &'static str>) -> Result<(), &'static str> {
    match result {
        Err(fence) if waive(fence) => Ok(()),
        other => other,
    }
}
/// Every fence waived so far.
#[must_use]
pub fn waived() -> Vec<String> {
    WAIVED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Located failures that a stage set aside to let its other bodies report.
static DEFERRED: Mutex<Vec<Deferred>> = Mutex::new(Vec::new());

/// Where a failure set aside was raised, within its source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    /// A byte offset.
    Byte(u32),
    /// A syntax node index, for stages that do not see source text.
    Node(u32),
}
/// One failure set aside: its source file index, position and message.
#[derive(Clone, Debug)]
pub struct Deferred {
    pub file: u32,
    pub position: Position,
    pub message: String,
}
/// Set a located failure aside. Refused outside the bring-up profile, where
/// every failure stops its stage at once.
pub fn defer(file: u32, position: Position, message: &str) -> bool {
    if selected().is_none() {
        return false;
    }
    DEFERRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(Deferred {
            file,
            position,
            message: message.to_owned(),
        });
    true
}
/// Take every failure set aside so far, in the order the stages met them.
#[must_use]
pub fn deferred() -> Vec<Deferred> {
    std::mem::take(&mut *DEFERRED.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Soundness proofs passed over, in first-met order.
static UNPROVEN: Mutex<Vec<Deferred>> = Mutex::new(Vec::new());

/// Pass over a failed soundness proof of one body: a borrow, lifetime or
/// const-cleanup check whose only result is acceptance, made of source that
/// rustc already accepted. The body keeps its code and the driver reports the
/// proof as missing. Refused outside the bring-up profile, where it stops the
/// stage.
pub fn unproven(file: u32, position: Position, message: &str) -> bool {
    if selected().is_none() {
        return false;
    }
    let mut unproven = UNPROVEN.lock().unwrap_or_else(PoisonError::into_inner);
    if !unproven
        .iter()
        .any(|seen| seen.file == file && seen.position == position && seen.message == message)
    {
        unproven.push(Deferred {
            file,
            position,
            message: message.to_owned(),
        });
    }
    true
}
/// Every soundness proof passed over so far.
#[must_use]
pub fn unproven_proofs() -> Vec<Deferred> {
    UNPROVEN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Function items replaced by trapping stubs, keyed by crate-relative source
/// path and the item's byte offset. An empty list still selects the profile.
#[derive(Debug, Default)]
pub struct Stubs {
    files: HashMap<String, Vec<u32>>,
    entries: usize,
}
impl Stubs {
    /// Parse `PATH<TAB>OFFSET[<TAB>NOTE...]` rows. Blank rows and rows that
    /// begin with `#` are ignored; notes are for people and are not read.
    /// # Errors
    /// Rejects a row without a path, a decimal offset or with a repeated key.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut stubs = Self::default();
        for (index, row) in text.lines().enumerate() {
            if row.is_empty() || row.starts_with('#') {
                continue;
            }
            let mut fields = row.split('\t');
            let path = fields.next().filter(|path| !path.is_empty());
            let offset = fields.next().and_then(|offset| offset.parse::<u32>().ok());
            let (Some(path), Some(offset)) = (path, offset) else {
                return Err(format!(
                    "bring-up skip list row {}: expected PATH<TAB>OFFSET",
                    index + 1
                ));
            };
            let offsets = stubs.files.entry(path.to_owned()).or_default();
            if offsets.contains(&offset) {
                return Err(format!(
                    "bring-up skip list row {}: repeated entry",
                    index + 1
                ));
            }
            offsets.push(offset);
            stubs.entries += 1;
        }
        Ok(stubs)
    }
    #[must_use]
    pub fn contains(&self, path: &str, offset: u32) -> bool {
        self.files
            .get(path)
            .is_some_and(|offsets| offsets.contains(&offset))
    }
    /// Listed entries, including any that match no function in this crate.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries == 0
    }
}
