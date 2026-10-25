//! Configured module instances share parsed files while retaining lexical owners.
mod diagnostic_warnings;
use super::{
    names::{NameId, Names},
    unit::{FileId, NodeKey, Unit},
};
use crate::expand::{
    marks::{ExpansionId, ItemContext, LocalContext, Marks, SyntaxContextId},
    stream::Origin,
};
use crate::support::hash::HashMap;
use crate::syntax::{ast::NodeId, token_tree::Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);
pub struct Module {
    pub implicit_prelude: bool,
    pub parent: Option<ModuleId>,
    pub declaration: Option<NodeKey>,
    pub name: NameId,
    pub file: FileId,
    pub root: NodeId,
}
pub struct Package {
    pub(crate) marks: Marks,
    pub(super) source: Option<Box<super::bindings::Source>>,
    pub(super) library: Option<Box<super::stability::Library>>,
    pub(super) macro_features: super::bindings::policy::MacroFeatures,
    pub(super) runtime_checks: crate::support::runtime_checks::RuntimeCheckConfig,
    pub no_std: bool,
    pub units: Vec<Unit>,
    pub file_paths: Vec<std::path::PathBuf>,
    pub crate_name: String,
    pub modules: Vec<Module>,
    pub names: Names,
    pub children: HashMap<(ModuleId, NodeKey), ModuleId>,
}
#[derive(Debug)]
pub struct ExpansionTrace {
    pub macro_name: String,
    pub call: crate::expand::stream::Location,
    pub definition: Option<crate::expand::stream::Location>,
}
#[derive(Debug)]
pub struct Warning {
    pub code: crate::support::diagnostics::WarningCode,
    pub diagnostic: Error,
}
/// A frontend diagnostic. It is one pointer, so the results that carry it
/// return in registers; diagnostics are rare and their data lives on the heap.
pub struct Error(Box<ErrorData>);
#[derive(Debug)]
pub enum DiagnosticFile {
    Source(FileId),
    Imported(String),
    Unlocated,
}
impl DiagnosticFile {
    #[must_use]
    pub const fn source(&self) -> Option<FileId> {
        match self {
            Self::Source(file) => Some(*file),
            _ => None,
        }
    }
}
#[derive(Debug)]
pub struct ErrorData {
    pub file: DiagnosticFile,
    pub span: Span,
    pub message: String,
    pub expansions: Vec<ExpansionTrace>,
    pub(crate) introduction: ExpansionId,
    retained_warnings: Vec<Warning>,
    warning_bytes: usize,
    warning_work: usize,
    warning_limit_note: bool,
}
impl Error {
    #[must_use]
    pub fn at(file: FileId, offset: u32, message: impl Into<String>) -> Self {
        Self::from(ErrorData {
            file: DiagnosticFile::Source(file),
            span: Span {
                start: offset,
                end: offset,
            },
            message: message.into(),
            expansions: Vec::new(),
            introduction: ExpansionId(0),
            retained_warnings: Vec::new(),
            warning_bytes: 0,
            warning_work: 0,
            warning_limit_note: false,
        })
    }
    pub(crate) fn imported(
        location: super::body_origin::ImportedDiagnostic,
        message: impl Into<String>,
    ) -> Self {
        let (label, span) = location.into_parts();
        Self::from(ErrorData {
            file: DiagnosticFile::Imported(label),
            span,
            message: message.into(),
            expansions: Vec::new(),
            introduction: ExpansionId(0),
            retained_warnings: Vec::new(),
            warning_bytes: 0,
            warning_work: 0,
            warning_limit_note: false,
        })
    }
    pub(crate) fn unlocated(message: impl Into<String>) -> Self {
        Self::from(ErrorData {
            file: DiagnosticFile::Unlocated,
            span: Span { start: 0, end: 0 },
            message: message.into(),
            expansions: Vec::new(),
            introduction: ExpansionId(0),
            retained_warnings: Vec::new(),
            warning_bytes: 0,
            warning_work: 0,
            warning_limit_note: false,
        })
    }
}
impl From<ErrorData> for Error {
    #[cold]
    fn from(data: ErrorData) -> Self {
        Self(Box::new(data))
    }
}
impl std::ops::Deref for Error {
    type Target = ErrorData;
    fn deref(&self) -> &ErrorData {
        &self.0
    }
}
impl std::ops::DerefMut for Error {
    fn deref_mut(&mut self) -> &mut ErrorData {
        &mut self.0
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(output)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.file {
            DiagnosticFile::Source(file) => write!(
                output,
                "file {}: {}: {}",
                file.0, self.span.start, self.message
            )?,
            DiagnosticFile::Imported(label) => write!(
                output,
                "file {label}: {}: {}",
                self.span.start, self.message
            )?,
            DiagnosticFile::Unlocated => write!(output, "input: {}", self.message)?,
        }
        if self.warning_limit_note {
            write!(
                output,
                "\n  note: retained warning work or storage budget exceeded; further warnings unavailable"
            )?;
        }
        for trace in &self.expansions {
            write!(
                output,
                "\n  in macro {} at file {}: {}",
                trace.macro_name, trace.call.file.0, trace.call.span.start
            )?;
            if let Some(definition) = trace.definition {
                write!(
                    output,
                    "; defined at file {}: {}",
                    definition.file.0, definition.span.start
                )?;
            } else {
                write!(output, "; builtin")?;
            }
        }
        Ok(())
    }
}
impl std::error::Error for Error {}
impl Package {
    pub(crate) fn coherence_is_core(&self) -> bool {
        self.macro_features.coherence()
    }
    pub(crate) fn preserves_ub_checks(&self) -> bool {
        self.macro_features.preserve_ub()
    }
    pub(crate) fn requested_cfg_ub_checks(&self) -> bool {
        self.macro_features.cfg_ub()
    }

