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
use crate::support::hash::HashMap;
use crate::{
    resolve::model::{DefKind, Namespace},
    types::{
        GenericArgs, ItemId, projection,
        substitute::{Normalizer, Request, Resolution, Step},
    },
};

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
    Dependency(crate::sema::traits::selection::Dependency),
    /// A fresh query consumes this trait associated type's result contract.
    Contract(DefId),
    Definition(Selected),
    Resolved(Resolution),
}
struct Query {
    original: TypeId,
    provisional_size: bool,
    fold: Normalizer,
    context: regions::Context,
    selected: Option<Selected>,
    pending: Option<selection::Pending>,
    reserved_sink: Option<usize>,
    destination: Destination,
    ambient: scoped::Ambient,
    scopes: scoped::Mapping,
}
/// A normalization root: a written source node, or an impl trait reference
/// argument that has no node of its own. An argument is identified by its impl
/// and argument index; `site` is that impl's trait path, used only to locate
/// diagnostics, so no node-keyed table ever holds an argument root.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum RootKey {
    Node(NodeKey),
    Argument {
        implementation: DefId,
        index: u16,
        site: NodeKey,
    },
}
impl RootKey {
    pub(super) const fn site(self) -> NodeKey {
        match self {
            Self::Node(key) | Self::Argument { site: key, .. } => key,
        }
    }
}
#[derive(Clone, Copy)]
enum Destination {
    Annotation,
    Requirement,
    Sized,
    Recheck(TypeId),
    /// A header kept as written beside a bound on its own `Self`.
    Header(TypeId),
}
#[derive(Clone, Copy)]
struct Root {
    ty: TypeId,
    normalized_subject: Option<TypeId>,
}
#[derive(Default)]
pub(super) struct Pending {
    rechecks: Vec<(RootKey, regions::Context, scoped::Ambient, TypeId, TypeId)>,
    headers: Vec<(RootKey, regions::Context, TypeId)>,
    self_bounded: HashMap<DefId, bool>,
    lookup: lookup::Pending,
    sized: Vec<(NodeKey, regions::Context, scoped::Ambient, TypeId)>,
    alias_environments:
        HashMap<DefId, std::sync::Arc<crate::types::parameters::environment::Environment>>,
    deferred: Vec<(NodeKey, regions::Context, scoped::Ambient, TypeId)>,
    plans: Vec<scoped::Plan>,
    body_sinks: HashMap<NodeKey, usize>,
    environments: HashMap<DefId, super::State>,
    raw_bound_nodes: crate::support::hash::HashSet<NodeKey>,
    raw_bounds: HashMap<DefId, bound_environments::Bounds>,
    raw_environments:
        HashMap<DefId, std::sync::Arc<crate::types::parameters::environment::Environment>>,
    alias_templates: HashMap<regions::BinderId, aliases::Template>,
    alias_scopes: HashMap<aliases::ScopeKey, crate::sema::alias_requirements::Scope>,
    roots: HashMap<RootKey, Query>,
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
            RootKey::Node(key),
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
        self.normalize_root(RootKey::Node(key), ty, context)
    }

    /// A root whose subject is not yet normalized: a written annotation, or an
    /// impl trait reference argument that has no node of its own.
    pub(super) fn normalize_root(
        &mut self,
        root: RootKey,
        ty: TypeId,
        context: regions::Context,
    ) -> Result<(), Error> {
        self.normalize_annotation_root(
            root,
            Root {
                ty,
                normalized_subject: None,
            },
            context,
        )
    }

    /// Written annotations and impl trait reference arguments share this
    /// routine; only where the original and the result are kept depends on
    /// the root.
    fn normalize_annotation_root(
        &mut self,
        root: RootKey,
        value: Root,
        context: regions::Context,
    ) -> Result<(), Error> {
        let ty = value.ty;
        if self.facts.types.contains_projections(ty) != Some(true) {
            return self.store_root(root, ty);
        }
        // Only written nodes are raw bounds; an argument root keeps its
        // original beside its result in the impl's pending reference.
        let raw_bound = match root {
            RootKey::Node(key) => {
                self.facts.unreduced.retain_node(key, ty);
                self.projections.raw_bound_nodes.remove(&key)
            }
            RootKey::Argument { .. } => false,
        };
        if self.header_projection_context(context) || raw_bound {
            return self.store_root(root, ty);
        }
        if self.self_bounded_header(context) {
            self.projections.headers.push((root, context, ty));
            return self.store_root(root, ty);
        }
        self.start_scoped_normalization(
            root,
            value,
            context,
            Destination::Annotation,
            scoped::Ambient::Root,
        )
    }

    fn store_root(&mut self, root: RootKey, ty: TypeId) -> Result<(), Error> {
        match root {
            RootKey::Node(key) => self.store(key, ty),
            RootKey::Argument {
                implementation,
                index,
                site,
            } => self
                .impl_headers
                .complete_argument(implementation, index, ty)
                .map_err(|e| self.error(site, e)),
        }
    }

    fn start_scoped_normalization(
        &mut self,
        root: RootKey,
        value: Root,
        context: regions::Context,
        destination: Destination,
        ambient: scoped::Ambient,
    ) -> Result<(), Error> {
        let key = root.site();
        let mut fold = Normalizer::default();
        let step = match value.normalized_subject {
            Some(input) => fold.start_with_input(&mut self.facts.types, value.ty, input),
            None => fold.start(&mut self.facts.types, value.ty),
        }
        .map_err(|e| self.error(key, e))?;
        self.projections.roots.insert(
            root,
            Query {
                original: value.ty,
                provisional_size: false,
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
        self.advance_projection(root, step)
    }

    fn advance_projection(&mut self, root: RootKey, step: Step) -> Result<(), Error> {
        let key = root.site();
        match step {
            Step::Complete(ty) => {
                let query = self
                    .projections
                    .roots
                    .remove(&root)
                    .ok_or_else(|| self.error(key, "missing completed projection query"))?;
                if query.provisional_size {
                    self.projections.rechecks.push((
                        root,
                        query.context,
                        query.ambient,
                        query.original,
                        ty,
                    ));
                }
                match query.destination {
                    Destination::Recheck(expected) if ty != expected => Err(self.error(
                        key,
                        "projection normalization changed after declaration completion",
                    )),
                    Destination::Header(written) if ty != written => Err(self.error(
                        key,
                        "impl header projection must stay rigid when the impl's bounds name `Self`",
                    )),
                    Destination::Sized => {
                        let depth = self.ambient_depth(key, query.ambient)?;
                        self.prove_alias_size(key, query.context, depth, ty)
                    }
                    Destination::Annotation => self.store_root(root, ty),
                    Destination::Recheck(_) | Destination::Header(_) | Destination::Requirement => {
                        Ok(())
                    }
                }
            }
            Step::Resolve(request) => {
                self.work.push(Work::ProjectionSelection(root, request));
                self.demand_projection_environment(root, request)
            }
        }
    }

    pub(super) fn complete_projection_selection(
        &mut self,
        root: RootKey,
        request: Request,
    ) -> Result<(), Error> {
        let key = root.site();
        let selected = match self.select_projection(root, request)? {
            Selection::Deferred => return self.defer_projection_requirement(root, request),
            Selection::Dependency(dependency) => {
                self.work.push(Work::ProjectionSelection(root, request));
                self.work.push(Work::selection_dependency(dependency));
                return Ok(());
            }
            Selection::Contract(member) => {
                self.work.push(Work::ProjectionSelection(root, request));
                self.work.push(Work::Definition(member, false));
                return Ok(());
            }
            Selection::Definition(selected) => selected,
            Selection::Resolved(resolution) => {
                let query = self
                    .projections
                    .roots
                    .get_mut(&root)
                    .ok_or_else(|| Error::at(key.file, 0, "missing pending projection fold"))?;
                let step = query
                    .fold
                    .resume(&mut self.facts.types, request, resolution)
                    .map_err(|e| self.error(key, e))?;
                return self.advance_projection(root, step);
            }
        };
        let definition = selected.definition;
        self.projections
            .roots
            .get_mut(&root)
            .ok_or_else(|| Error::at(key.file, 0, "missing projection scope"))?
            .selected = Some(selected);
        self.work.push(Work::ProjectionResult(root));
        self.work.push(Work::Definition(definition, false));
        Ok(())
    }

    pub(super) fn complete_projection_result(&mut self, root: RootKey) -> Result<(), Error> {
        let key = root.site();
        let mut selected = self
            .projections
            .roots
            .get_mut(&root)
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
            .get_mut(&root)
            .ok_or_else(|| Error::at(key.file, 0, "missing pending projection fold"))?;
        let step = query
            .fold
            .resume(
                &mut self.facts.types,
                selected.request,
                Resolution::Type(ty),
            )
            .map_err(|e| self.error(key, e))?;
        self.advance_projection(root, step)
    }
}
