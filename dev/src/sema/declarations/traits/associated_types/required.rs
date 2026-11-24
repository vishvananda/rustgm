//! Required family bounds intersect member contributions and propagate only along uses.
mod sites;
mod uses;
use super::{Checker, DefId, Error};
use crate::resolve::body_origin::DeclarationOrigin;
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    RegionId, TypeId, outlives::Queries, projection::Application, region_graph::VerificationId,
    relations::Constraints,
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Condition {
    Type(TypeId, RegionId),
    Region(RegionId, RegionId),
}
impl Condition {
    fn key(self) -> (u8, u32, u32) {
        match self {
            Self::Type(a, b) => (0, a.0, b.0),
            Self::Region(a, b) => (1, a.0, b.0),
        }
    }
    fn test(
        self,
        proof: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
    ) -> Result<VerificationId, String> {
        match self {
            Self::Type(ty, shorter) => proof.test_type(queries, ty, shorter),
            Self::Region(a, b) => proof.test_region(a, b),
        }
        .map_err(|e| e.to_string())
    }
    fn assume(self, proof: &mut Constraints<'_>, queries: &mut Queries<'_>) -> Result<(), String> {
        match self {
            Self::Type(ty, shorter) => proof.assume_type_outlives(queries, ty, shorter),
            Self::Region(a, b) => proof.assume(a, b).map_err(|e| e.to_string()),
        }
    }
}
struct Family {
    owner: DefId,
    origin: DeclarationOrigin,
    formal: Application,
    users: usize,
    votes: HashMap<Condition, usize>,
    required: HashSet<Condition>,
}
struct Use {
    target: DefId,
    types: Vec<(TypeId, TypeId)>,
    regions: Vec<(RegionId, RegionId)>,
    contributed: HashSet<Condition>,
}
struct Site {
    owner: DefId,
    origin: DeclarationOrigin,
    inputs: Vec<TypeId>,
    uses: Vec<Use>,
}
#[derive(Default)]
struct Analysis {
    families: HashMap<DefId, Family>,
    sites: Vec<Site>,
    dependents: HashMap<DefId, usize>,
    queue: VecDeque<usize>,
    queued: HashSet<usize>,
    steps: usize,
}
fn site_error(
    checker: &Checker<'_>,
    origin: DeclarationOrigin,
    message: impl Into<String>,
) -> Error {
    match origin {
        DeclarationOrigin::Source(source) => checker.error(source, message),
        DeclarationOrigin::Imported(_) => Error::unlocated(message),
    }
}
fn declared_environment<'a>(
    checker: &'a Checker<'_>,
    owner: DefId,
    origin: DeclarationOrigin,
    queries: &mut Queries<'_>,
) -> Result<Constraints<'a>, String> {
    let mut proof = checker.trait_region_declared_environment(owner, queries)?;
    if matches!(origin, DeclarationOrigin::Imported(_)) {
        let generics = &checker.facts.generics;
        for scope in generics.scopes(owner) {
            let inputs = crate::sema::declaration_inputs::declared_own(
                generics,
                checker.facts.crates.as_deref(),
                scope,
            )?;
            proof.assume_outlives_clauses(queries, inputs.iter().map(|input| input.value))?;
        }
    }
    Ok(proof)
}
fn charge(steps: &mut usize, count: usize) -> Result<(), &'static str> {
    *steps = steps.saturating_add(count);
    if *steps > 1_000_000 {
        return Err("required family bounds work budget exceeded");
    }
    Ok(())
}
fn sorted(conditions: &HashSet<Condition>) -> Vec<Condition> {
    let mut result: Vec<_> = conditions.iter().copied().collect();
    result.sort_unstable_by_key(|c| c.key());
    result
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_required_family_bounds(
        &mut self,
    ) -> Result<usize, Error> {
        let mut analysis = Analysis::default();
        analysis.collect(self)?;
        while let Some(index) = analysis.queue.pop_front() {
            analysis.queued.remove(&index);
            let origin = analysis.sites[index].origin;
            let answers = analysis
                .evaluate(self, index)
                .map_err(|e| site_error(self, origin, e))?;
            analysis
                .publish(index, answers)
                .map_err(|e| site_error(self, origin, e))?;
        }
        let mut families: Vec<_> = analysis.families.keys().copied().collect();
        families.sort_unstable_by_key(|id| id.0);
        for id in families {
            let family = &analysis.families[&id];
            if family.required.is_empty() {
                continue;
            }
            let mut queries = Queries::new(
                &self.facts.types,
                &self.facts.lifetimes.nominal_requirements,
                &self.facts.lifetimes.callables,
            );
            let mut proof = declared_environment(self, id, family.origin, &mut queries)
                .map_err(|e| site_error(self, family.origin, e))?;
            let mut tests = Vec::new();
            for condition in sorted(&family.required) {
                tests.push(
                    condition
                        .test(&mut proof, &mut queries)
                        .map_err(|e| site_error(self, family.origin, e))?,
                );
            }
            let answers = proof
                .test_answers(&tests)
                .map_err(|e| site_error(self, family.origin, e.to_string()))?;
            charge(&mut analysis.steps, answers.steps)
                .map_err(|e| site_error(self, family.origin, e))?;
            if answers.values.contains(&false) {
                return Err(site_error(
                    self,
                    family.origin,
                    "associated type is missing required where clauses",
                ));
            }
        }
        Ok(analysis.steps)
    }
}