    /// The source loader copies this immutable compilation input once.
    #[must_use]
    pub const fn runtime_checks(&self) -> crate::support::runtime_checks::RuntimeCheckConfig {
        self.runtime_checks
    }

    pub(crate) fn requested_internal_attributes(&self) -> bool {
        self.macro_features.internal()
    }
    pub(crate) fn requested_unboxed_closures(&self) -> bool {
        self.macro_features.unboxed()
    }
    /// Library requests and completed policy are published only by resolution.
    #[must_use]
    pub fn library_policy(&self) -> Option<&super::stability::Library> {
        self.library.as_deref()
    }

    pub(crate) fn origin(&self, file: FileId, offset: u32) -> Option<Origin> {
        let origins = &self.unit(file).origins;
        // Generated text is appended after the original source, so offsets
        // before the first generated span belong to no expansion.
        if origins
            .first()
            .is_none_or(|origin| offset < origin.generated.start)
        {
            return None;
        }
        let index = origins.partition_point(|origin| {
            origin.generated.end <= offset && origin.generated.start != offset
        });
        origins
            .get(index)
            .filter(|origin| origin.generated.start <= offset && offset <= origin.generated.end)
            .copied()
    }
    /// # Errors
    /// Rejects malformed or excessively deep expansion provenance.
    pub fn syntax_context(&self, file: FileId, offset: u32) -> Result<SyntaxContextId, Error> {
        super::unit::contexts::at(&self.units, file, offset)
    }
    /// # Errors
    /// Rejects missing source provenance or a token context outside this package.
    pub(crate) fn local_context(&self, file: FileId, offset: u32) -> Result<LocalContext, Error> {
        self.marks
            .local(self.syntax_context(file, offset)?)
            .map_err(|message| Error::at(file, offset, message))
    }

    pub(crate) fn item_context(&self, file: FileId, offset: u32) -> Result<ItemContext, Error> {
        self.marks
            .item(self.syntax_context(file, offset)?)
            .map_err(|message| Error::at(file, offset, message))
    }

    /// Read canonical names directly from a parsed path.
    /// # Errors
    /// Rejects malformed path facts, unsupported crate identities and name overflow.
    pub fn path_names(
        &mut self,
        file: super::unit::FileId,
        node: NodeId,
    ) -> Result<(bool, Vec<NameId>), Error> {
        let mut names = Vec::new();
        let absolute = self.path_names_into(file, node, &mut names)?;
        Ok((absolute, names))
    }

    /// Append a path's canonical names to caller-owned scratch storage.
    /// # Errors
    /// Rejects malformed path facts, unsupported crate identities and name overflow.
    pub fn path_names_into(
        &mut self,
        file: super::unit::FileId,
        node: NodeId,
        names: &mut Vec<NameId>,
    ) -> Result<bool, Error> {
        self.append_path(file, node, names)
    }

    #[must_use]
    pub fn unit(&self, file: FileId) -> &Unit {
        &self.units[file.0 as usize]
    }
    /// A name the parser interned, or the span's interned spelling.
    /// # Errors
    /// Rejects invalid source spans and name-arena overflow.
    pub fn parsed_name(
        &mut self,
        file: FileId,
        span: Span,
        parsed: Option<NameId>,
    ) -> Result<NameId, Error> {
        match parsed {
            Some(name) => Ok(name),
            None => self.name(file, span),
        }
    }
    /// # Errors
    /// Rejects invalid source spans and name-arena overflow.
    pub fn name(&mut self, file: FileId, span: Span) -> Result<NameId, Error> {
        let unit = &self.units[file.0 as usize];
        let spelling = unit
            .spelling(span)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        self.names
            .intern(spelling)
            .map_err(|e| Error::at(file, span.start, e))
    }
    #[must_use]
    pub fn is_descendant(&self, mut module: ModuleId, ancestor: ModuleId) -> bool {
        loop {
            if module == ancestor {
                return true;
            }
            let Some(parent) = self.modules[module.0 as usize].parent else {
                return false;
            };
            module = parent;
        }
    }
}
