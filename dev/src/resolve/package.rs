//! Configured module instances share parsed files while retaining lexical owners.
use super::{
    names::{NameId, Names},
    unit::{FileId, NodeKey, Unit},
};
use crate::syntax::{
    ast::{Kind, NodeId},
    token_tree::Span,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);
pub struct Module {
    pub parent: Option<ModuleId>,
    pub declaration: Option<NodeKey>,
    pub name: NameId,
    pub file: FileId,
    pub root: NodeId,
}
pub struct Package {
    pub units: Vec<Unit>,
    pub modules: Vec<Module>,
    pub names: Names,
    pub children: HashMap<(ModuleId, NodeKey), ModuleId>,
}
#[derive(Debug)]
pub struct Error {
    pub file: FileId,
    pub span: Span,
    pub message: String,
}
impl Error {
    #[must_use]
    pub fn at(file: FileId, offset: u32, message: impl Into<String>) -> Self {
        Self {
            file,
            span: Span {
                start: offset,
                end: offset,
            },
            message: message.into(),
        }
    }
}
impl Package {
    /// Read canonical names directly from a parsed path.
    /// # Errors
    /// Rejects malformed path facts, unsupported crate identities and name overflow.
    pub fn path_names(
        &mut self,
        file: super::unit::FileId,
        node: NodeId,
    ) -> Result<(bool, Vec<NameId>), Error> {
        match self.unit(file).configured.ast.node(node).kind {
            Kind::CrateRoot(0) => {
                let name = self.names.intern("crate").map_err(|e| {
                    Error::at(
                        file,
                        self.unit(file).configured.ast.node(node).span.start,
                        e,
                    )
                })?;
                Ok((false, vec![name]))
            }
            Kind::CrateRoot(_) => Err(Error::at(
                file,
                self.unit(file).configured.ast.node(node).span.start,
                "external defining crate requires metadata",
            )),
            Kind::Name(span) => Ok((false, vec![self.name(file, span)?])),
            Kind::Applied { base, .. } => self.path_names(file, base),
            Kind::Path { absolute, segments } => {
                let mut names = Vec::with_capacity((segments.end - segments.start) as usize);
                for offset in segments.start..segments.end {
                    let id = self.unit(file).configured.ast.edges[offset as usize];
                    names.extend(self.path_names(file, id)?.1);
                }
                Ok((absolute, names))
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
