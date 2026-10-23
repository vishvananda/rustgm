//! Preserve complete associated applications and one enclosing trait binder.
use super::graph::{Graph, quantified_value};
use super::{Bound, Checker, DefId, Error, NodeKey, Pending, Predicate, RegionId, Type, TypeId};
use crate::support::hash::HashSet;
use crate::types::{ItemId, arguments::Argument, dynamic, parameters::clauses::Clause, substitute};
use crate::types::{
    parameters::clauses::{Comparison, Quantified},
    traits::Application,
};

struct Equality {
    binding: dynamic::Binding,
    inputs: Vec<Argument>,
    result: TypeId,
    bound_regions: u32,
}
struct Inputs {
    root: Clause,
    subject: TypeId,
    supplied: HashSet<Comparison>,
    fixed: u32,
}

impl Checker<'_> {
    pub(super) fn complete_object_bounds(
        &mut self,
        key: NodeKey,
        pending: &Pending,
        predicate: Predicate,
        equalities: Vec<crate::sema::generics::Predicate>,
        mut bound_regions: u32,
        region: RegionId,
    ) -> Result<TypeId, Error> {
        let declared = pending
            .principal
            .as_ref()
            .ok_or_else(|| self.error(key, "missing principal object syntax"))?;
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(key, "missing principal object application"));
        };
        let principal = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(key, "invalid principal object application"))?;
        let mut arguments = self
            .facts
            .types
            .arguments(principal.arguments)
            .ok_or_else(|| self.error(key, "invalid object arguments"))?
            .collect::<Vec<_>>();
        let principal_arguments = u32::try_from(arguments.len())
            .map_err(|_| self.error(key, "too many object arguments"))?;
        if declared.scope.is_none() {
            let mut shift = substitute::Bound::shift(1);
            for argument in &mut arguments {
                *argument = shift
                    .apply_argument(&mut self.facts.types, *argument)
                    .map_err(|e| self.error(key, e))?;
            }
        }
        let mut bindings = Vec::new();
        for equality in equalities {
            bindings.push(self.object_equality(equality.source, equality.value)?);
        }
        bindings.sort_unstable_by_key(|equality| equality.binding.item.0);
        for equality in &bindings {
            bound_regions = bound_regions.max(equality.bound_regions);
            arguments.extend_from_slice(&equality.inputs);
            arguments.push(Argument::Type(equality.result));
        }
        self.reject_object_self(key, declared.definition, &arguments)?;
        let bindings = bindings
            .into_iter()
            .map(|equality| equality.binding)
            .collect::<Vec<_>>();
        let shape = self
            .facts
            .types
            .intern_dynamic_shape(dynamic::Bounds {
                principal: Some(principal.item),
                principal_arguments,
                bindings: &bindings,
                auto_traits: &pending.autos,
            })
            .map_err(|e| self.error(key, e))?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(key, e))?;
        self.intern(
            key,
            Type::Dynamic {
                shape,
                arguments,
                bound_regions,
                region,
            },
        )
    }

    fn object_equality(&self, key: NodeKey, clause: Clause) -> Result<Equality, Error> {
        let (value, bound_regions) = match clause {
            Clause::Predicate(_) => {
                return Err(self.error(key, "object binding requires a complete binder"));
            }
            Clause::ForAll(id) => {
                let value = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(key, "invalid object equality binder"))?;
                (value.predicate, value.bound_regions)
            }
        };
        let Bound::ProjectionEqual(result) = value.bound else {
            return Err(self.error(key, "object binding requires an associated equality"));
        };
        let Some(Type::Projection(application)) = self.facts.types.kind(value.ty) else {
            return Err(self.error(key, "object binding requires an associated projection"));
        };
        if application.arguments != crate::types::GenericArgs::EMPTY {
            return Err(self.error(key, "object cannot bind generic associated types"));
        }
        let parent = self
            .facts
            .types
            .trait_application(application.trait_application)
            .ok_or_else(|| self.error(key, "missing object binding parent"))?;
        let inputs = self
            .facts
            .types
            .arguments(parent.arguments)
            .ok_or_else(|| self.error(key, "invalid object binding inputs"))?
            .collect::<Vec<_>>();
        Ok(Equality {
            binding: dynamic::Binding {
                item: application.item,
                trait_item: parent.item,
                arguments: u32::try_from(inputs.len())
                    .map_err(|_| self.error(key, "too many object binding inputs"))?,
            },
            inputs,
            result,
            bound_regions,
        })
    }

    fn reject_object_self(
        &mut self,
        key: NodeKey,
        principal: DefId,
        arguments: &[Argument],
    ) -> Result<(), Error> {
        let subject = self
            .facts
            .associated
            .schema(principal)
            .ok_or_else(|| self.error(key, "missing object Self identity"))?
            .self_type;
        let Some(Type::Parameter(parameter)) = self.facts.types.kind(subject) else {
            return Err(self.error(key, "invalid object Self identity"));
        };
        let mut uses = crate::types::parameters::uses::Uses::default();
        for &argument in arguments {
            if let Argument::Type(ty) = argument {
                uses.collect(&self.facts.types, ty)
                    .map_err(|e| self.error(key, e))?;
            }
        }
        if uses.parameters.contains(&parameter) {
            return Err(self.error(
                key,
                "object arguments cannot depend on an erased Self default",
            ));
        }
        Ok(())
    }

    pub(super) fn check_object_bindings(
        &mut self,
        source: NodeKey,
        ty: TypeId,
        principal: DefId,
        queries: &mut crate::sema::dyn_compat::Queries,
    ) -> Result<(), Error> {
        let mut inputs = self.object_bound_inputs(source, ty, principal)?;
        let mut required = HashSet::default();
        let mut graph = Graph::new(inputs.root);
        while let Some(visit) = graph
            .next(&mut self.facts)
            .map_err(|e| self.error(source, e))?
        {
            let Some((id, _)) = visit.predicate.bound.trait_parts() else {
                return Err(self.error(source, "object graph requires a trait application"));
            };
            let item = self
                .facts
                .types
                .trait_application(id)
                .ok_or_else(|| self.error(source, "invalid object trait application"))?
                .item;
            let contract = queries
                .demand(self.report, &self.facts, DefId(item.0))
                .map_err(|e| self.error(source, e.to_string()))?;
            for &member in &contract.associated {
                required.insert(self.object_member_key(
                    source,
                    ItemId(member.0),
                    id,
                    visit.predicate.ty,
                    visit.bound_regions,
                    inputs.fixed,
                )?);
            }
            for clause in visit.implied {
                let value = quantified_value(&self.facts.types, clause)
                    .map_err(|e| self.error(source, e))?;
                if matches!(value.bound, Bound::ProjectionEqual(_))
                    && let Some(Type::Projection(application)) = self.facts.types.kind(value.ty)
                    && application.subject == inputs.subject
                {
                    let Clause::ForAll(id) = clause else {
                        return Err(
                            self.error(source, "inherited object equality requires a binder")
                        );
                    };
                    let count = self
                        .facts
                        .types
                        .quantified(id)
                        .ok_or_else(|| self.error(source, "invalid inherited object binder"))?
                        .bound_regions;
                    inputs.supplied.insert(self.object_member_key(
                        source,
                        application.item,
                        application.trait_application,
                        inputs.subject,
                        count,
                        inputs.fixed,
                    )?);
                }
            }
        }
        if !required.is_subset(&inputs.supplied) {
            return Err(self.error(
                source,
                "object requires an equality for each associated type",
            ));
        }
        Ok(())
    }

    fn object_bound_inputs(
        &mut self,
        source: NodeKey,
        ty: TypeId,
        principal: DefId,
    ) -> Result<Inputs, Error> {
        let Some(Type::Dynamic {
            shape,
            arguments,
            bound_regions,
            ..
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(source, "missing object type"));
        };
        let value = self
            .facts
            .types
            .dynamic_shape(shape)
            .ok_or_else(|| self.error(source, "invalid object bound shape"))?;
        let arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| self.error(source, "invalid object inputs"))?
            .collect::<Vec<_>>();
        let application = self
            .facts
            .types
            .dynamic_principal(ty)
            .map_err(|e| self.error(source, e))?
            .ok_or_else(|| self.error(source, "missing object principal"))?;
        let subject = self
            .facts
            .associated
            .schema(principal)
            .ok_or_else(|| self.error(source, "missing object principal schema"))?
            .self_type;
        let root = self
            .facts
            .types
            .intern_quantified(Quantified {
                bound_regions,
                predicate: Predicate {
                    ty: subject,
                    bound: Bound::Trait(application),
                },
            })
            .map(Clause::ForAll)
            .map_err(|e| self.error(source, e))?;
        let mut slots = crate::types::bound_uses::captures::Slots::default();
        let fixed = slots
            .predicate(
                &self.facts.types,
                Predicate {
                    ty: subject,
                    bound: Bound::Trait(application),
                },
            )
            .map_err(|e| self.error(source, e))?;
        let bindings = self
            .facts
            .types
            .dynamic_bindings(shape)
            .ok_or_else(|| self.error(source, "invalid object equalities"))?
            .to_vec();
        let mut supplied = HashSet::default();
        let mut start = value.principal_arguments as usize;
        for binding in bindings {
            let end = start + binding.arguments as usize;
            let args = arguments
                .get(start..end)
                .ok_or_else(|| self.error(source, "invalid object equality inputs"))?;
            let args = self
                .facts
                .types
                .intern_arguments(args)
                .map_err(|e| self.error(source, e))?;
            let application = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: binding.trait_item,
                    arguments: args,
                })
                .map_err(|e| self.error(source, e))?;
            supplied.insert(self.object_member_key(
                source,
                binding.item,
                application,
                subject,
                bound_regions,
                fixed,
            )?);
            start = end + 1;
        }
        Ok(Inputs {
            root,
            subject,
            supplied,
            fixed,
        })
    }
}
