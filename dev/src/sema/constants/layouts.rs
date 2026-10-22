//! Layout prerequisites share the constant work stack, including unexecuted borrows.
use super::{Error, NodeKey, Queries, Query, Type, TypeId, Work};
use crate::support::hash::{HashMap, HashSet};
use crate::{sema::model::ReferenceAlignment, types::aggregate};

#[derive(Default)]
pub(super) struct Scratch {
    active: HashSet<TypeId>,
    sources: HashMap<NodeKey, usize>,
    indexed: usize,
}
impl Queries<'_> {
    pub(super) fn prepare_alignment(&mut self, key: NodeKey) -> Result<(), Error> {
        // Typing already retained packed-place obligations for dead branches.
        // Index appended facts once; do not evaluate places or reconstruct them.
        while self.layout_scratch.indexed < self.typing.facts.reference_alignment.len() {
            self.budget()?;
            let index = self.layout_scratch.indexed;
            let obligation = self.typing.facts.reference_alignment[index];
            self.layout_scratch.sources.insert(obligation.source, index);
            self.layout_scratch.indexed += 1;
        }
        let Some(&index) = self.layout_scratch.sources.get(&key) else {
            return Ok(());
        };
        let mut obligation = self.typing.facts.reference_alignment[index];
        obligation.ty = self.specialized_type(key, obligation.ty)?;
        self.work.push(Work::Alignment(obligation));
        let ty = match self.typing.facts.types.kind(obligation.ty) {
            Some(Type::Primitive(super::Primitive::Str)) => return Ok(()),
            Some(Type::Slice(element)) => element,
            _ => obligation.ty,
        };
        self.work.push(Work::Layout(key, ty, false));
        Ok(())
    }
    pub(super) fn check_alignment(&mut self, obligation: ReferenceAlignment) -> Result<(), Error> {
        let facts = &mut *self.typing.facts;
        let align = facts
            .layouts
            .reference_align(&facts.types, &facts.aggregates, obligation.ty)
            .map_err(|e| self.error(Query::Node(obligation.source), e.message))?;
        if align > obligation.packed {
            return Err(self.error(
                Query::Node(obligation.source),
                "constant reference to packed storage is unaligned",
            ));
        }
        Ok(())
    }
    pub(super) fn layout(&mut self, key: NodeKey, ty: TypeId, finish: bool) -> Result<(), Error> {
        if self.typing.facts.layouts.get(ty).is_some() {
            return Ok(());
        }
        if finish || self.discriminants.is_none() {
            let facts = &mut *self.typing.facts;
            facts
                .layouts
                .demand(&facts.types, &facts.aggregates, ty)
                .map_err(|e| self.error(Query::Node(key), e.message))?;
            self.layout_scratch.active.remove(&ty);
            return Ok(());
        }
        if !self.layout_scratch.active.insert(ty) {
            return Err(self.error(Query::Node(key), "cycle in constant layout dependencies"));
        }
        self.work.push(Work::Layout(key, ty, true));
        match self.typing.facts.types.kind(ty) {
            Some(Type::Tuple(fields)) => self.layout_fields(key, fields)?,
            // Zero-length arrays retain their element's alignment and validity.
            Some(Type::Array { element, .. }) => self.work.push(Work::Layout(key, element, false)),
            Some(Type::Adt(item, _)) => {
                let facts = &*self.typing.facts;
                let shape = facts
                    .aggregates
                    .shape(&facts.types, ty)
                    .map_err(|e| self.error(Query::Node(key), e))?;
                for variant in shape.variants().rev() {
                    let fields = facts
                        .types
                        .list(variant.fields)
                        .ok_or_else(|| self.error(Query::Node(key), "missing layout fields"))?;
                    self.work.extend(
                        fields
                            .iter()
                            .rev()
                            .map(|&field| Work::Layout(key, field, false)),
                    );
                }
                if shape.definition.kind == aggregate::Kind::Enum {
                    self.work.extend(
                        facts
                            .members
                            .variants(item)
                            .iter()
                            .rev()
                            .map(|&variant| Work::Enter(Query::Variant(variant))),
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn layout_fields(&mut self, key: NodeKey, fields: crate::types::ListId) -> Result<(), Error> {
        let fields = self
            .typing
            .facts
            .types
            .list(fields)
            .ok_or_else(|| self.error(Query::Node(key), "missing layout fields"))?;
        self.work.extend(
            fields
                .iter()
                .rev()
                .map(|&field| Work::Layout(key, field, false)),
        );
        Ok(())
    }
}
