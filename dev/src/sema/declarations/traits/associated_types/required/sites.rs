//! Visit relevant trait members once and retain each member's separate contribution.
use super::{Analysis, Checker, DefId, Error, Family, Site, charge, uses::Collector};
use crate::{
    resolve::model::DefKind,
    types::{GenericArgs, Kind, parameters::clauses::Clause},
};
use std::collections::{HashMap, HashSet};

impl Analysis {
    pub(super) fn collect(&mut self, checker: &mut Checker<'_>) -> Result<(), Error> {
        if checker.facts.associated.families.is_empty() {
            return Ok(());
        }
        let owners = self.register_families(checker)?;
        if owners.is_empty() {
            return Ok(());
        }
        let members = checker.facts.associated.members().to_vec();
        for member in members {
            let id = member.definition;
            let Some(owner) = checker
                .facts
                .associated
                .trait_owner(id)
                .filter(|o| owners.contains(o))
            else {
                continue;
            };
            let definition = &checker.report.resolution.definitions[id.0 as usize];
            let source = definition.node.ok_or_else(|| {
                checker.error(
                    checker.facts.associated.owners()[member.owner.0 as usize].source,
                    "missing required-bound member source",
                )
            })?;
            self.collect_site(checker, id, owner, source)?;
        }
        Ok(())
    }

    fn register_families(&mut self, checker: &Checker<'_>) -> Result<HashSet<DefId>, Error> {
        let mut owners = HashSet::new();
        for member in checker.facts.associated.members() {
            let id = member.definition;
            let Some(owner) = checker.facts.associated.trait_owner(id) else {
                continue;
            };
            let Some(contract) = checker.facts.associated.type_contract(id) else {
                continue;
            };
            let Some(Kind::Projection(formal)) = checker.facts.types.kind(contract.projection)
            else {
                return Err(checker.error(
                    checker.facts.associated.owners()[member.owner.0 as usize].source,
                    "missing required-bound family projection",
                ));
            };
            if formal.arguments == GenericArgs::EMPTY {
                continue;
            }
            let source = checker.report.resolution.definitions[id.0 as usize]
                .node
                .ok_or_else(|| {
                    checker.error(
                        checker.facts.associated.owners()[member.owner.0 as usize].source,
                        "missing required-bound family source",
                    )
                })?;
            charge(&mut self.steps, 1).map_err(|e| checker.error(source, e))?;
            owners.insert(owner);
            self.families.insert(
                id,
                Family {
                    owner,
                    source,
                    formal,
                    users: 0,
                    votes: HashMap::new(),
                    required: HashSet::new(),
                },
            );
        }
        Ok(owners)
    }

    fn collect_site(
        &mut self,
        checker: &mut Checker<'_>,
        id: DefId,
        owner: DefId,
        source: super::NodeKey,
    ) -> Result<(), Error> {
        let definition = &checker.report.resolution.definitions[id.0 as usize];
        charge(&mut self.steps, 1).map_err(|e| checker.error(source, e))?;
        let mut collector = Collector::default();
        let mut inputs = Vec::new();
        match definition.kind {
            DefKind::Function => {
                let ty = checker
                    .facts
                    .unreduced
                    .definition(id)
                    .or(checker.facts.definitions[id.0 as usize])
                    .ok_or_else(|| checker.error(source, "missing required-bound method type"))?;
                let Some(Kind::FunctionItem { signature, .. }) = checker.facts.types.kind(ty)
                else {
                    return Err(checker.error(source, "invalid required-bound method type"));
                };
                let Some(Kind::Function {
                    parameters, result, ..
                }) = checker.facts.types.kind(signature)
                else {
                    return Err(checker.error(source, "invalid required-bound method signature"));
                };
                inputs.extend_from_slice(
                    checker
                        .facts
                        .types
                        .list(parameters)
                        .ok_or_else(|| checker.error(source, "missing method inputs"))?,
                );
                for ty in inputs.iter().copied().chain(std::iter::once(result)) {
                    collector
                        .collect(&checker.facts.types, ty, 0)
                        .map_err(|e| checker.error(source, e))?;
                }
            }
            DefKind::TypeAlias => {
                for clause in result_bounds(checker, id).map_err(|e| checker.error(source, e))? {
                    collector
                        .clause(&checker.facts.types, clause)
                        .map_err(|e| checker.error(source, e))?;
                }
            }
            _ => return Ok(()),
        }
        let uses = collector
            .finish(&mut checker.facts.types, &self.families, owner, id)
            .map_err(|e| checker.error(source, e))?;
        charge(&mut self.steps, collector.steps).map_err(|e| checker.error(source, e))?;
        if uses.is_empty() {
            return Ok(());
        }
        for &ty in &inputs {
            checker
                .facts
                .lifetimes
                .nominal_requirements
                .prepare(&mut checker.facts.types, ty)
                .map_err(|e| checker.error(source, e))?;
        }
        for use_ in &uses {
            self.families
                .get_mut(&use_.target)
                .ok_or_else(|| checker.error(source, "missing family use"))?
                .users += 1;
        }
        let index = self.sites.len();
        self.dependents.insert(id, index);
        self.sites.push(Site {
            owner: id,
            source,
            inputs,
            uses,
        });
        self.queued.insert(index);
        self.queue.push_back(index);
        Ok(())
    }
}

fn result_bounds(checker: &Checker<'_>, id: DefId) -> Result<Vec<Clause>, &'static str> {
    let Some(contract) = checker.facts.associated.type_contract(id) else {
        return Ok(Vec::new());
    };
    let inputs = checker
        .facts
        .associated
        .families
        .get(&id)
        .and_then(|f| f.requirements.as_ref())
        .map_or(0, Vec::len);
    let predicates = checker.facts.generics.declared_predicates(id);
    let results = predicates
        .get(inputs..)
        .ok_or("invalid family result boundary")?;
    let inherited = contract
        .bounds
        .len()
        .checked_sub(results.len())
        .ok_or("invalid family result contract")?;
    let mut bounds = contract.bounds[..inherited].to_vec();
    bounds.extend(results.iter().map(|p| {
        checker
            .facts
            .unreduced
            .predicate(p.source)
            .unwrap_or(p.value)
    }));
    Ok(bounds)
}
