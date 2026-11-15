//! Bring-up stubs are an explicit unqualified profile, never a release contract.
use super::hash::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

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

/// Located failures that a stage set aside to let its other bodies report.
static DEFERRED: Mutex<Vec<Deferred>> = Mutex::new(Vec::new());

/// Where a failure set aside was raised, within its source file.
#[derive(Clone, Copy, Debug)]
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
