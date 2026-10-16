//! Associated type paths suspend normalization while their selected definitions elaborate.
mod aliases;
mod bound_environments;
mod deferred;
mod environments;
mod proofs;
mod scoped;
use super::{Checker, DefId, Error, NodeKey, Type, TypeId, Work, regions};
use crate::{
    resolve::model::{DefKind, Namespace},
    types::{
        GenericArgs, ItemId, projection,
        substitute::{Normalizer, Request, Resolution, Step},
    },
};
use std::collections::HashMap;

struct Selected {
    request: Request,
    owner: DefId,
    definition: DefId,
    arguments: GenericArgs,
    restore: Option<Box<crate::types::substitute::Bound>>,
}
enum Selection {
    Deferred,
    Definition(Selected),
    Resolved(Resolution),
}
struct Query {
    fold: Normalizer,
    context: regions::Context,
    selected: Option<Selected>,
    destination: Destination,
    ambient: scoped::Ambient,
    scopes: scoped::Mapping,
}
#[derive(Clone, Copy)]
enum Destination {
    Annotation,
    Requirement,
    Sized,
}
#[derive(Default)]
pub(super) struct Pending {
    sized: Vec<(NodeKey, regions::Context, scoped::Ambient, TypeId)>,
    alias_environments:
        HashMap<DefId, std::sync::Arc<crate::types::parameters::environment::Environment>>,
    deferred: Vec<(NodeKey, regions::Context, scoped::Ambient, TypeId)>,
    plans: Vec<scoped::Plan>,
    body_sinks: HashMap<NodeKey, usize>,
    environments: HashMap<DefId, super::State>,
    raw_bound_nodes: std::collections::HashSet<NodeKey>,
    raw_bounds: HashMap<DefId, bound_environments::Bounds>,
    raw_environments:
        HashMap<DefId, std::sync::Arc<crate::types::parameters::environment::Environment>>,
    alias_templates: HashMap<regions::BinderId, aliases::Template>,
    alias_scopes: HashMap<aliases::ScopeKey, crate::sema::alias_requirements::Scope>,
    roots: HashMap<NodeKey, Query>,
    scopes: HashMap<Option<regions::BinderId>, usize>,
    proofs: Vec<(
        NodeKey,
        regions::Context,
        crate::sema::traits::obligations::Obligations,
    )>,
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
                    && self.facts.types.projection_schema(ItemId(id.0)).is_some()
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
        if self.projections.raw_bound_nodes.remove(&key) {
            return self.store(key, ty);
        }
        self.start_normalization(key, ty, context, Destination::Annotation)
    }

    fn start_normalization(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
        destination: Destination,
    ) -> Result<(), Error> {
        self.start_scoped_normalization(key, ty, context, destination, scoped::Ambient::Root)
    }

    fn start_scoped_normalization(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
        destination: Destination,
        ambient: scoped::Ambient,
    ) -> Result<(), Error> {
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
                destination,
                ambient,
                scopes: scoped::Mapping::default(),
            },
        );
        self.advance_projection(key, step)
    }

    fn advance_projection(&mut self, key: NodeKey, step: Step) -> Result<(), Error> {
        match step {
            Step::Complete(ty) => {
                let query = self
                    .projections
                    .roots
                    .remove(&key)
                    .ok_or_else(|| self.error(key, "missing completed projection query"))?;
                match query.destination {
                    Destination::Sized => {
                        let depth = self.ambient_depth(key, query.ambient)?;
                        self.prove_alias_size(key, query.context, depth, ty)
                    }
                    Destination::Annotation => self.store(key, ty),
                    Destination::Requirement => Ok(()),
                }
            }
            Step::Resolve(request) => {
                self.work.push(Work::ProjectionSelection(key, request));
                self.demand_projection_environment(key, request)
            }
        }
    }

    pub(super) fn complete_projection_selection(
        &mut self,
        key: NodeKey,
        request: Request,
    ) -> Result<(), Error> {
        let selected = match self.select_projection(key, request)? {
            Selection::Deferred => return self.defer_projection_requirement(key, request),
            Selection::Definition(selected) => selected,
            Selection::Resolved(resolution) => {
                let query = self
                    .projections
                    .roots
                    .get_mut(&key)
                    .ok_or_else(|| Error::at(key.file, 0, "missing pending projection fold"))?;
                let step = query
                    .fold
                    .resume(&mut self.facts.types, request, resolution)
                    .map_err(|e| self.error(key, e))?;
                return self.advance_projection(key, step);
            }
        };
        let definition = selected.definition;
        self.projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing projection scope"))?
            .selected = Some(selected);
        self.work.push(Work::ProjectionResult(key));
        self.work.push(Work::Definition(definition, false));
        Ok(())
    }

    pub(super) fn complete_projection_result(&mut self, key: NodeKey) -> Result<(), Error> {
        let mut selected = self
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
        let ty = if let Some(restore) = &mut selected.restore {
            restore
                .apply(&mut self.facts.types, ty)
                .map_err(|e| self.error(key, e))?
        } else {
            ty
        };
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

    fn select_projection(&mut self, key: NodeKey, request: Request) -> Result<Selection, Error> {
        use crate::{
            sema::{nominals::traits::State, traits::selection},
            types::parameters::predicates::{Bound, Predicate},
        };
        let application = request.application;
        let predicate = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        let context = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing projection scope"))?
            .context;
        let deferred = self.may_defer_projection(key, context, predicate)?;
        if deferred
            && predicate
                .contains_projections(&self.facts.types)
                .map_err(|e| self.error(key, e))?
        {
            return Ok(Selection::Deferred);
        }
        let (environment, first) = self.scoped_projection_environment(key, request, context)?;
        let projected = State::projection(
            &mut self.facts,
            environment,
            application,
            first,
            deferred,
            crate::sema::traits::implementations::Availability::Declarations,
        )
        .map_err(|e| self.error(key, e))?;
        let Some(source) = projected.source else {
            if deferred {
                return Ok(Selection::Deferred);
            }
            return Err(self.error(
                key,
                "associated type requires an unambiguous implementation",
            ));
        };
        let proof = projected.proof;
        if let Some(resolution) = projected.resolution {
            self.retain_scoped_projection_proof(key, request, context, proof)?;
            return Ok(Selection::Resolved(resolution));
        }
        let selection::Source::Implementation { owner, arguments } = source else {
            return Err(self.error(key, "missing associated assumption normal form"));
        };
        let mut restore = if proof.regions.is_empty() && proof.equalities.is_empty() {
            None
        } else {
            Some(Box::new(
                proof.restoration().map_err(|e| self.error(key, e))?,
            ))
        };
        self.retain_alias_sizes(
            key,
            request,
            context,
            projected.sized,
            restore.as_deref_mut(),
        )?;
        self.retain_scoped_projection_proof(key, request, context, proof)?;
        let definition = crate::sema::projections::definition(&self.facts, owner, application.item)
            .map_err(|e| self.error(key, e))?;
        Ok(Selection::Definition(Selected {
            request,
            owner,
            definition,
            arguments,
            restore,
        }))
    }
}
