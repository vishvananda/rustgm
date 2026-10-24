//! Typed path traversal shares canonical names without discarding identifier sites.
use super::{
    model::ImportContexts,
    names::NameId,
    package::{Error, Package},
    unit::FileId,
};
use crate::{
    expand::marks::ItemContext,
    syntax::{
        ast::{Kind, NodeId},
        token_tree::Span,
    },
};
use std::sync::Arc;

pub(super) trait Segments {
    fn reserve(&mut self, additional: usize);
    fn append(
        &mut self,
        package: &Package,
        file: FileId,
        span: Span,
        name: NameId,
    ) -> Result<(), Error>;
}
impl Segments for Vec<NameId> {
    fn reserve(&mut self, additional: usize) {
        Self::reserve(self, additional);
    }
    fn append(&mut self, _: &Package, _: FileId, _: Span, name: NameId) -> Result<(), Error> {
        self.push(name);
        Ok(())
    }
}

impl Package {
    pub(super) fn append_path(
        &mut self,
        file: FileId,
        node: NodeId,
        segments: &mut impl Segments,
    ) -> Result<bool, Error> {
        let value = self.unit(file).configured.ast.node(node);
        let span = value.span;
        match value.kind {
            Kind::CrateRoot(0) => {
                let name = self
                    .names
                    .intern("crate")
                    .map_err(|e| Error::at(file, span.start, e))?;
                segments.append(self, file, span, name)?;
                Ok(false)
            }
            Kind::CrateRoot(_) => Err(Error::at(
                file,
                span.start,
                "external defining crate requires metadata",
            )),
            Kind::Name(span, name) => {
                let name = self.parsed_name(file, span, name)?;
                segments.append(self, file, span, name)?;
                Ok(false)
            }
            Kind::Applied { base, .. } => self.append_path(file, base, segments),
            Kind::Path {
                absolute,
                segments: nodes,
            } => {
                segments.reserve((nodes.end - nodes.start) as usize);
                for offset in nodes.start..nodes.end {
                    let node = self.unit(file).configured.ast.edges[offset as usize];
                    self.append_path(file, node, segments)?;
                }
                Ok(absolute)
            }
            _ => Err(Error::at(file, span.start, "expected a name path")),
        }
    }
}

/// A group traversal appends each local prefix once and restores its previous
/// length after each child. Root identifiers allocate no context storage.
pub(super) struct Buffer {
    pub names: Vec<NameId>,
    contexts: Vec<ItemContext>,
    non_root: usize,
}
impl Buffer {
    pub fn new(mut names: Vec<NameId>) -> Self {
        names.clear();
        Self {
            names,
            contexts: Vec::new(),
            non_root: 0,
        }
    }
    pub fn truncate(&mut self, len: usize) {
        self.names.truncate(len);
        if self.non_root != 0 {
            self.non_root -= self.contexts[len..]
                .iter()
                .filter(|context| **context != ItemContext::ROOT)
                .count();
            self.contexts.truncate(len);
            if self.non_root == 0 {
                self.contexts.clear();
            }
        }
    }
    pub fn freeze_contexts(&self, glob: ItemContext) -> Option<Arc<ImportContexts>> {
        if self.non_root == 0 && glob == ItemContext::ROOT {
            return None;
        }
        let segments = if self.non_root == 0 {
            vec![ItemContext::ROOT; self.names.len()].into_boxed_slice()
        } else {
            self.contexts.clone().into_boxed_slice()
        };
        Some(Arc::new(ImportContexts { segments, glob }))
    }
}
impl Segments for Buffer {
    fn reserve(&mut self, additional: usize) {
        self.names.reserve(additional);
        if self.non_root != 0 {
            self.contexts.reserve(additional);
        }
    }
    fn append(
        &mut self,
        package: &Package,
        file: FileId,
        span: Span,
        name: NameId,
    ) -> Result<(), Error> {
        let context = package.item_context(file, span.start)?;
        if context != ItemContext::ROOT {
            if self.non_root == 0 {
                self.contexts.resize(self.names.len(), ItemContext::ROOT);
            }
            self.non_root += 1;
        }
        if self.non_root != 0 {
            self.contexts.push(context);
        }
        self.names.push(name);
        Ok(())
    }
}
