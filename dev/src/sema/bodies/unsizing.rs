//! Select one structural unsizing and retain the source-side lifetime view.
use super::{Checker, Constructor, Error, Kind, NodeId, TermId};
mod construction_context;
mod elaboration;
mod relations;
mod tail_conversion;
mod wrappers;
use crate::types::infer::arguments::Argument;

pub(super) struct Pending {
    pub view: TermId,
    pub conversion: crate::types::unsizing::Conversion<TermId>,
    pub justification: Justification,
    pub relations: Vec<(TermId, TermId)>,
}

#[derive(Clone, Copy)]
pub(super) enum Justification {
    Structural,
    Borrow(TermId),
    /// Stable index of this body's actual canonical `CoerceUnsized` goal.
    NominalGoal(u32),
}
const _: [(); 8] = [(); size_of::<Justification>()];
const _: [(); 48] = [(); size_of::<Pending>()];
const _: [(); 12] = [(); size_of::<crate::types::unsizing::Conversion<crate::types::TypeId>>()];
impl Pending {
    pub(super) fn borrow_view(&self) -> Option<TermId> {
        match self.justification {
            Justification::Borrow(view) => Some(view),
            Justification::Structural | Justification::NominalGoal(_) => None,
        }
    }
    pub(super) fn retain_borrow(&mut self, view: TermId) -> Result<(), &'static str> {
        if matches!(self.justification, Justification::NominalGoal(_)) {
            return Err("nominal whole-value coercion cannot replace a required source borrow");
        }
        self.justification = Justification::Borrow(view);
        Ok(())
    }
}

impl Checker<'_> {
    pub(super) fn retain_nominal_branch_expectation(&mut self, node: NodeId, expected: TermId) {
        if self.facts.source_bindings.is_some()
            && matches!(
                self.kind(node),
                Kind::If {
                    otherwise: Some(_),
                    ..
                } | Kind::Match { .. }
            )
        {
            self.state.branch_expectations.insert(node, expected);
        }
    }

    /// A hard expectation supplies the common type for branch unsizing or
    /// when a nominal result can constrain a pending source projection.
    pub(super) fn contextual_branch_result(
        &mut self,
        node: NodeId,
    ) -> Result<Option<TermId>, Error> {
        let Some(&expected) = self.state.branch_expectations.get(&node) else {
            return Ok(None);
        };
        match self.head(node, expected)? {
            Some(Constructor::Pointer { .. }) => return Ok(Some(expected)),
            Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)) => {
                return self.nominal_branch_result(node, owner, expected);
            }
            Some(Constructor::Reference { .. }) => {}
            _ => return Ok(None),
        }
        let mut term = self.child(node, expected, 0)?;
        // The expected reference gives each arm a directed coercion site.
        // An open referent still needs common-type inference across the arms.
        if crate::sema::compiler_items::dereferencing::available(self.facts)
            && self
                .state
                .infer
                .inputs_fixed(&self.facts.types, term)
                .map_err(|error| self.error(node, error))?
            && self.contextual_head(node, term)?.is_some()
        {
            return Ok(Some(expected));
        }
        loop {
            self.state.reference_steps += 1;
            if self.state.reference_steps > 1_000_000 {
                return Err(self.error(node, "unsizing context work budget exceeded"));
            }
            match self.head(node, term)? {
                Some(Constructor::Dynamic { .. } | Constructor::Slice) => {
                    return Ok(Some(expected));
                }
                Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) => {
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
    fn nominal_branch_result(
        &mut self,
        node: NodeId,
        owner: crate::types::ItemId,
        expected: TermId,
    ) -> Result<Option<TermId>, Error> {
        // Each arm must supply its own checked coercion to a settled wrapper
        // expectation. An open target still needs the arms' common-type inference.
        if let Some(rules) = self.facts.unsizing_rules.as_deref()
            && rules
                .nominal_hint_slots(&self.facts.types, owner)
                .map_err(|error| self.error(node, error))?
                .is_some()
            && self
                .state
                .infer
                .inputs_fixed(&self.facts.types, expected)
                .map_err(|error| self.error(node, error))?
        {
            return Ok(Some(expected));
        }
        self.projected_if_result(node, expected)
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
