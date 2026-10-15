//! Associated type paths suspend normalization while their selected definitions elaborate.
use super::{Checker, DefId, Error, NodeKey, Type, TypeId, Work, regions};
use crate::{
    resolve::model::{DefKind, Namespace},
    types::{
        GenericArgs, ItemId, projection,
        substitute::{Normalizer, Request, Resolution, Step},
    },
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Selected {
    request: Request,
    owner: DefId,
    definition: DefId,
    arguments: GenericArgs,
}
struct Query {
    fold: Normalizer,
    context: regions::Context,
    selected: Option<Selected>,
}
#[derive(Default)]
pub(super) struct Pending {
    roots: HashMap<NodeKey, Query>,
}

impl Checker<'_> {
    pub(super) fn schedule_projection_path(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let path = self
            .report
            .resolution
            .qualified_paths
            .get(&key)
            .ok_or_else(|| self.error(key, "missing qualified type path"))?;
        if path.namespace != Namespace::Type || path.suffix.len() != 1 {
            return Err(self.error(key, "associated type paths require one type member"));
        }
        let trait_path = path
            .trait_path
            .ok_or_else(|| self.error(key, "associated type path requires a trait qualifier"))?;
        let subject = path.ty;
        let suffix = NodeKey {
            node: path.suffix[0].0,
            ..key
        };
        if crate::sema::lifetimes::arguments(self.report, suffix, false)?.is_some() {
            return Err(self.error(
                suffix,
                "associated type arguments require advanced generic relationships",
            ));
        }
        let (prefix, owner) = self.resolved_trait_path(NodeKey {
            node: trait_path,
            ..key
        })?;
        self.work.push(Work::ProjectionPath(key, context));
        self.work.push(Work::PathArguments(prefix, owner, context));
        self.node_work(key.file, subject, context);
        Ok(())
    }

    pub(super) fn complete_projection_path(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let path = self
            .report
            .resolution
            .qualified_paths
            .get(&key)
            .ok_or_else(|| self.error(key, "missing qualified type path"))?;
        let subject_key = NodeKey {
            node: path.ty,
            ..key
        };
        let subject = self.ty(subject_key)?;
        let name = path.suffix[0].1;
        let (prefix, owner) = self.resolved_trait_path(NodeKey {
            node: path
                .trait_path
                .ok_or_else(|| self.error(key, "missing projection trait"))?,
            ..key
        })?;
        let scope = self.report.resolution.definitions[owner.0 as usize]
            .members
            .ok_or_else(|| self.error(key, "missing trait member scope"))?;
        let declaration = self.report.resolution.scopes[scope.0 as usize]
            .bindings
            .get(&(name, Namespace::Type))
            .map(|b| b.target)
            .filter(|id| {
                self.facts.associated.trait_owner(*id) == Some(owner)
                    && self.facts.associated.type_contract(*id).is_some()
            })
            .ok_or_else(|| self.error(key, "qualified trait has no prepared associated type"))?;
        let arguments = self.application_arguments(prefix, owner, context)?;
        let trait_application = self.applied_trait(prefix, owner, subject, arguments)?;
        let ty = self.intern(
            key,
            Type::Projection(projection::Application {
                item: ItemId(declaration.0),
                subject,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let original_subject = self.facts.unreduced.node(subject_key).unwrap_or(subject);
        let original_arguments = self.facts.unreduced.arguments(prefix).unwrap_or(arguments);
        if original_subject != subject || original_arguments != arguments {
            let trait_application =
                self.applied_trait(prefix, owner, original_subject, original_arguments)?;
            let original = self.intern(
                key,
                Type::Projection(projection::Application {
                    item: ItemId(declaration.0),
                    subject: original_subject,
                    trait_application,
                    arguments: GenericArgs::EMPTY,
                }),
            )?;
            self.facts.unreduced.retain_node(key, original);
        }
        self.normalize_annotation(key, ty, context)
    }

    pub(super) fn normalize_annotation(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.facts.types.contains_projections(ty) != Some(true) {
            return self.store(key, ty);
        }
        self.facts.unreduced.retain_node(key, ty);
        let mut fold = Normalizer::default();
        let step = fold
            .start(&mut self.facts.types, ty)
            .map_err(|e| self.error(key, e))?;
        self.projections.roots.insert(
            key,
            Query {
                fold,
                context,
                selected: None,
            },
        );
        self.advance_projection(key, step)
    }

    fn advance_projection(&mut self, key: NodeKey, step: Step) -> Result<(), Error> {
        match step {
            Step::Complete(ty) => {
                self.projections.roots.remove(&key);
                self.store(key, ty)
            }
            Step::Resolve(request) => {
                let selected = self.select_projection(key, request)?;
                self.projections
                    .roots
                    .get_mut(&key)
                    .ok_or_else(|| Error::at(key.file, 0, "missing projection scope"))?
                    .selected = Some(selected);
                self.work.push(Work::ProjectionResult(key));
                self.work.push(Work::Definition(selected.definition, false));
                Ok(())
            }
        }
    }

    pub(super) fn complete_projection_result(&mut self, key: NodeKey) -> Result<(), Error> {
        let selected = self
            .projections
            .roots
            .get_mut(&key)
            .and_then(|query| query.selected.take())
            .ok_or_else(|| self.error(key, "missing selected projection definition"))?;
        let template = self.facts.definitions[selected.definition.0 as usize]
            .ok_or_else(|| self.error(key, "missing selected associated type definition"))?;
        let ty = self
            .facts
            .generics
            .substitute(
                &mut self.facts.types,
                selected.owner,
                selected.arguments,
                template,
            )
            .map_err(|e| self.error(key, e))?;
        let query = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing pending projection fold"))?;
        let step = query
            .fold
            .resume(
                &mut self.facts.types,
                selected.request,
                Resolution::Type(ty),
            )
            .map_err(|e| self.error(key, e))?;
        self.advance_projection(key, step)
    }

    fn select_projection(&mut self, key: NodeKey, request: Request) -> Result<Selected, Error> {
        use crate::{
            sema::traits::{assumptions::Assumptions, selection},
            types::{
                infer::{Context, Instantiation},
                parameters::predicates::{Bound, Predicate},
            },
        };
        let application = request.application;
        let predicate = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        if predicate
            .escaping_binders(&self.facts.types)
            .map_err(|e| self.error(key, e))?
            != 0
        {
            return Err(self.error(
                key,
                "projection normalization requires a scoped lifetime mapping",
            ));
        }
        let context = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing projection scope"))?
            .context;
        let environment = self.selection_environment(key, context);
        let assumptions = self.selection_trait_assumptions(key, context, environment.as_ref())?;
        let empty = Assumptions::default();
        let mut inference = Context::default();
        let mut substitution = Instantiation::new(&mut inference, &self.facts.types, &[])
            .map_err(|e| self.error(key, e.to_string()))?;
        let goal = inference
            .instantiate_predicate(&mut self.facts.types, predicate, &mut substitution)
            .map_err(|e| self.error(key, e.to_string()))?;
        let selection = selection::select_declaration(
            &mut self.facts,
            &mut inference,
            &goal,
            selection::Caller {
                assumptions: assumptions.as_deref().unwrap_or(&empty),
                environment: environment.as_ref(),
                first_region: 1,
            },
        )
        .map_err(|e| self.error(key, e))?;
        let selection::Selection::Proven(evidence) = selection else {
            return Err(self.error(
                key,
                "associated type requires an unambiguous implementation",
            ));
        };
        let selection::Source::Implementation { owner, arguments } = evidence.source else {
            return Err(self.error(
                key,
                "symbolic associated types require projection equality obligations",
            ));
        };
        if !evidence.regions.is_empty()
            || !evidence.equalities.is_empty()
            || !evidence.outlives.is_empty()
            || !evidence.type_outlives.is_empty()
            || inference.region_equalities.iter().any(|(a, b)| a != b)
        {
            return Err(self.error(
                key,
                "projection normalization requires retained lifetime evidence",
            ));
        }
        let name = self.report.resolution.definitions[application.item.0 as usize].name;
        let scope = self.report.resolution.definitions[owner.0 as usize].scope;
        let definition = self.report.resolution.scopes[scope.0 as usize]
            .bindings
            .get(&(name, Namespace::Type))
            .map(|binding| binding.target)
            .filter(|id| {
                self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
                    && self.facts.associated.owner(*id) == self.facts.associated.owner(owner)
            })
            .ok_or_else(|| {
                self.error(
                    key,
                    "selected implementation is missing the associated type",
                )
            })?;
        Ok(Selected {
            request,
            owner,
            definition,
            arguments,
        })
    }
}