impl Analysis {
    fn evaluate(
        &mut self,
        checker: &Checker<'_>,
        index: usize,
    ) -> Result<Vec<(usize, Condition)>, String> {
        let site = &self.sites[index];
        charge(&mut self.steps, 1)?;
        let mut queries = Queries::new(
            &checker.facts.types,
            &checker.facts.lifetimes.nominal_requirements,
            &checker.facts.lifetimes.callables,
        );
        let mut proof = declared_environment(checker, site.owner, site.origin, &mut queries)?;
        for &ty in &site.inputs {
            proof.assume_well_formed(&mut queries, ty)?;
        }
        if let Some(family) = self.families.get(&site.owner) {
            for condition in sorted(&family.required) {
                condition.assume(&mut proof, &mut queries)?;
            }
        }
        let mut tests = Vec::new();
        let mut mapped = Vec::new();
        for (use_index, use_) in site.uses.iter().enumerate() {
            for &(region, formal) in &use_.regions {
                if region == RegionId::STATIC {
                    continue;
                }
                for &(ty, parameter) in &use_.types {
                    charge(&mut self.steps, 1)?;
                    let condition = Condition::Type(parameter, formal);
                    if !use_.contributed.contains(&condition) {
                        tests.push(
                            proof
                                .test_type(&mut queries, ty, region)
                                .map_err(|e| e.to_string())?,
                        );
                        mapped.push((use_index, condition));
                    }
                }
                for &(shorter, target) in &use_.regions {
                    charge(&mut self.steps, 1)?;
                    let condition = Condition::Region(formal, target);
                    if region != shorter
                        && shorter != RegionId::STATIC
                        && !use_.contributed.contains(&condition)
                    {
                        tests.push(
                            proof
                                .test_region(region, shorter)
                                .map_err(|e| e.to_string())?,
                        );
                        mapped.push((use_index, condition));
                    }
                }
            }
        }
        let answers = proof.test_answers(&tests).map_err(|e| e.to_string())?;
        charge(&mut self.steps, answers.steps)?;
        Ok(mapped
            .into_iter()
            .zip(answers.values)
            .filter_map(|(key, proved)| proved.then_some(key))
            .collect())
    }

    fn publish(
        &mut self,
        index: usize,
        answers: Vec<(usize, Condition)>,
    ) -> Result<(), &'static str> {
        charge(&mut self.steps, answers.len())?;
        for (use_index, condition) in answers {
            let use_ = &mut self.sites[index].uses[use_index];
            if !use_.contributed.insert(condition) {
                continue;
            }
            let family = self
                .families
                .get_mut(&use_.target)
                .ok_or("missing required-bound family")?;
            let votes = family.votes.entry(condition).or_default();
            *votes += 1;
            if *votes == family.users
                && family.required.insert(condition)
                && let Some(&dependent) = self.dependents.get(&use_.target)
                && self.queued.insert(dependent)
            {
                self.queue.push_back(dependent);
            }
        }
        Ok(())
    }
}
