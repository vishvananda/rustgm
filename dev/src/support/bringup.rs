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

/// The pipeline stage that set a failure aside. Every stage after typing
/// records each body it rejects and goes on with the next one; the stage
/// still fails when it ends, so a later stage first runs in the next
/// compilation, once the recorded functions are stubs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Typing,
    /// Construction of control-flow graphs.
    Cfg,
    /// Constant demands of a body's graph.
    Constants,
    Promotion,
    /// Copy requirements of template bodies.
    Copies,
    /// Region and lifetime proofs.
    Lifetimes,
    /// Ownership, initialization and loan proofs of template bodies.
    Borrows,
    /// Const cleanup qualification, drop requirements and destruction proofs.
    Destruction,
    /// Storage of opaque types.
    Opaques,
    /// Method and operator dispatch of the ordinary bodies.
    Dispatch,
    /// Proofs of the ordinary (nongeneric) bodies that lowering consumes.
    Ordinary,
    /// Specialization of demanded generic instances.
    Instances,
    /// Emission of `LowIR` functions.
    Lowering,
}
impl Stage {
    /// The stage's name in bring-up records.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Typing => "typing",
            Self::Cfg => "cfg",
            Self::Constants => "constants",
            Self::Promotion => "promotion",
            Self::Copies => "copies",
            Self::Lifetimes => "lifetimes",
            Self::Borrows => "borrows",
            Self::Destruction => "destruction",
            Self::Opaques => "opaques",
            Self::Dispatch => "dispatch",
            Self::Ordinary => "ordinary",
            Self::Instances => "instances",
            Self::Lowering => "lowering",
        }
    }
}

/// Where a failure set aside was raised, within its source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    /// A byte offset.
    Byte(u32),
    /// A syntax node index, for stages that do not see source text.
    Node(u32),
}
/// One failure set aside: its stage, source file index, position and message.
#[derive(Clone, Debug)]
pub struct Deferred {
    pub stage: Stage,
    pub file: u32,
    pub position: Position,
    pub message: String,
    /// The function that owns the failure wherever it was raised, when the
    /// stage that set it aside knows it; otherwise the innermost function
    /// containing the position owns it.
    pub owner: Option<Owner>,
}
/// A function item by its source file index, syntax node and definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub file: u32,
    pub node: u32,
    pub definition: u32,
}
/// Set a located failure of `stage` aside. Refused outside the bring-up
/// profile, where every failure stops its stage at once.
#[must_use]
pub fn defer(stage: Stage, file: u32, position: Position, message: &str) -> bool {
    push_deferred(stage, file, position, message, None)
}
/// Set aside a failure that belongs to the function `owner` wherever it was
/// raised: a body that failed to type keeps its checked signature and becomes
/// a trapping stub. Refused outside the bring-up profile.
#[must_use]
pub fn defer_owned(file: u32, position: Position, message: &str, owner: Owner) -> bool {
    push_deferred(Stage::Typing, file, position, message, Some(owner))
}
fn push_deferred(
    stage: Stage,
    file: u32,
    position: Position,
    message: &str,
    owner: Option<Owner>,
) -> bool {
    if selected().is_none() {
        return false;
    }
    DEFERRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(Deferred {
            stage,
            file,
            position,
            message: message.to_owned(),
            owner,
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
pub fn unproven(stage: Stage, file: u32, position: Position, message: &str) -> bool {
    if selected().is_none() {
        return false;
    }
    let mut unproven = UNPROVEN.lock().unwrap_or_else(PoisonError::into_inner);
    if !unproven
        .iter()
        .any(|seen| seen.file == file && seen.position == position && seen.message == message)
    {
        unproven.push(Deferred {
            stage,
            file,
            position,
            message: message.to_owned(),
            owner: None,
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
