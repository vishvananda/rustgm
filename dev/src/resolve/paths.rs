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

/// Borrowed canonical names paired with the identity of every actual identifier.
/// Prefix selection keeps the same provenance; ordinary source needs no side array.
#[derive(Clone, Copy)]
pub(super) struct Identified<'a> {
    pub names: &'a [NameId],
    contexts: Option<&'a [ItemContext]>,
}
impl<'a> Identified<'a> {
    pub fn new(names: &'a [NameId], contexts: &'a [ItemContext]) -> Self {
        Self {
            names,
            contexts: Some(contexts),
        }
    }

    pub fn imported(import: &'a super::model::Import) -> Self {
        Self {
            names: &import.path,
            contexts: import
                .contexts
                .as_ref()
                .map(|contexts| contexts.segments.as_ref()),
        }
    }

    pub fn prefix(self, length: usize) -> Self {
        Self {
            names: &self.names[..length],
            contexts: self.contexts,
        }
    }

    pub fn context(self, index: usize) -> Result<ItemContext, &'static str> {
        if index >= self.names.len() {
            return Err("missing path identifier");
        }
        self.contexts.map_or(Ok(ItemContext::ROOT), |contexts| {
            contexts
                .get(index)
                .copied()
                .ok_or("missing path identifier context")
        })
    }
}
impl std::ops::Deref for Identified<'_> {
    type Target = [NameId];
    fn deref(&self) -> &Self::Target {
        self.names
    }
}

/// Registered calls retain only immutable slices, with no root-context allocation.
pub(super) struct Saved {
    names: Box<[NameId]>,
    contexts: Option<Box<[ItemContext]>>,
}
impl Saved {
    pub fn view(&self) -> Identified<'_> {
        Identified {
            names: &self.names,
            contexts: self.contexts.as_deref(),
        }
    }
}
impl std::ops::Deref for Saved {
    type Target = [NameId];
    fn deref(&self) -> &Self::Target {
        &self.names
    }
}

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
#[derive(Default)]
pub(super) struct Buffer {
    pub names: Vec<NameId>,
    contexts: Vec<ItemContext>,
    non_root: usize,
}
impl Buffer {
    pub fn save(self) -> Saved {
        Saved {
            names: self.names.into_boxed_slice(),
            contexts: (self.non_root != 0).then(|| self.contexts.into_boxed_slice()),
        }
    }
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
    pub fn view(&self) -> Identified<'_> {
        Identified {
            names: &self.names,
            contexts: (self.non_root != 0).then_some(self.contexts.as_slice()),
        }
    }

    /// `local_inner_macros` introduces an explicit defining-crate prefix.
    pub fn prepend_root(&mut self, name: NameId) {
        self.names.insert(0, name);
        if self.non_root != 0 {
            self.contexts.insert(0, ItemContext::ROOT);
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
impl std::ops::Deref for Buffer {
    type Target = [NameId];
    fn deref(&self) -> &Self::Target {
        &self.names
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
