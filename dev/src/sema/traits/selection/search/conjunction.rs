//! Canonical declaration requirements share one caller's real formation graph.
use super::{Caller, Context, Decision, Declarations, Frame, Predicate, Search, inference};
use crate::{
    sema::traits::{implementations::Availability, obligations::Obligations, selection::Failure},
    types::{TypeId, infer::Instantiation},
};

pub(in crate::sema) enum Conjunction {
    Proven {
        proof: Obligations,
        sized: Vec<TypeId>,
        provisional_size: bool,
        selected: Vec<crate::sema::generics::ImplementationSelection>,
    },
    Unproven(usize),
}

enum Roots {
    Proven(Vec<usize>),
    Unproven(usize),
    Refuted {
        index: usize,
        root: usize,
        expected: Predicate,
    },
}

/// Each root is checked normally. Completed formation and child selections stay
/// inside this single query; they never become independent cross-caller facts.
pub(in crate::sema) fn prove_conjunction(
    facts: &mut Declarations,
    context: &mut Context,
    requirements: &[Predicate],
    caller: Caller<'_>,
    availability: Availability,
) -> Result<Conjunction, Failure> {
    let mut search = Search::new(caller, availability);
    let roots = search.conjunction_roots(facts, context, requirements);
    let charged = context.account_work(search.query.steps).map_err(inference);
    let roots = roots?;
    charged?;
    match roots {
        Roots::Proven(roots) => search.finish_conjunction(facts, roots).map_err(Into::into),
        Roots::Unproven(index) => Ok(Conjunction::Unproven(index)),
        Roots::Refuted {
            index,
            root,
            expected,
        } => {
            search
                .refutation(facts, root, expected)?
                .reject()
                .map_err(str::to_owned)?;
            Ok(Conjunction::Unproven(index))
        }
    }
}

impl Search<'_> {
    fn conjunction_roots(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        requirements: &[Predicate],
    ) -> Result<Roots, Failure> {
        self.account_query_work(requirements.len().saturating_mul(3))?;
        let mut roots = Vec::new();
        roots
            .try_reserve(requirements.len())
            .map_err(|_| "declaration requirement roots allocation failed".to_owned())?;
        let mut substitution = Instantiation::new(context, &facts.types, &[]).map_err(inference)?;
        for (index, &predicate) in requirements.iter().enumerate() {
            if index != 0 {
                self.query.resume_sibling().map_err(str::to_owned)?;
            }
            // Retained sibling proofs own every prior witness, including failed
            // candidate allocations. Later roots cannot reuse those identities.
            let first = self.query.allocations.reserve_scoped();
            let goal = context
                .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
                .map_err(inference)?;
            let frame = Frame::new(facts, context, &goal, first)?;
            match self.run_checked(facts, frame)? {
                Decision::Proven(root) => {
                    context
                        .equate_proved_predicate(
                            &mut facts.types,
                            &goal,
                            self.query.nodes[root].goal,
                        )
                        .map_err(inference)?;
                    roots.push(root);
                }
                Decision::Refuted(root) => {
                    let expected = context
                        .resolved_predicate(&mut facts.types, &goal)
                        .map_err(inference)?;
                    return Ok(Roots::Refuted {
                        index,
                        root,
                        expected,
                    });
                }
                Decision::Absent | Decision::Deferred => return Ok(Roots::Unproven(index)),
            }
        }
        Ok(Roots::Proven(roots))
    }

    fn finish_conjunction(
        mut self,
        facts: &Declarations,
        roots: Vec<usize>,
    ) -> Result<Conjunction, String> {
        let mut provisional_size = false;
        let mut selected = Vec::new();
        for &root in &roots {
            let node = &self.query.nodes[root];
            provisional_size |= node.provisional_size;
            if let Some(super::Source::Implementation { owner, arguments }) = &node.source {
                selected.push(crate::sema::generics::ImplementationSelection {
                    owner: *owner,
                    arguments: *arguments,
                });
            }
        }
        let end = self.query.allocations.end();
        let mut proof = std::mem::replace(
            &mut self.query.allocations,
            super::allocations::Allocations::new(1),
        )
        .finish(end);
        let mut sized = Vec::new();
        // One union traversal visits shared prerequisites once and retains every
        // root's actual region, equality, formation and sizing obligations.
        self.query.reused = true;
        self.collect_graph(facts, roots, None, &mut proof, &mut sized)?;
        Ok(Conjunction::Proven {
            proof,
            sized,
            provisional_size,
            selected,
        })
    }
}
