//! Crate-wide reuse of trivially proved concrete body goals in neutral callers.
//!
//! A body goal whose inputs are closed canonical types (no parameters,
//! projections, opaques or regions) and whose caller assumes nothing has an
//! answer independent of the body. Only implementation evidence with an empty
//! proof is reused, and only after a second identical search allocated no
//! caller region and interned no canonical identity. Later uses then publish
//! exactly the evidence a fresh search would.
use super::{Evidence, Selection, Source, search::ChildProofs};
use crate::resolve::model::DefId;
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, Store, TypeId,
    infer::{self, Context, arguments::Argument, predicates::Predicate as Goal},
    lifetime_proof::Obligations,
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Answer {
    owner: DefId,
    arguments: GenericArgs,
}
#[derive(Clone, Copy)]
enum State {
    Observed(Answer),
    Verified(Answer),
    Rejected,
}
/// Search footprint required before an answer can be recorded.
pub(in crate::sema) struct Watch {
    extent: usize,
    pub untouched: bool,
}

#[derive(Default)]
pub(super) struct ChildWork {
    pub goals: usize,
    pub source_provider: usize,
    pub hits: usize,
}

#[derive(Default)]
struct Storage {
    roots: HashMap<Vec<u32>, State>,
    children: ChildProofs,
}

#[derive(Default)]
pub(in crate::sema) struct Memo {
    storage: Option<Box<Storage>>,
    /// The prepared key: trait item, effect, then every input type identity.
    key: Vec<u32>,
    pub(super) child_work: ChildWork,
}
impl Memo {
    pub(super) fn children(&self) -> Option<&ChildProofs> {
        self.storage.as_ref().map(|storage| &storage.children)
    }

    pub(super) fn child_proofs(&mut self) -> &mut ChildProofs {
        &mut self.storage.get_or_insert_with(Default::default).children
    }

    fn root(&self) -> Option<State> {
        self.storage
            .as_ref()?
            .roots
            .get(self.key.as_slice())
            .copied()
    }

    /// Prepare this goal's key. Ineligible goals keep the ordinary search.
    /// # Errors
    /// Rejects invalid inference identities.
    pub fn prepare(
        &mut self,
        store: &Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<bool, infer::Error> {
        self.key.clear();
        let (Some((item, _)), Some(effect)) = (goal.trait_arguments(), goal.constness()) else {
            return Ok(false);
        };
        self.key.extend([item.0, effect as u32]);
        for argument in goal.arguments() {
            let Argument::Type(term) = argument else {
                return Ok(false);
            };
            let Some(ty) = context.find_identity(store, term)? else {
                return Ok(false);
            };
            if !closed(store, ty) {
                return Ok(false);
            }
            self.key.push(ty.0);
        }
        Ok(true)
    }
    /// The goals this memo has observed, verified or rejected.
    #[cfg(test)]
    pub(in crate::sema) fn entries(&self) -> usize {
        self.storage
            .as_ref()
            .map_or(0, |storage| storage.roots.len())
    }
    /// A verified answer for the prepared key, as fresh evidence at `first`.
    pub fn reuse(&self, first: u32) -> Option<Selection> {
        let State::Verified(answer) = self.root()? else {
            return None;
        };
        Some(Selection::Proven(Evidence {
            completion: super::Completion::default(),
            source: Source::Implementation {
                owner: answer.owner,
                arguments: answer.arguments,
            },
            proof: Obligations::new(first),
            sized: Vec::new(),
        }))
    }
    /// Whether the upcoming search for the prepared key should be observed.
    pub fn watch(&self, store: &Store) -> Option<Watch> {
        if matches!(self.root(), Some(State::Rejected | State::Verified(_))) {
            return None;
        }
        Some(Watch {
            extent: store.interned_extent(),
            untouched: false,
        })
    }
    /// Record an observed search result for the prepared key.
    pub fn record(&mut self, store: &Store, watch: &Watch, first: u32, result: &Selection) {
        let answer = match result {
            Selection::Proven(evidence) if watch.untouched => trivial(evidence, first),
            _ => None,
        };
        let interned = store.interned_extent() != watch.extent;
        let next = match (self.root(), answer) {
            (None, Some(answer)) => State::Observed(answer),
            (Some(State::Observed(prior)), Some(answer)) if prior == answer && !interned => {
                State::Verified(answer)
            }
            (Some(State::Verified(answer)), _) => State::Verified(answer),
            // Unproved, nontrivial or disagreeing results keep the ordinary search.
            _ => State::Rejected,
        };
        let roots = &mut self.storage.get_or_insert_with(Default::default).roots;
        if let Some(state) = roots.get_mut(self.key.as_slice()) {
            *state = next;
        } else {
            roots.insert(self.key.clone(), next);
        }
    }
}

fn closed(store: &Store, ty: TypeId) -> bool {
    store.contains_parameters(ty) == Some(false)
        && store.contains_projections(ty) == Some(false)
        && store.contains_regions(ty) == Some(false)
        && store.contains_opaques(ty) == Some(false)
}

fn trivial(evidence: &Evidence, first: u32) -> Option<Answer> {
    let proof = &evidence.proof;
    let Source::Implementation { owner, arguments } = evidence.source else {
        return None;
    };
    (!evidence.completion.provisional()
        && evidence.sized.is_empty()
        && proof.regions == (first..first)
        && proof.equalities.is_empty()
        && proof.outlives.is_empty()
        && proof.types.is_empty()
        && !proof.has_quantified())
    .then_some(Answer { owner, arguments })
}
