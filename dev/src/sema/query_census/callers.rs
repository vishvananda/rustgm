//! A caller's premises as content, so that the census can count the keys a
//! cache would see if callers with equal premises shared one identity. Free
//! regions, openings and feature grants keep the caller's own identity.
use super::{CONSTANT, Canonical, Environment, FxHasher, PARAMETER, State, pair};
use crate::types::{
    Store,
    parameters::{
        clauses::Clause,
        environment::CensusPremise as Premise,
        predicates::{Bound, Predicate},
        premise_index::ClauseIndex,
    },
};
use std::hash::{Hash, Hasher};

const DECLARED: u64 = PARAMETER + 2;
const SUBJECT: u64 = DECLARED + 1;
const SIZED: u64 = DECLARED + 2;
const TRAIT: u64 = DECLARED + 3;
const QUANTIFIED: u64 = DECLARED + 4;
const FEATURE: u64 = DECLARED + 5;
const RESULT: u64 = DECLARED + 6;
const OUTLIVES: u64 = DECLARED + 7;
const BUILTIN: u64 = DECLARED + 8;

impl State {
    /// The premises hashed exactly and with parameters renamed, in no order.
    pub(super) fn premises(
        &mut self,
        types: &Store,
        clauses: Option<&ClauseIndex>,
        environment: Option<&Environment>,
    ) -> Option<[[u64; 2]; 2]> {
        let mut premises = Vec::new();
        if let Some(environment) = environment
            && !environment.census_premises(&mut |premise| premises.push(premise))
        {
            return None;
        }
        if let Some(index) = clauses {
            if index.grants_features() {
                return None;
            }
            for (layer, opening) in index.layers() {
                if opening.is_some() {
                    return None;
                }
                let quantified = layer.quantified.iter().flat_map(|index| &index.seen);
                premises.extend(
                    layer
                        .seen
                        .iter()
                        .map(|&predicate| Clause::Predicate(predicate))
                        .chain(quantified.map(|&id| Clause::ForAll(id)))
                        .map(Premise::Clause),
                );
            }
        }
        let mut result = [[0; 2]; 2];
        for (slot, rename) in result.iter_mut().zip([false, true]) {
            self.rename = rename;
            let mut items = Vec::with_capacity(premises.len());
            for &premise in &premises {
                self.key.clear();
                self.regions.clear();
                if !self.premise(types, premise) || !self.regions.is_empty() {
                    self.rename = false;
                    return None;
                }
                items.push(pair(&self.key));
            }
            self.rename = false;
            items.sort_unstable();
            items.dedup();
            *slot = pair(items.as_flattened());
        }
        Some(result)
    }

    fn premise(&mut self, types: &Store, premise: Premise) -> bool {
        match premise {
            Premise::Parameter(id, abilities) => {
                self.key.extend([
                    DECLARED,
                    u64::from(abilities.copy),
                    u64::from(abilities.clone_),
                ]);
                self.parameter(types, id);
            }
            Premise::Subject(ty, abilities) => {
                self.key.extend([
                    SUBJECT,
                    u64::from(abilities.copy),
                    u64::from(abilities.clone_),
                ]);
                self.ty(types, ty);
            }
            Premise::Sized(ty) => {
                self.key.push(SIZED);
                self.ty(types, ty);
            }
            Premise::Clause(Clause::Predicate(predicate)) => {
                return self.predicate(types, predicate);
            }
            Premise::Clause(Clause::ForAll(id)) => {
                let Some(quantified) = types.quantified(id) else {
                    return false;
                };
                self.key
                    .extend([QUANTIFIED, u64::from(quantified.bound_regions)]);
                return self.predicate(types, quantified.predicate);
            }
            Premise::Clause(Clause::UnstableFeature(id)) => {
                let mut hasher = FxHasher::default();
                id.hash(&mut hasher);
                self.key.extend([FEATURE, hasher.finish()]);
            }
        }
        true
    }

    fn predicate(&mut self, types: &Store, predicate: Predicate) -> bool {
        self.ty(types, predicate.ty);
        match predicate.bound {
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                let Some(application) = types.trait_application(id) else {
                    return false;
                };
                let effect = match predicate.bound {
                    Bound::Trait(_) => 0,
                    Bound::ConstTrait(_) => 1,
                    _ => 2,
                };
                self.key
                    .extend([TRAIT, effect, u64::from(application.item.0)]);
                for argument in types.arguments(application.arguments).into_iter().flatten() {
                    match argument {
                        Canonical::Lifetime(region) => self.region(region),
                        Canonical::Const(constant) => {
                            self.key.extend([CONSTANT, u64::from(constant.0)]);
                        }
                        Canonical::Type(member) => self.ty(types, member),
                    }
                }
            }
            Bound::ProjectionEqual(result) => {
                self.key.push(RESULT);
                self.ty(types, result);
            }
            Bound::Outlives(region) => {
                self.key.push(OUTLIVES);
                self.region(region);
            }
            bound @ (Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_)) => {
                let mut hasher = FxHasher::default();
                bound.hash(&mut hasher);
                self.key.extend([BUILTIN, hasher.finish()]);
            }
        }
        true
    }
}
