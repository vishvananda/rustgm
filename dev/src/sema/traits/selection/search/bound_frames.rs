//! Family and inherited-body clauses share one canonical binder-opening path.
use super::{Context, Declarations, Frame, Search, inference};
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::{infer::Instantiation, lifetime_proof::ScopeId, parameters::clauses::Clause},
};

#[derive(Clone, Copy)]
pub(super) struct ClauseRequest {
    pub clause: Clause,
    pub scope: ScopeId,
}
impl Search<'_> {
    pub(super) fn clause_frame<const CHARGED: bool>(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        request: ClauseRequest,
        universes: &mut HashMap<u32, ScopeId>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Frame, String> {
        let mut first = self.query.allocations.reserve_scoped();
        let (predicate, scope) = match request.clause {
            Clause::Predicate(predicate) => (predicate, request.scope),
            Clause::ForAll(id) => {
                if self.caller.retain_sized {
                    return Err(
                        "quantified family conditions require scoped sized requirements".into(),
                    );
                }
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("missing family condition binder")?;
                if CHARGED && self.query.scopes.is_none() {
                    charge(1, size_of::<super::scopes::Scopes>())?;
                }
                let scopes = self.query.scopes.get_or_insert_with(Default::default);
                let scope = if let Some(&scope) = universes.get(&value.bound_regions) {
                    scope
                } else {
                    if CHARGED {
                        let growth =
                            Growth::map::<u32, ScopeId>(universes.len(), universes.capacity(), 1);
                        charge(1 + growth.work, growth.bytes)?;
                    }
                    context
                        .account_work(value.bound_regions as usize)
                        .map_err(inference)?;
                    let scope = if CHARGED {
                        scopes.introduce_counted(
                            request.scope,
                            value.bound_regions,
                            first,
                            charge,
                        )?
                    } else {
                        scopes.introduce(request.scope, value.bound_regions, first)?
                    };
                    universes.insert(value.bound_regions, scope);
                    scope
                };
                let (predicate, next, work) = if CHARGED {
                    scopes.open_counted(facts, scope, value.predicate, first, charge)?
                } else {
                    scopes.open(facts, scope, value.predicate, first)?
                };
                context.account_work(work).map_err(inference)?;
                first = next;
                self.query.allocations.record(next);
                (predicate, scope)
            }
        };
        let mut child_context = Context::default();
        if CHARGED {
            charge(1, size_of::<Frame>())?;
        }
        let child = (|| {
            let mut substitution =
                Instantiation::new(&mut child_context, &facts.types, &[]).map_err(inference)?;
            let goal = child_context
                .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
                .map_err(inference)?;
            Frame::new(facts, &mut child_context, &goal, first)
        })();
        if CHARGED {
            charge(child_context.steps, 0)?;
        }
        let mut child = child?;
        context
            .account_work(child_context.steps)
            .map_err(inference)?;
        child.scope = scope;
        Ok(child)
    }
}
