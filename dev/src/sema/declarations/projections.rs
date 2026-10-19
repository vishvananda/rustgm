//! Associated type paths suspend normalization while their selected definitions elaborate.
mod aliases;
mod applications;
mod bound_environments;
mod deferred;
mod environments;
mod implicit;
pub(super) mod lookup;
mod proofs;
mod scoped;
mod selection;
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
    family: GenericArgs,
    restore: Option<Box<crate::types::substitute::Bound>>,
}
enum Selection {
    Deferred,
    Header(DefId),
    Definition(Selected),
    Resolved(Resolution),
}
struct Query {
    fold: Normalizer,
    context: regions::Context,
    selected: Option<Selected>,
    pending: Option<selection::Pending>,
    reserved_sink: Option<usize>,
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
#[derive(Clone, Copy)]
struct Root {
    ty: TypeId,
    normalized_subject: Option<TypeId>,
}
#[derive(Default)]
pub(super) struct Pending {
    lookup: lookup::Pending,
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
        let name = path.suffix[0].1;
        let (prefix, owner) = self.resolved_trait_path(NodeKey {
            node: trait_path,
            ..key
        })?;
        let member = self.qualified_projection_member(key, owner, name)?;
        self.work.push(Work::ProjectionPath(key, context));
        self.work.push(Work::PathArguments(suffix, member, context));
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
        let name = path.suffix[0].1;
        let (prefix, owner) = self.resolved_trait_path(NodeKey {
            node: path
                .trait_path
                .ok_or_else(|| self.error(key, "missing projection trait"))?,
            ..key
        })?;
        let suffix = NodeKey {
            node: path.suffix[0].0,
            ..key
        };
        let declaration = self.qualified_projection_member(key, owner, name)?;
        if self.partial_projection_path(
            key,
            crate::sema::generics::AnnotationProjection {
                subject: subject_key,
                parent: prefix,
                member: suffix,
                declaration,
            },
            context,
        )? {
            return Ok(());
        }
        let subject = self.ty(subject_key)?;
        let arguments = self.application_arguments(prefix, owner, context)?;
        let family = self.projection_arguments(suffix, declaration, context)?;
        let trait_application = self.applied_trait(prefix, owner, subject, arguments)?;
        let ty = self.intern(
            key,
            Type::Projection(projection::Application {
                item: ItemId(declaration.0),
                subject,
                trait_application,
                arguments: family,
            }),
        )?;
        let original_subject = self.facts.unreduced.node(subject_key).unwrap_or(subject);
        let original_arguments = self.facts.unreduced.arguments(prefix).unwrap_or(arguments);
        let original_family = self.facts.unreduced.arguments(suffix).unwrap_or(family);
        if original_subject != subject
            || original_arguments != arguments
            || original_family != family
        {
            let trait_application =
                self.applied_trait(prefix, owner, original_subject, original_arguments)?;
            let original = self.intern(
                key,
                Type::Projection(projection::Application {
                    item: ItemId(declaration.0),
                    subject: original_subject,
                    trait_application,
                    arguments: original_family,
                }),
            )?;
            self.facts.unreduced.retain_node(key, original);
        }
        // The subject's own annotation has already retained formation evidence
        // in this scope. Trait defaults may introduce new projections, so only
        // the checked subject, not the newly assembled arguments, is reusable.
        self.normalize_annotation_root(
            key,
            Root {
                ty,
                normalized_subject: Some(subject),
            },
            context,
        )
    }

    pub(super) fn normalize_annotation(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
    ) -> Result<(), Error> {
        self.normalize_annotation_root(
            key,
            Root {
                ty,
                normalized_subject: None,
            },
            context,
        )
    }

    fn normalize_annotation_root(
        &mut self,
        key: NodeKey,
        root: Root,
        context: regions::Context,
    ) -> Result<(), Error> {
        let ty = root.ty;
        if self.facts.types.contains_projections(ty) != Some(true) {
            return self.store(key, ty);
        }
        self.facts.unreduced.retain_node(key, ty);
        if self.header_projection_context(context) {
            self.projections.raw_bound_nodes.remove(&key);
            return self.store(key, ty);
        }
        if self.projections.raw_bound_nodes.remove(&key) {
            return self.store(key, ty);
        }
        self.start_scoped_normalization(
            key,
            root,
            context,
            Destination::Annotation,
            scoped::Ambient::Root,
        )
    }

    fn start_scoped_normalization(
        &mut self,
        key: NodeKey,
        root: Root,
        context: regions::Context,
        destination: Destination,
        ambient: scoped::Ambient,
    ) -> Result<(), Error> {
        let mut fold = Normalizer::default();
        let step = match root.normalized_subject {
            Some(input) => fold.start_with_input(&mut self.facts.types, root.ty, input),
            None => fold.start(&mut self.facts.types, root.ty),
        }
        .map_err(|e| self.error(key, e))?;
        self.projections.roots.insert(
            key,
            Query {
                fold,
                context,
                selected: None,
                pending: None,
                reserved_sink: None,
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
            Selection::Header(owner) => {
                self.work.push(Work::ProjectionSelection(key, request));
                self.work
                    .push(Work::ImplHeader(owner, super::impl_headers::Phase::Begin));
                return Ok(());
            }
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
            .instantiate_associated_definition(
                selected.owner,
                selected.arguments,
                selected.definition,
                selected.family,
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
}
