//! Configured module instances share parsed files while retaining lexical owners.
use super::{
    names::{NameId, Names},
    unit::{FileId, NodeKey, Unit},
};
use crate::syntax::{ast::NodeId, token_tree::Span};
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
