//! Select one structural unsizing and retain the source-side lifetime view.
use super::{Checker, Constructor, Error, NodeId, TermId};
mod construction_context;
mod relations;
mod tail_conversion;
use crate::types::infer::arguments::Argument;

pub(super) struct Pending {
    pub view: TermId,
    pub array: TermId,
    pub borrow_view: Option<TermId>,
    pub relations: Vec<(TermId, TermId)>,
}

impl Checker<'_> {
    /// A declared DST result is the common type before joining differently
    /// sized branch operands. Ordinary joins retain their existing inference.
    pub(super) fn contextual_unsizing_result(
        &mut self,
        node: NodeId,
    ) -> Result<Option<TermId>, Error> {
        let Some(&expected) = self.state.unsizing_contexts.get(&node) else {
            return Ok(None);
        };
        let mut term = self.child(node, expected, 0)?;
        let mut nominal = false;
        loop {
            self.state.reference_steps += 1;
            if self.state.reference_steps > 1_000_000 {
                return Err(self.error(node, "unsizing context work budget exceeded"));
            }
            match self.head(node, term)? {
                Some(Constructor::Slice) => return Ok(nominal.then_some(expected)),
                Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) => {
                    nominal = true;
                    let slot = self
                        .facts
                        .unsizing_rules
                        .get_or_insert_with(Default::default)
                        .parameter(&self.facts.types, &self.facts.aggregates, item)
                        .map_err(|e| self.error(node, e))?;
                    let Some(slot) = slot else {
                        let eligible = self
                            .facts
                            .unsizing_rules
                            .get_or_insert_with(Default::default)
                            .projected_parameters(&self.facts.types, &self.facts.aggregates, item)
                            .map(|slots| !slots.is_empty())
                            .map_err(|e| self.error(node, e))?;
                        if !eligible {
                            return Ok(None);
                        }
                        let field = self.unsizing_tail(node, item)?;
                        term = self.inferred_field(node, term, item, field)?;
                        self.advance_projections(node)?;
                        continue;
                    };
                    let arguments = self
                        .state
                        .infer
                        .item_arguments(&self.facts.types, term)
                        .map_err(|e| self.error(node, e))?;
                    let Some(Argument::Type(ty)) = arguments.get(slot as usize) else {
                        return Err(self.error(node, "invalid unsizing context argument"));
                    };
                    term = *ty;
                }
                _ => return Ok(None),
            }
        }
    }
    /// Keep all selected reference ancestors, including their original regions.
    pub(super) fn unsizing_source_view(
        &mut self,
        node: NodeId,
        mut source: TermId,
        depth: u32,
        mut referent: TermId,
    ) -> Result<TermId, Error> {
        let mut parents = Vec::new();
        for _ in 0..depth {
            let Some(head @ Constructor::Reference { .. }) = self.head(node, source)? else {
                return Err(self.error(node, "invalid unsizing reference ancestry"));
            };
            parents.push(head);
            source = self.child(node, source, 0)?;
        }
        for head in parents.into_iter().rev() {
            referent = self
                .state
                .infer
                .construct(head, &[referent])
                .map_err(|e| self.error(node, e))?;
        }
        Ok(referent)
    }
}
