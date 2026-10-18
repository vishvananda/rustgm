//! Trait paths use ordinary argument elaboration, with Self supplied by the bound subject.
use super::{Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::types::{
    ItemId,
    arguments::Argument,
    substitute::Bound,
    traits::{Application, ApplicationId},
};
use crate::{
    resolve::model::{DefKind, Namespace, Target},
    sema::generics::TraitUse,
};

#[derive(Clone, Copy)]
pub(super) struct Callable {
    owner: DefId,
    method: DefId,
    prefix: NodeKey,
    suffix: NodeKey,
    subject: Option<NodeKey>,
}

impl Checker<'_> {
    pub(super) fn schedule_trait_callable(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<crate::syntax::ast::NodeId>,
    ) -> Result<bool, Error> {
        let Some(callable) = self.trait_callable(key)? else {
            return Ok(false);
        };
        if self.facts.generics.trait_uses.contains_key(&key)
            && self.facts.generics.item_uses.contains_key(&key)
        {
            return Ok(true);
        }
        let Callable {
            owner,
            method,
            prefix,
            suffix,
            subject,
        } = callable;
        self.work.push(super::Work::TraitCallable(key, callable));
        if let Some(subject) = subject {
            self.annotation_holes(subject, steps, limit, scan)?;
            self.node_work(subject.file, subject.node, context);
            self.annotation_holes(prefix, steps, limit, scan)?;
            self.work
                .push(super::Work::PathArguments(prefix, owner, context));
        }
        self.method_path_segments(key, prefix, suffix)?;
        self.schedule_item_arguments(prefix, owner, context, steps, limit, scan)?;
        self.schedule_item_arguments(suffix, method, context, steps, limit, scan)?;
        self.work.push(super::Work::Definition(method, false));
        Ok(true)
    }
    pub(super) fn complete_trait_callable(
        &mut self,
        key: NodeKey,
        callable: Callable,
    ) -> Result<(), Error> {
        let Callable {
            owner,
            method,
            prefix,
            suffix,
            subject,
        } = callable;
        let trait_arguments = self
            .facts
            .generics
            .item_uses
            .remove(&prefix)
            .ok_or_else(|| self.error(key, "missing trait path arguments"))?;
        let method_arguments = self
            .facts
            .generics
            .item_uses
            .remove(&suffix)
            .ok_or_else(|| self.error(key, "missing trait method arguments"))?;
        self.facts.generics.trait_uses.insert(
            key,
            TraitUse {
                owner,
                arguments: Some(trait_arguments),
                subject,
            },
        );
        self.facts.generics.item_uses.insert(key, method_arguments);
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing trait callable path fact"))?
            .reference = Some(Target::Definition(method));
        Ok(())
    }
    fn trait_callable(&self, key: NodeKey) -> Result<Option<Callable>, Error> {
        if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
            let Some(trait_path) = path.trait_path else {
                return Ok(None);
            };
            if path.namespace != Namespace::Value || path.suffix.len() != 1 {
                return Ok(None);
            }
            let (prefix, owner) = self.resolved_trait_path(NodeKey {
                node: trait_path,
                ..key
            })?;
            let scope = self.report.resolution.definitions[owner.0 as usize]
                .members
                .ok_or_else(|| self.error(key, "trait declaration has no member scope"))?;
            let method = self.report.resolution.scopes[scope.0 as usize]
                .bindings
                .get(&(path.suffix[0].1, Namespace::Value))
                .map(|binding| binding.target)
                .filter(|id| {
                    matches!(
                        self.report.resolution.definitions[id.0 as usize].kind,
                        DefKind::Function | DefKind::Const
                    ) && self.facts.associated.trait_owner(*id) == Some(owner)
                })
                .ok_or_else(|| self.error(key, "qualified trait has no such method"))?;
            return Ok(Some(Callable {
                owner,
                method,
                prefix,
                suffix: NodeKey {
                    node: path.suffix[0].0,
                    ..key
                },
                subject: Some(NodeKey {
                    node: path.ty,
                    ..key
                }),
            }));
        }
        let Some(Target::Definition(method)) = self.facts.node(key).and_then(|fact| fact.reference)
        else {
            return Ok(None);
        };
        let Some(owner) = self.facts.associated.trait_owner(method) else {
            return Ok(None);
        };
        let Some(path) = self.report.resolution.associated_paths.get(&key) else {
            return Ok(None);
        };
        if path.owner != owner || path.suffix.len() != 1 {
            return Ok(None);
        }
        Ok(Some(Callable {
            owner,
            method,
            prefix: NodeKey {
                node: path.prefix,
                ..key
            },
            suffix: NodeKey {
                node: path.suffix[0].0,
                ..key
            },
            subject: None,
        }))
    }
    pub(super) fn resolved_trait_path(&self, mut key: NodeKey) -> Result<(NodeKey, DefId), Error> {
        loop {
            if let Some(crate::resolve::model::Target::Definition(id)) =
                self.facts.node(key).and_then(|facts| facts.reference)
            {
                return if self.report.resolution.definitions[id.0 as usize].kind == DefKind::Trait {
                    Ok((key, id))
                } else {
                    Err(self.error(key, "implementation requires a trait declaration"))
                };
            }
            key.node = match self.kind(key) {
                crate::syntax::ast::Kind::Group(node) => node,
                crate::syntax::ast::Kind::Type {
                    flavor: crate::syntax::ast::TypeFlavor::Path,
                    elements,
                } => *self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements)
                    .first()
                    .ok_or_else(|| self.error(key, "empty trait implementation path"))?,
                _ => return Err(self.error(key, "implementation requires a resolved trait path")),
            };
        }
    }

    pub(super) fn retain_trait_contract(
        &mut self,
        owner: crate::sema::associated::Owner,
        definition: DefId,
        self_bounds: std::collections::HashSet<NodeKey>,
    ) -> Result<(), Error> {
        let schema = self
            .facts
            .associated
            .schema(definition)
            .ok_or_else(|| self.error(owner.source, "missing trait Self schema"))?;
        let mut header = crate::sema::traits::Header {
            self_type: schema.self_type,
            self_bounds,
            parameters: self.facts.generics.parameters[&definition].clone(),
            predicates: self
                .facts
                .generics
                .predicates
                .get(&definition)
                .cloned()
                .unwrap_or_default(),
            results: Vec::new(),
            regions: schema.bounds.clone(),
        };
        if !self.associated_type_bounds.is_empty() {
            header
                .partition_results(&self.facts.types, ItemId(definition.0))
                .map_err(|e| self.error(owner.source, e))?;
        }
        self.facts
            .generics
            .traits
            .get_or_insert_with(Default::default)
            .declare(ItemId(definition.0), header)
            .map_err(|e| self.error(owner.source, e))
    }

    pub(super) fn complete_trait_contracts(&mut self) -> Result<(), Error> {
        if let Some(contracts) = &self.facts.generics.traits {
            contracts
                .check_cycles(&self.facts.types)
                .map_err(|(item, error)| {
                    let key = self.report.resolution.definitions[item.0 as usize].node;
                    key.map_or_else(
                        || Error::at(crate::resolve::unit::FileId(0), 0, error),
                        |key| self.error(key, error),
                    )
                })?;
        }
        self.complete_projection_promises()?;
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            if let crate::sema::associated::OwnerKind::Trait { definition, .. } = owner.kind {
                self.facts
                    .generics
                    .complete_environment(&mut self.facts.types, definition)
                    .map_err(|e| self.error(owner.source, e))?;
            }
        }
        Ok(())
    }

    pub(super) fn trait_requirements(
        &mut self,
        predicate: crate::sema::generics::Predicate,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        use crate::types::parameters::{clauses::Clause, predicates::Bound as Requirement};
        let requirements = self
            .facts
            .generics
            .traits
            .as_mut()
            .ok_or_else(|| Error::at(predicate.source.file, 0, "missing trait contracts"))?
            .formation(&mut self.facts.types, predicate.value)
            .map_err(|e| self.error(predicate.source, e))?;
        // Ordinary subjects were checked by validate_bounds. Quantified subjects
        // require the independently lifted input, if formation was not deferred.
        let subject = if matches!(predicate.value, Clause::ForAll(_)) {
            requirements.subject()
        } else {
            None
        };
        for ty in subject
            .into_iter()
            .chain(requirements.types().iter().copied())
        {
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(predicate.source, e))?
                .well_formed
            {
                return Err(self.error(predicate.source, "trait argument must be well-formed"));
            }
        }
        for &requirement in requirements.required() {
            let requirement = match requirement {
                Clause::Predicate(value) if matches!(value.bound, Requirement::Outlives(_)) => {
                    continue; // Proved with the completed nominal lifetime contracts.
                }
                Clause::Predicate(value) => proof
                    .normalize_builtin(&mut self.facts, value)
                    .map_err(|e| self.error(predicate.source, e))?
                    .into(),
                Clause::ForAll(_) => requirement,
            };
            if !proof
                .prove_clause(&mut self.facts, requirement)
                .map_err(|e| self.error(predicate.source, e))?
            {
                return Err(self.error(
                    predicate.source,
                    "trait argument does not satisfy its declaration requirements",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn trait_path(
        &mut self,
        path: NodeKey,
        owner: DefId,
        subject: TypeId,
        context: regions::Context,
    ) -> Result<ApplicationId, Error> {
        let arguments = self.application_arguments(path, owner, context)?;
        self.applied_trait(path, owner, subject, arguments)
    }

    pub(super) fn applied_trait(
        &mut self,
        path: NodeKey,
        owner: DefId,
        subject: TypeId,
        arguments: crate::types::GenericArgs,
    ) -> Result<ApplicationId, Error> {
        let self_type = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| {
                self.error(path, "trait bound requires an elaborated trait declaration")
            })?
            .self_type;
        let Some(Type::Parameter(self_parameter)) = self.facts.types.kind(self_type) else {
            return Err(self.error(path, "trait Self must be a rigid type parameter"));
        };
        let mut replace =
            Bound::parameter_only(&self.facts.types, self_parameter, Argument::Type(subject))
                .map_err(|e| self.error(path, e))?;
        let mut arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| self.error(path, "missing elaborated trait arguments"))?
            .collect::<Vec<_>>();
        for argument in &mut arguments {
            *argument = replace
                .apply_argument(&mut self.facts.types, *argument)
                .map_err(|e| self.error(path, e))?;
        }
        let arguments = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(path, e))?;
        self.facts
            .types
            .intern_trait_application(Application {
                item: ItemId(owner.0),
                arguments,
            })
            .map_err(|e| self.error(path, e))
    }
}
