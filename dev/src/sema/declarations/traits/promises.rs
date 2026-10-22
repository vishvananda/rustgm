//! Close builtin Self promises before projection identities acquire cached properties.
use super::super::generics::bounds::Written;
use super::super::{Checker, DefId, Error, NodeKey};
use crate::support::hash::HashMap;
pub(super) use crate::types::projection::Builtins;
use crate::{
    sema::associated::OwnerKind,
    types::{
        ItemId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

impl Checker<'_> {
    /// The decoded bounds already retain resolved declaration identities. Their
    /// builtin Self promises are independent of application arguments, but remain
    /// conditional on proving the complete application through ordinary semantics.
    pub(super) fn inherited_builtins(&self) -> Result<HashMap<DefId, Builtins>, Error> {
        let mut promises = HashMap::default();
        let mut dependents: HashMap<DefId, Vec<DefId>> = HashMap::default();
        let mut work = Vec::new();
        let mut steps = 0_usize;
        for owner in self.facts.associated.owners() {
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            steps =
                steps.saturating_add(1 + self.bound_syntax.get(&definition).map_or(0, Vec::len));
            if steps > 1_000_000 {
                return Err(self.error(
                    owner.source,
                    "associated builtin bound work budget exceeded",
                ));
            }
            let mut result = Builtins::default();
            for (_, bound) in self.declared_self_bounds(definition, owner.self_definition) {
                match bound {
                    Written::Trait(bound) => {
                        result.include(Builtins::bound(bound));
                    }
                    Written::User {
                        definition: parent, ..
                    } => {
                        dependents.entry(parent).or_default().push(definition);
                    }
                    Written::Outlives(_) => {}
                }
            }
            promises.insert(definition, result);
            if result != Builtins::default() {
                work.push(definition);
            }
        }
        self.close_builtin_promises(promises, &dependents, work, steps)
    }

    fn close_builtin_promises(
        &self,
        mut promises: HashMap<DefId, Builtins>,
        dependents: &HashMap<DefId, Vec<DefId>>,
        mut work: Vec<DefId>,
        mut steps: usize,
    ) -> Result<HashMap<DefId, Builtins>, Error> {
        // Each builtin permission changes at most once per declaration. Cycles need
        // no special assumptions here; the ordinary trait gate still rejects them.
        while let Some(parent) = work.pop() {
            let inherited = promises[&parent];
            for &child in dependents.get(&parent).into_iter().flatten() {
                steps = steps.saturating_add(1);
                if steps > 1_000_000 {
                    let key = self.report.resolution.definitions[child.0 as usize]
                        .node
                        .or_else(|| self.facts.associated.language_origin(child))
                        .ok_or_else(|| {
                            Error::at(crate::resolve::unit::FileId(0), 0, "missing trait source")
                        })?;
                    return Err(self.error(key, "associated builtin bound work budget exceeded"));
                }
                if promises.entry(child).or_default().include(inherited) {
                    work.push(child);
                }
            }
        }
        Ok(promises)
    }

    /// Complete conditional properties once, before caller environments freeze.
    /// Canonical schemas and their already interned applications remain unchanged.
    pub(in crate::sema::declarations) fn complete_projection_promises(
        &mut self,
    ) -> Result<(), Error> {
        if self.associated_type_bounds.is_empty() {
            return Ok(());
        }
        let inherited = self.typed_builtin_promises()?;
        let mut promises = HashMap::default();
        for (&member, bounds) in &self.associated_type_bounds {
            let key = self.report.resolution.definitions[member.0 as usize]
                .node
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing result declaration source",
                    )
                })?;
            let schema = self
                .facts
                .types
                .projection_schema(ItemId(member.0))
                .ok_or_else(|| self.error(key, "missing reserved result schema"))?;
            let original = Builtins {
                sized: schema.sized,
                copies: schema.copies,
                autos: 0,
            };
            let mut result = original;
            for syntax in &bounds.syntax {
                if let Some((_, owner)) = syntax.trait_path()
                    && let Some(&properties) = inherited.get(&owner)
                {
                    result.include(properties);
                }
            }
            for predicate in self
                .facts
                .generics
                .traits
                .as_ref()
                .ok_or_else(|| self.error(key, "missing completed trait headers"))?
                .associated_bounds(ItemId(member.0))
            {
                let bound = self.promise_bound(key, *predicate)?;
                let properties = if let Bound::Trait(id) = bound {
                    let application = self
                        .facts
                        .types
                        .trait_application(id)
                        .ok_or_else(|| self.error(key, "missing result bound application"))?;
                    inherited
                        .get(&DefId(application.item.0))
                        .copied()
                        .unwrap_or_default()
                } else {
                    Builtins::bound(bound)
                };
                result.include(properties);
            }
            if result != original {
                promises.insert(ItemId(member.0), result);
            }
        }
        if !promises.is_empty() {
            self.facts.generics.projection_promises = Some(std::sync::Arc::new(promises));
        }
        Ok(())
    }

    fn typed_builtin_promises(&self) -> Result<HashMap<DefId, Builtins>, Error> {
        let Some(contracts) = &self.facts.generics.traits else {
            return Ok(HashMap::default());
        };
        let mut promises = HashMap::default();
        let mut dependents: HashMap<DefId, Vec<DefId>> = HashMap::default();
        let mut work = Vec::new();
        let mut steps = 0_usize;
        for (item, header) in contracts.headers() {
            let definition = DefId(item.0);
            let key = self.report.resolution.definitions[item.0 as usize]
                .node
                .or_else(|| self.facts.associated.language_origin(definition))
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing trait declaration source",
                    )
                })?;
            steps = steps.saturating_add(1 + header.predicates.len());
            if steps > 1_000_000 {
                return Err(self.error(key, "typed builtin promise work budget exceeded"));
            }
            let mut result = Builtins::default();
            for predicate in &header.predicates {
                if !header
                    .implies_self(&self.facts.types, predicate)
                    .map_err(|e| self.error(key, e))?
                {
                    continue;
                }
                let bound = self.promise_bound(key, predicate.value)?;
                if let Bound::Trait(id) = bound {
                    let parent = self
                        .facts
                        .types
                        .trait_application(id)
                        .ok_or_else(|| self.error(key, "missing typed Self application"))?;
                    dependents
                        .entry(DefId(parent.item.0))
                        .or_default()
                        .push(definition);
                } else {
                    result.include(Builtins::bound(bound));
                }
            }
            promises.insert(definition, result);
            if result != Builtins::default() {
                work.push(definition);
            }
        }
        self.close_builtin_promises(promises, &dependents, work, steps)
    }

    /// Classify uniform builtin properties only. The complete clause remains
    /// required for implementation validation and never becomes unbound evidence.
    fn promise_bound(&self, key: NodeKey, clause: Clause) -> Result<Bound, Error> {
        match clause {
            Clause::Predicate(predicate) => Ok(predicate.bound),
            Clause::ForAll(id) => self
                .facts
                .types
                .quantified(id)
                .map(|value| value.predicate.bound)
                .ok_or_else(|| self.error(key, "invalid builtin promise clause")),
        }
    }
}
