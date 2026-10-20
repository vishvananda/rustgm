//! Structural auto-trait leaves retain projected proofs in the requesting scope.
use super::{Action, Decision, Declarations, Dependency, Frame, Predicate, Search, inference};
use crate::types::{
    Kind,
    auto_traits::{Obligation, Query, Rule, Step},
    infer::Instantiation,
    parameters::{environment::BuiltinQuery, predicates::Bound},
};
use std::ops::ControlFlow;

pub(super) struct Pending {
    root: Obligation,
    query: Query,
    permissions: Option<Box<BuiltinQuery>>,
    needed: Option<Leaf>,
    stopped: Option<Decision>,
    pub children: Vec<usize>,
    pub end: u32,
}
struct Leaf {
    requested: Obligation,
    proof: Obligation,
    reveal: bool,
}

impl Search<'_> {
    pub(super) fn auto_condition(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        root: Obligation,
    ) -> Result<ControlFlow<Action, bool>, String> {
        let first = frame.available_region();
        let pending = frame.autos.get_or_insert_with(|| {
            Box::new(Pending {
                root,
                query: Query::new(root),
                permissions: None,
                needed: None,
                stopped: None,
                children: Vec::new(),
                end: first,
            })
        });
        if pending.root != root {
            return Err("auto-trait proof changed its canonical subject".into());
        }
        if let Some(decision) = pending.stopped {
            return Ok(ControlFlow::Break(Action::Complete(decision)));
        }
        loop {
            let (step, work) = pending.advance(facts, self.caller.environment)?;
            frame.inputs.context.account_work(work).map_err(inference)?;
            match step {
                Step::Ready(value) => return Ok(ControlFlow::Continue(value)),
                Step::Fields(item) => {
                    return Ok(ControlFlow::Break(Action::Wait(Dependency::Fields(item))));
                }
                Step::Proof(obligation)
                    if matches!(
                        facts.types.kind(obligation.ty),
                        Some(Kind::Projection(_) | Kind::Adt(..))
                    ) && obligation != root =>
                {
                    pending.needed = Some(Leaf {
                        requested: obligation,
                        proof: obligation,
                        reveal: false,
                    });
                    return Ok(ControlFlow::Break(Action::Descend));
                }
                Step::Proof(obligation)
                    if matches!(facts.types.kind(obligation.ty), Some(Kind::Adt(..))) =>
                {
                    let registry = facts
                        .generics
                        .implementations
                        .as_mut()
                        .ok_or("missing explicit auto-trait implementations")?;
                    let candidates = registry
                        .candidates(
                            &mut facts.types,
                            &mut frame.inputs.context,
                            &frame.goal,
                            self.query.availability,
                        )
                        .map_err(inference)?;
                    match candidates {
                        crate::sema::traits::implementations::Candidates::Pending(owners) => {
                            frame.pending_headers = Some(owners.into_iter());
                            return Ok(ControlFlow::Break(self.await_header(facts, frame)?));
                        }
                        crate::sema::traits::implementations::Candidates::Ready(owners) => {
                            if !registry.auto_override(&facts.types, obligation) {
                                pending.query.reconsider(obligation)?;
                                continue;
                            }
                            frame.autos = None;
                            frame.explicit_auto = true;
                            frame.candidates = owners;
                            return Ok(ControlFlow::Break(Action::Continue));
                        }
                    }
                }
                Step::Proof(obligation) => {
                    let (action, steps) = pending.result(facts, obligation)?;
                    frame
                        .inputs
                        .context
                        .account_work(steps)
                        .map_err(inference)?;
                    if let Some(action) = action {
                        return Ok(ControlFlow::Break(action));
                    }
                }
            }
        }
    }

    pub(super) fn auto_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        if frame.normalizing() || frame.forming() {
            return Ok(None);
        }
        let Some(pending) = &frame.autos else {
            return Ok(None);
        };
        let needed = pending
            .needed
            .as_ref()
            .ok_or("missing projected auto-trait subject")?;
        let first = pending.end.max(self.query.allocations.floor());
        let mut substitution =
            Instantiation::new(&mut frame.inputs.context, &facts.types, &[]).map_err(inference)?;
        let goal = frame
            .inputs
            .context
            .instantiate_predicate(
                &mut facts.types,
                Predicate {
                    ty: needed.proof.ty,
                    bound: Bound::Auto(needed.proof.marker),
                },
                &mut substitution,
            )
            .map_err(inference)?;
        let mut child = Frame::new(facts, &mut frame.inputs.context, &goal, first)?;
        child.scope = frame.scope;
        Ok(Some(child))
    }

    pub(super) fn accept_auto(
        &mut self,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let pending = frame
            .autos
            .as_mut()
            .ok_or("missing auto-trait proof frontier")?;
        let needed = pending
            .needed
            .take()
            .ok_or("missing auto-trait proof request")?;
        let satisfied = match decision {
            Decision::Proven(index) => {
                pending.children.push(index);
                pending.end = pending.end.max(self.query.nodes[index].end);
                true
            }
            Decision::Absent => false,
            Decision::Deferred => {
                pending.stopped = Some(decision);
                return Ok(());
            }
        };
        pending.query.answer(needed.requested, satisfied)?;
        Ok(())
    }
}

impl Pending {
    pub(super) fn revealing(&self) -> bool {
        self.needed.as_ref().is_some_and(|leaf| leaf.reveal)
    }

    fn result(
        &mut self,
        facts: &mut Declarations,
        requested: Obligation,
    ) -> Result<(Option<Action>, usize), String> {
        let (valid, mut steps) = crate::sema::traits::results::auto(facts, requested)?;
        if !valid && let Some(Kind::Opaque(id, arguments)) = facts.types.kind(requested.ty) {
            let declaration = facts
                .opaques
                .declarations
                .get(&id)
                .ok_or("missing opaque auto-trait declaration")?;
            let Some(hidden) = declaration.hidden else {
                return Ok((Some(Action::Wait(Dependency::Opaque(id))), steps));
            };
            let mut fold = declaration.substitution(&facts.types, arguments)?;
            let ty = fold.apply(&mut facts.types, hidden)?;
            steps += fold.steps;
            self.needed = Some(Leaf {
                requested,
                proof: Obligation {
                    ty,
                    marker: requested.marker,
                },
                reveal: true,
            });
            return Ok((Some(Action::Descend), steps));
        }
        self.query.answer(requested, valid)?;
        Ok((None, steps))
    }

    fn advance(
        &mut self,
        facts: &mut Declarations,
        environment: Option<&std::sync::Arc<crate::types::parameters::environment::Environment>>,
    ) -> Result<(Step, usize), String> {
        let before = self.query.steps;
        let permissions_before = self.permissions.as_ref().map_or(0, |q| q.steps());
        let step = self.query.advance(
            &mut facts.types,
            &mut facts.aggregates,
            |types, obligation| {
                if environment.map_or(Ok(false), |environment| {
                    environment.matching_auto(types, obligation, &mut self.permissions)
                })? {
                    Ok(Rule::Assumed)
                } else if facts
                    .generics
                    .implementations
                    .as_ref()
                    .is_some_and(|registry| registry.auto_override(types, obligation))
                {
                    Ok(Rule::External)
                } else {
                    Ok(Rule::Structural)
                }
            },
        )?;
        let work = (self.query.steps - before).saturating_add(
            self.permissions.as_ref().map_or(0, |q| q.steps()) - permissions_before,
        );
        Ok((step, work))
    }
}
