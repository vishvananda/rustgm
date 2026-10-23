//! Configured module instances share parsed files while retaining lexical owners.
use super::{
    names::{NameId, Names},
    unit::{FileId, NodeKey, Unit},
};
use crate::expand::stream::{ContextId, Origin};
use crate::support::hash::HashMap;
use crate::syntax::{
    ast::{Kind, NodeId},
    token_tree::Span,
};

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
    pub(super) source: Option<Box<super::bindings::Source>>,
    pub(super) library: Option<Box<super::stability::Library>>,
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
    pub(crate) introduction: ContextId,
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
            introduction: ContextId(0),
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
            introduction: ContextId(0),
        })
    }
    pub(crate) fn unlocated(message: impl Into<String>) -> Self {
        Self::from(ErrorData {
            file: DiagnosticFile::Unlocated,
            span: Span { start: 0, end: 0 },
            message: message.into(),
            expansions: Vec::new(),
            introduction: ContextId(0),
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
    pub fn expansion_context(&self, mut file: FileId, mut offset: u32) -> Result<ContextId, Error> {
        for _ in 0..4096 {
            let Some(origin) = self.origin(file, offset) else {
                return Ok(ContextId(0));
            };
            if let Some(context) = origin.context {
                return Ok(context);
            }
            offset = origin.original.span.start
                + (offset - origin.generated.start)
                    .min(origin.original.span.end - origin.original.span.start);
            file = origin.original.file;
        }
        Err(Error::at(
            file,
            offset,
            "expansion provenance chain exceeds 4096",
        ))
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
        match self.unit(file).configured.ast.node(node).kind {
            Kind::CrateRoot(0) => {
                let name = self.names.intern("crate").map_err(|e| {
                    Error::at(
                        file,
                        self.unit(file).configured.ast.node(node).span.start,
                        e,
                    )
                })?;
                names.push(name);
                Ok(false)
            }
            Kind::CrateRoot(_) => Err(Error::at(
                file,
                self.unit(file).configured.ast.node(node).span.start,
                "external defining crate requires metadata",
            )),
            Kind::Name(span, name) => {
                names.push(self.parsed_name(file, span, name)?);
                Ok(false)
            }
            Kind::Applied { base, .. } => self.path_names_into(file, base, names),
            Kind::Path { absolute, segments } => {
                names.reserve((segments.end - segments.start) as usize);
                for offset in segments.start..segments.end {
                    let id = self.unit(file).configured.ast.edges[offset as usize];
                    self.path_names_into(file, id, names)?;
                }
                Ok(absolute)
            }
            _ => Err(Error::at(
                file,
                self.unit(file).configured.ast.node(node).span.start,
                "expected a name path",
            )),
        }
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
