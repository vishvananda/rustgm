//! Header member routes depend on resolved trait identities, never unfinished proof contracts.
use super::{Checker, DefId, Error, NodeKey, Predicate, Seed, Work, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{model::DefKind, names::NameId},
    sema::associated::OwnerKind,
};

#[derive(Default)]
pub(super) struct Headers {
    order: Vec<DefId>,
    parents: HashMap<DefId, Vec<Seed>>,
    dependents: HashMap<DefId, Vec<DefId>>,
    declarations: HashMap<NameId, Vec<DefId>>,
    pub(super) reachable: HashMap<NameId, HashSet<DefId>>,
    queries: HashMap<NodeKey, Vec<Predicate>>,
    steps: usize,
}
impl Headers {
    fn check_cycles(&mut self) -> Result<(), (DefId, &'static str)> {
        let mut states = HashMap::default();
        for index in 0..self.order.len() {
            let root = self.order[index];
            let mut work = vec![(root, false)];
            while let Some((owner, finish)) = work.pop() {
                self.charge(1).map_err(|e| (owner, e))?;
                if finish {
                    states.insert(owner, true);
                    continue;
                }
                match states.get(&owner) {
                    Some(true) => continue,
                    Some(false) => return Err((owner, "cycle in supertrait requirements")),
                    None => {}
                }
                states.insert(owner, false);
                work.push((owner, true));
                for seed in self.parents.get(&owner).into_iter().flatten().rev() {
                    work.push((seed.definition, false));
                }
            }
        }
        Ok(())
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(cost);
        if self.steps > 1_000_000 {
            return Err("trait header naming work budget exceeded");
        }
        Ok(())
    }
    pub(super) fn reach(&mut self, name: NameId) -> Result<(), &'static str> {
        if self.reachable.contains_key(&name) {
            return Ok(());
        }
        let mut reached = HashSet::default();
        let mut work = self.declarations.get(&name).cloned().unwrap_or_default();
        while let Some(owner) = work.pop() {
            self.charge(1)?;
            if reached.insert(owner) {
                work.extend(self.dependents.get(&owner).into_iter().flatten().copied());
            }
        }
        self.reachable.insert(name, reached);
        Ok(())
    }
    fn inputs(&mut self, name: NameId, roots: &[DefId]) -> Result<Vec<Seed>, &'static str> {
        self.reach(name)?;
        let mut work = roots.to_vec();
        let mut seen = HashSet::default();
        let mut inputs = Vec::new();
        while let Some(owner) = work.pop() {
            self.charge(1)?;
            if !seen.insert(owner) {
                continue;
            }
            for &seed in self.parents.get(&owner).into_iter().flatten() {
                if self.reachable[&name].contains(&seed.definition) {
                    inputs.push(seed);
                    work.push(seed.definition);
                }
            }
        }
        self.charge(inputs.len())?;
        Ok(inputs)
    }
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn projection_scope_owner(&self, owner: DefId) -> DefId {
        if self.report.resolution.definitions[owner.0 as usize].kind == DefKind::SelfType {
            self.facts.associated.trait_owner(owner).unwrap_or(owner)
        } else {
            owner
        }
    }

    pub(in crate::sema::declarations) fn begin_header_names(&mut self) -> Result<(), Error> {
        if self.associated_type_bounds.is_empty() {
            return Ok(());
        }
        let mut headers = Headers::default();
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let seeds = self
                .projections
                .lookup
                .bounds
                .get(&(definition, owner.self_definition))
                .cloned()
                .unwrap_or_default();
            for seed in &seeds {
                headers
                    .dependents
                    .entry(seed.definition)
                    .or_default()
                    .push(definition);
            }
            headers.parents.insert(definition, seeds);
            headers.order.push(definition);
        }
        for &member in self.associated_type_bounds.keys() {
            if let Some(owner) = self.facts.associated.trait_owner(member) {
                let name = self.report.resolution.definitions[member.0 as usize].name;
                headers.declarations.entry(name).or_default().push(owner);
            }
        }
        headers.check_cycles().map_err(|(owner, message)| {
            self.report.resolution.definitions[owner.0 as usize]
                .node
                .map_or_else(
                    || Error::at(crate::resolve::unit::FileId(0), 0, message),
                    |key| self.error(key, message),
                )
        })?;
        self.projections.lookup.headers = Some(Box::new(headers));
        Ok(())
    }

    pub(in crate::sema::declarations) fn header_names_active(&self) -> bool {
        self.projections.lookup.headers.is_some()
    }

    pub(in crate::sema::declarations) fn end_header_names(&mut self) {
        self.projections.lookup.headers = None;
    }

    pub(in crate::sema::declarations::projections) fn schedule_header_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        name: NameId,
        predicates: Vec<Predicate>,
    ) -> Result<(), Error> {
        let mut roots = Vec::with_capacity(predicates.len());
        for predicate in &predicates {
            let Some((application, _)) = predicate.bound.trait_parts() else {
                continue;
            };
            let application = self
                .facts
                .types
                .trait_application(application)
                .ok_or_else(|| self.error(key, "missing header trait application"))?;
            roots.push(DefId(application.item.0));
        }
        let headers = self
            .projections
            .lookup
            .headers
            .as_mut()
            .ok_or_else(|| Error::at(key.file, 0, "missing trait header names"))?;
        let seeds = headers
            .inputs(name, &roots)
            .map_err(|e| Error::at(key.file, 0, e))?;
        if headers.queries.insert(key, predicates).is_some() {
            return Err(self.error(key, "cycle in trait header member inputs"));
        }
        self.work.push(Work::HeaderProjection(key, context));
        for seed in seeds.into_iter().rev() {
            self.work.push(Work::ProjectionBound(seed, false));
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn complete_header_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let predicates = self
            .projections
            .lookup
            .headers
            .as_mut()
            .and_then(|headers| headers.queries.remove(&key))
            .ok_or_else(|| self.error(key, "missing trait header member query"))?;
        self.finish_implicit_projection(key, context, predicates)
    }

    pub(in crate::sema::declarations::projections) fn header_projection_parents(
        &mut self,
        key: NodeKey,
        name: NameId,
        predicate: Predicate,
    ) -> Result<Vec<Predicate>, Error> {
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(Vec::new());
        };
        let application = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(key, "missing header projection application"))?;
        let headers = self
            .projections
            .lookup
            .headers
            .as_ref()
            .ok_or_else(|| self.error(key, "missing trait header graph"))?;
        let reached = headers
            .reachable
            .get(&name)
            .ok_or_else(|| self.error(key, "missing header name routes"))?;
        let mut templates = Vec::new();
        for seed in headers
            .parents
            .get(&DefId(application.item.0))
            .into_iter()
            .flatten()
        {
            if reached.contains(&seed.definition) {
                templates.extend(
                    self.projections
                        .lookup
                        .typed
                        .get(&seed.path)
                        .ok_or_else(|| self.error(key, "unfinished header parent inputs"))?
                        .clauses(),
                );
            }
        }
        if templates.is_empty() {
            return Ok(Vec::new());
        }
        let mut fold = crate::sema::traits::results::substitution(&self.facts, predicate)
            .map_err(|e| self.error(key, e))?;
        let mut result = Vec::with_capacity(templates.len());
        for template in templates {
            let template = self.projection_naming_predicate(key, template, None)?;
            result.push(
                fold.predicate(&mut self.facts.types, template)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        self.projections
            .lookup
            .headers
            .as_mut()
            .ok_or_else(|| Error::at(key.file, 0, "missing header naming workspace"))?
            .charge(fold.steps)
            .map_err(|e| Error::at(key.file, 0, e))?;
        Ok(result)
    }
}
