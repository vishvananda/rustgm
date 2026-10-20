//! Unsizing follows one physical tail; marker arguments are not extra obligations.
use super::{Checker, Constructor, Error, NodeId, Pending, TermId};
use crate::types::{ItemId, TypeId, infer::arguments::Argument};
use std::collections::HashSet;

type Pair = (TermId, TermId);
struct Selected {
    view: TermId,
    conversion: crate::types::unsizing::Conversion<TermId>,
}
enum Parent {
    Structural {
        item: ItemId,
        arguments: Vec<Argument>,
        slot: usize,
    },
    Projected {
        view: TermId,
        target_field: TermId,
    },
}
enum Input {
    Ready(Selected),
    Tail(Parent, Pair),
    Unsupported,
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn unsizing_referent(
        &mut self,
        node: NodeId,
        source: TermId,
        target: TermId,
    ) -> Result<Option<Pending>, Error> {
        self.advance_projections(node)?;
        // A lifetime-only or still-inferable equality is not an unsizing demand.
        // Inspect the graph once at the root, never once per nested wrapper.
        if !self
            .state
            .infer
            .unsizing_mismatch(&self.facts.types, source, target)
            .map_err(|e| self.error(node, e))?
        {
            return Ok(None);
        }
        let mut pair = (source, target);
        let mut parents = Vec::new();
        let mut active = HashSet::new();
        let mut relations = Vec::new();
        let mut result = loop {
            self.state.reference_steps += 1;
            if self.state.reference_steps > 1_000_000 {
                return Err(self.error(node, "unsizing selection work budget exceeded"));
            }
            self.facts.body_work.unsizing_inputs += 1;
            self.advance_projections(node)?;
            pair = (
                self.state
                    .infer
                    .root(pair.0)
                    .map_err(|e| self.error(node, e))?,
                self.state
                    .infer
                    .root(pair.1)
                    .map_err(|e| self.error(node, e))?,
            );
            if pair.0 == pair.1 {
                return Ok(None);
            }
            if !active.insert(pair) {
                return Err(self.error(node, "recursive unsizing obligation"));
            }
            match self.unsizing_input(node, pair)? {
                Input::Ready(result) => break result,
                Input::Tail(parent, child) => {
                    parents.push(parent);
                    pair = child;
                }
                Input::Unsupported => return Ok(None),
            }
        };
        for parent in parents.into_iter().rev() {
            match parent {
                Parent::Structural {
                    item,
                    mut arguments,
                    slot,
                } => {
                    arguments[slot] = Argument::Type(result.view);
                    result.view = self
                        .state
                        .infer
                        .nominal(item, &arguments)
                        .map_err(|e| self.error(node, e))?;
                }
                Parent::Projected { view, target_field } => {
                    relations.push((result.view, target_field));
                    result.view = view;
                }
            }
        }
        Ok(Some(Pending {
            view: result.view,
            conversion: result.conversion,
            borrow_view: None,
            relations,
        }))
    }
    fn unsizing_input(&mut self, node: NodeId, pair: Pair) -> Result<Input, Error> {
        let heads = (self.head(node, pair.0)?, self.head(node, pair.1)?);
        if matches!(heads.1, Some(Constructor::Dynamic { .. })) {
            let upcast = matches!(heads.0, Some(Constructor::Dynamic { .. }));
            return self.object_input(node, pair, upcast);
        }
        if matches!(
            heads,
            (
                Some(
                    Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)
                ),
                Some(Constructor::Slice)
            )
        ) {
            let element = self.child(node, pair.0, 0)?;
            let target_element = self.child(node, pair.1, 0)?;
            self.callable_shape(node, element, target_element)?;
            let view = self
                .state
                .infer
                .construct(Constructor::Slice, &[element])
                .map_err(|e| self.error(node, e))?;
            return Ok(Input::Ready(Selected {
                view,
                conversion: crate::types::unsizing::Conversion::Slice(pair.0),
            }));
        }
        let (
            Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)),
            Some(Constructor::Adt(other, _) | Constructor::AdtArgs(other, _)),
        ) = heads
        else {
            return Ok(Input::Unsupported);
        };
        if item != other {
            return Ok(Input::Unsupported);
        }
        let direct = self
            .facts
            .unsizing_rules
            .get_or_insert_with(Default::default)
            .parameter(&self.facts.types, &self.facts.aggregates, item)
            .map_err(|e| self.error(node, e))?;
        let candidates = if direct.is_none() {
            let candidates = self
                .facts
                .unsizing_rules
                .get_or_insert_with(Default::default)
                .projected_parameters(&self.facts.types, &self.facts.aggregates, item)
                .map(<[u32]>::to_vec)
                .map_err(|e| self.error(node, e))?;
            if candidates.is_empty() {
                return Ok(Input::Unsupported);
            }
            candidates
        } else {
            Vec::new()
        };
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, pair.0)
            .map_err(|e| self.error(node, e))?;
        let targets = self
            .state
            .infer
            .item_arguments(&self.facts.types, pair.1)
            .map_err(|e| self.error(node, e))?;
        if let Some(slot) = direct {
            self.unchanged_unsizing_arguments(node, &[slot], &arguments, &targets)?;
            let slot = slot as usize;
            let (Some(Argument::Type(left)), Some(Argument::Type(right))) =
                (arguments.get(slot), targets.get(slot))
            else {
                return Err(self.error(node, "invalid selected unsizing argument"));
            };
            let child = (*left, *right);
            return Ok(Input::Tail(
                Parent::Structural {
                    item,
                    arguments,
                    slot,
                },
                child,
            ));
        }
        self.unchanged_unsizing_arguments(node, &candidates, &arguments, &targets)?;
        self.projected_unsizing_input(node, item, pair, arguments, &targets, &candidates)
    }

    fn object_input(&mut self, node: NodeId, pair: Pair, upcast: bool) -> Result<Input, Error> {
        self.object_conversion_requirements(node, pair.0, pair.1, upcast)?;
        let conversion = if upcast {
            crate::types::unsizing::Conversion::Upcast {
                source: pair.0,
                target: pair.1,
            }
        } else {
            crate::types::unsizing::Conversion::Object {
                concrete: pair.0,
                target: pair.1,
            }
        };
        Ok(Input::Ready(Selected {
            view: pair.1,
            conversion,
        }))
    }
    fn object_conversion_requirements(
        &mut self,
        node: NodeId,
        concrete: TermId,
        target: TermId,
        upcast: bool,
    ) -> Result<(), Error> {
        let requirements = if upcast {
            self.state
                .infer
                .upcast_requirements(&mut self.facts.types, concrete, target)
        } else {
            self.state
                .infer
                .object_requirements(&mut self.facts.types, concrete, target)
        }
        .map_err(|e| self.error(node, e))?;
        for requirement in requirements {
            let index = self.state.call_types.obligations.len();
            if upcast && requirement.predicate.trait_arguments().is_some() {
                self.state.call_types.require_intrinsic(index);
            }
            if let Some(count) = requirement.bound_regions {
                self.state.call_types.quantify(index, count);
            } else {
                self.defer_bound_inputs(node, &requirement.predicate)?;
            }
            self.state
                .call_types
                .obligations
                .push((node, requirement.predicate));
        }
        Ok(())
    }
    fn projected_unsizing_input(
        &mut self,
        node: NodeId,
        item: ItemId,
        pair: Pair,
        mut arguments: Vec<Argument>,
        targets: &[Argument],
        candidates: &[u32],
    ) -> Result<Input, Error> {
        for &slot in candidates {
            let slot = slot as usize;
            let value = targets
                .get(slot)
                .copied()
                .ok_or_else(|| self.error(node, "missing unsizing target argument"))?;
            *arguments
                .get_mut(slot)
                .ok_or_else(|| self.error(node, "missing unsizing source argument"))? = value;
        }
        let field = self.unsizing_tail(node, item)?;
        let source = self.inferred_field(node, pair.0, item, field)?;
        let target = self.inferred_field(node, pair.1, item, field)?;
        let view = self
            .state
            .infer
            .nominal(item, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.facts.body_work.unsizing_fields += 1;
        Ok(Input::Tail(
            Parent::Projected {
                view,
                target_field: target,
            },
            (source, target),
        ))
    }
    pub(in crate::sema::bodies) fn unsizing_tail(
        &self,
        node: NodeId,
        item: ItemId,
    ) -> Result<TypeId, Error> {
        self.facts
            .aggregates
            .get(item)
            .and_then(|definition| self.facts.aggregates.variants(definition).first())
            .and_then(|variant| self.facts.types.list(variant.fields))
            .and_then(|fields| fields.last())
            .copied()
            .ok_or_else(|| self.error(node, "missing projected unsizing field"))
    }
    fn unchanged_unsizing_arguments(
        &mut self,
        node: NodeId,
        selected: &[u32],
        source: &[Argument],
        target: &[Argument],
    ) -> Result<(), Error> {
        if source.len() != target.len() {
            return Err(self.error(node, "unsizing changed the argument count"));
        }
        let mut selected = selected.iter().copied().peekable();
        for (index, (&source, &target)) in source.iter().zip(target).enumerate() {
            if selected.peek().is_some_and(|&slot| slot as usize == index) {
                selected.next();
                continue;
            }
            match (source, target) {
                (Argument::Lifetime(_), Argument::Lifetime(_)) => {}
                (Argument::Type(left), Argument::Type(right)) => {
                    self.callable_shape(node, left, right)?;
                }
                (Argument::Const(left), Argument::Const(right)) => {
                    self.state
                        .infer
                        .equate_consts(left, right)
                        .map_err(|e| self.error(node, e))?;
                }
                _ => return Err(self.error(node, "unsizing changed an argument kind")),
            }
        }
        Ok(())
    }
}
