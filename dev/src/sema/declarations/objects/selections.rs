//! Named object equalities preserve quantifiers introduced by supertraits.
use super::{Checker, Error, NodeKey, Pending, Predicate, Type, graph::Graph};
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        model::{DefId, Namespace},
        names::NameId,
    },
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::{
            clauses::{Clause, Comparison, Quantified},
            predicates::Bound,
        },
        projection, substitute,
    },
};
use std::sync::Arc;

pub(super) struct Members {
    names: HashMap<NameId, Candidate>,
}
struct Candidate {
    value: Quantified,
    key: Comparison,
    direct: bool,
    ambiguous: bool,
}
impl Members {
    fn select(&self, name: NameId) -> Result<Quantified, &'static str> {
        let candidate = self
            .names
            .get(&name)
            .ok_or("trait bound has no such associated type")?;
        if candidate.ambiguous {
            return Err("ambiguous inherited associated type binding");
        }
        Ok(candidate.value)
    }
}

impl Checker<'_> {
    pub(super) fn object_bindings(
        &mut self,
        key: NodeKey,
        pending: &Pending,
        predicate: Predicate,
        original: Predicate,
    ) -> Result<Vec<crate::sema::generics::Predicate>, Error> {
        let principal = pending
            .principal
            .as_ref()
            .ok_or_else(|| self.error(key, "missing principal object syntax"))?;
        if crate::sema::lifetimes::call_arguments(self.report, principal.path)?.is_some() {
            let mut output = Vec::new();
            self.call_output_binding(
                principal.path,
                predicate,
                original,
                principal.scope,
                &mut output,
            )?;
            return Ok(output);
        }
        let Some(list) = self.associated_bindings.bindings(principal.path) else {
            return Ok(Vec::new());
        };
        let root = self.object_trait_clause(key, principal.scope, predicate)?;
        let original_root = self.object_trait_clause(key, principal.scope, original)?;
        let members = self.object_member_index(key, root)?;
        let originals = self.object_member_index(key, original_root)?;
        let mut output = Vec::new();
        let mut shift = substitute::Bound::shift(1);
        for offset in list.start..list.end {
            let node = self.report.package.unit(key.file).configured.ast.edges[offset as usize];
            let source = NodeKey { node, ..key };
            let binding = self
                .report
                .resolution
                .associated_bindings
                .get(&source)
                .ok_or_else(|| self.error(source, "missing resolved object binding"))?;
            let name = binding.name;
            if binding.arguments.is_some() {
                return Err(self.error(source, "object cannot bind generic associated types"));
            }
            let mut selected = members.select(name).map_err(|e| self.error(source, e))?;
            let mut raw = originals.select(name).map_err(|e| self.error(source, e))?;
            if let Some(Type::Projection(application)) =
                self.facts.types.kind(selected.predicate.ty)
                && self
                    .facts
                    .associated
                    .families
                    .contains_key(&DefId(application.item.0))
            {
                return Err(self.error(source, "object cannot bind generic associated types"));
            }
            let value = NodeKey {
                node: self.associated_binding_value(source)?,
                ..key
            };
            let mut result = self.ty(value)?;
            let mut raw_result = self.facts.unreduced.node(value).unwrap_or(result);
            if principal.scope.is_none() {
                result = shift
                    .apply(&mut self.facts.types, result)
                    .map_err(|e| self.error(source, e))?;
                raw_result = shift
                    .apply(&mut self.facts.types, raw_result)
                    .map_err(|e| self.error(source, e))?;
            }
            selected.predicate.bound = Bound::ProjectionEqual(result);
            raw.predicate.bound = Bound::ProjectionEqual(raw_result);
            self.check_object_binding_lifetimes(source, raw)?;
            let original = self
                .facts
                .types
                .intern_quantified(raw)
                .map(Clause::ForAll)
                .map_err(|e| self.error(source, e))?;
            self.facts.unreduced.retain_predicate(source, original);
            let value = self
                .facts
                .types
                .intern_quantified(selected)
                .map(Clause::ForAll)
                .map_err(|e| self.error(source, e))?;
            output.push(crate::sema::generics::Predicate { source, value });
        }
        Ok(output)
    }

    fn object_member_index(
        &mut self,
        source: NodeKey,
        root: Clause,
    ) -> Result<Arc<Members>, Error> {
        if let Some(index) = self.object_syntax.members.get(&root) {
            return Ok(Arc::clone(index));
        }
        let mut graph = Graph::new(root);
        let mut direct = true;
        let mut names: HashMap<NameId, Candidate> = HashMap::default();
        while let Some(visit) = graph
            .next(&mut self.facts)
            .map_err(|e| self.error(source, e))?
        {
            let Bound::Trait(application) = visit.predicate.bound else {
                return Err(self.error(source, "object member lookup requires a trait"));
            };
            let owner = self
                .facts
                .types
                .trait_application(application)
                .ok_or_else(|| self.error(source, "invalid object member parent"))?
                .item;
            let owner = self
                .facts
                .associated
                .owner(DefId(owner.0))
                .ok_or_else(|| self.error(source, "missing object member owner"))?;
            let members = self
                .facts
                .associated
                .owner_members(owner)
                .ok_or_else(|| self.error(source, "missing object member inventory"))?
                .to_vec();
            self.object_syntax.member_steps = self
                .object_syntax
                .member_steps
                .saturating_add(members.len());
            if self.object_syntax.member_steps > 1_000_000 {
                return Err(self.error(source, "object member index work budget exceeded"));
            }
            for member in members {
                let definition = &self.report.resolution.definitions[member.0 as usize];
                if definition.namespace != Namespace::Type {
                    continue;
                }
                let name = definition.name;
                if names.get(&name).is_some_and(|candidate| candidate.direct) {
                    continue;
                }
                let projection = self.intern(
                    source,
                    Type::Projection(projection::Application {
                        item: ItemId(member.0),
                        subject: visit.predicate.ty,
                        trait_application: application,
                        arguments: GenericArgs::EMPTY,
                    }),
                )?;
                let value = Quantified {
                    bound_regions: visit.bound_regions,
                    predicate: Predicate {
                        ty: projection,
                        bound: Bound::Sized,
                    },
                };
                let id = self
                    .facts
                    .types
                    .intern_quantified(value)
                    .map_err(|e| self.error(source, e))?;
                let fixed = graph.fixed_slots().map_err(|e| self.error(source, e))?;
                let (key, _) = self
                    .facts
                    .types
                    .clause_comparison_preserving(id, fixed)
                    .map_err(|e| self.error(source, e))?;
                if let Some(previous) = names.get_mut(&name) {
                    previous.ambiguous |= previous.key != key;
                } else {
                    names.insert(
                        name,
                        Candidate {
                            value,
                            key,
                            direct,
                            ambiguous: false,
                        },
                    );
                }
            }
            direct = false;
        }
        let members = Arc::new(Members { names });
        self.object_syntax
            .members
            .insert(root, Arc::clone(&members));
        Ok(members)
    }

    pub(super) fn object_member_key(
        &mut self,
        source: NodeKey,
        member: ItemId,
        application: crate::types::traits::ApplicationId,
        subject: TypeId,
        bound_regions: u32,
        fixed: u32,
    ) -> Result<Comparison, Error> {
        let ty = self.intern(
            source,
            Type::Projection(projection::Application {
                item: member,
                subject,
                trait_application: application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let id = self
            .facts
            .types
            .intern_quantified(Quantified {
                bound_regions,
                predicate: Predicate {
                    ty,
                    bound: Bound::Sized,
                },
            })
            .map_err(|e| self.error(source, e))?;
        self.facts
            .types
            .clause_comparison_preserving(id, fixed)
            .map(|(key, _)| key)
            .map_err(|e| self.error(source, e))
    }

    fn check_object_binding_lifetimes(
        &self,
        source: NodeKey,
        value: Quantified,
    ) -> Result<(), Error> {
        let Some(Type::Projection(application)) = self.facts.types.kind(value.predicate.ty) else {
            return Err(self.error(source, "object equality requires a projected member"));
        };
        let Bound::ProjectionEqual(result) = value.predicate.bound else {
            return Err(self.error(source, "object equality requires a result"));
        };
        let mut inputs = crate::types::bound_uses::Uses::constraining();
        inputs
            .predicate(
                &self.facts.types,
                Predicate {
                    ty: application.subject,
                    bound: Bound::Trait(application.trait_application),
                },
            )
            .map_err(|e| self.error(source, e))?;
        let mut outputs = crate::types::bound_uses::Uses::default();
        outputs
            .collect(&self.facts.types, result)
            .map_err(|e| self.error(source, e))?;
        if !outputs.indices.is_subset(&inputs.indices) {
            return Err(self.error(
                source,
                "object associated result lifetime is unconstrained by its parent inputs",
            ));
        }
        Ok(())
    }
}
