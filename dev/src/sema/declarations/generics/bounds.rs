//! Decode resolved bounds once; subjects and trait arguments share the declaration worklist.
pub(in crate::sema::declarations) mod lexical;
use super::{Checker, DefId, Error, Kind, NodeKey, Type, Work, regions};
use crate::{
    resolve::model::{BuiltinTrait, Target},
    syntax::{
        ast::{List, TypeFlavor},
        item::GenericFlavor,
    },
    types::parameters::predicates::{Bound, Predicate},
};
use lexical::Scope;
#[derive(Clone, Copy)]
pub(in super::super) enum Subject {
    Parameter(DefId),
    Associated(DefId),
    Type(NodeKey),
}
#[derive(Clone, Copy)]
pub(in super::super) struct Syntax {
    source: NodeKey,
    subject: Subject,
    bound: Written,
    scope: Option<Scope>,
}
#[derive(Clone, Copy)]
pub(in super::super) enum Written {
    Trait(Bound),
    User { path: NodeKey, definition: DefId },
    Outlives(NodeKey),
}
impl Checker<'_> {
    pub(in super::super) fn prepare_bounds(
        &mut self,
        owner: DefId,
        key: NodeKey,
        parameters: &[NodeKey],
        context: regions::Context,
    ) -> Result<(), Error> {
        let item = self.item(key)?;
        let mut syntax = Vec::new();
        let mut type_parameters = crate::support::hash::HashSet::default();
        for key in parameters {
            if self.generic_syntax(*key)?.flavor == GenericFlavor::Type {
                type_parameters.insert(self.generic_binding(*key)?);
            }
        }
        for key in parameters {
            let parameter = self.generic_syntax(*key)?;
            if parameter.flavor == GenericFlavor::Lifetime {
                continue;
            }
            self.bound_list(
                *key,
                Subject::Parameter(self.generic_binding(*key)?),
                parameter.bounds,
                &type_parameters,
                (context, None),
                &mut syntax,
            )?;
        }
        if let crate::syntax::item::ItemKind::Trait { bounds, .. } = item.kind {
            let self_definition = self
                .facts
                .associated
                .owner(owner)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| self.error(key, "missing trait Self declaration"))?
                .self_definition;
            self.bound_list(
                key,
                Subject::Parameter(self_definition),
                bounds,
                &type_parameters,
                (context, None),
                &mut syntax,
            )?;
        }
        let clauses = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(item.where_clause)
            .to_vec();
        for node in clauses {
            let key = NodeKey { node, ..key };
            let Kind::Where {
                quantified,
                binders,
                ty,
                bounds,
            } = self.kind(key)
            else {
                return Err(self.error(key, "invalid where-clause"));
            };
            let subject = NodeKey { node: ty, ..key };
            if self.lifetime_argument(subject) {
                continue;
            }
            let scope = self.bound_scope(key, quantified.then_some(binders), context, true)?;
            self.bound_list(
                key,
                Subject::Type(subject),
                bounds,
                &type_parameters,
                (context, scope),
                &mut syntax,
            )?;
        }
        if !syntax.is_empty() {
            self.index_parameter_bounds(owner, &syntax, context);
            self.bound_syntax.insert(owner, syntax);
        } else if let Some(family) = self.facts.associated.families.get_mut(&owner) {
            // Preparation has completed an empty input contract. An absent
            // contract before this point still denotes unfinished work.
            family.requirements = Some(Vec::new());
        }
        Ok(())
    }
    /// Derived lookup indices share the decoded written contract. Neither index
    /// confers a caller permission or replaces the contract's ordinary proof.
    fn index_parameter_bounds(
        &mut self,
        owner: DefId,
        syntax: &[Syntax],
        context: regions::Context,
    ) {
        for bound in syntax {
            let subject = match bound.subject {
                Subject::Parameter(id) => Some(id),
                Subject::Type(key) => self.bound_parameter(key, true),
                Subject::Associated(_) => None,
            };
            if bound.scope.is_none()
                && let (Some(subject), Written::Outlives(key)) = (subject, bound.bound)
            {
                self.retain_object_container_bound(subject, key);
            }
            if let (Some(subject), Written::User { path, definition }) = (subject, bound.bound) {
                self.retain_projection_bound(
                    owner,
                    subject,
                    path,
                    definition,
                    bound.context(context),
                    bound.scope,
                );
            }
        }
    }
    fn bound_list(
        &mut self,
        key: NodeKey,
        subject: Subject,
        list: List,
        parameters: &crate::support::hash::HashSet<DefId>,
        scope: (regions::Context, Option<Scope>),
        output: &mut Vec<Syntax>,
    ) -> Result<(), Error> {
        let (context, enclosing) = scope;
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(list)
            .to_vec();
        for node in nodes {
            let key = NodeKey { node, ..key };
            if self.lifetime_argument(key) {
                output.push(Syntax {
                    source: key,
                    subject,
                    bound: Written::Outlives(key),
                    scope: enclosing,
                });
                continue;
            }
            let Kind::Bound {
                optional,
                quantified,
                binders,
                value,
            } = self.kind(key)
            else {
                return Err(self.error(
                    key,
                    "generic type-outlives bounds require symbolic region proofs",
                ));
            };
            if enclosing.is_some() && quantified {
                return Err(self.error(key, "nested lifetime quantification is not allowed"));
            }
            let scope = enclosing.or(self.bound_scope(
                key,
                quantified.then_some(binders),
                context,
                false,
            )?);
            let value = NodeKey { node: value, ..key };
            if self.lifetime_argument(value) {
                if optional {
                    return Err(self.error(key, "lifetime bounds cannot be relaxed"));
                }
                output.push(Syntax {
                    source: key,
                    subject,
                    bound: Written::Outlives(value),
                    scope,
                });
                continue;
            }
            let bound = self.trait_bound(value)?;
            let scope = self.call_bound_scope(key, bound, scope, context)?;
            if optional {
                if !matches!(bound, Written::Trait(Bound::Sized)) {
                    return Err(self.error(key, "only Sized bounds can be relaxed"));
                }
                let binding = match subject {
                    Subject::Parameter(id) => Some(id),
                    Subject::Associated(_) => None,
                    Subject::Type(key) => self.bound_parameter(key, false),
                };
                let binding = binding
                    .filter(|id| parameters.contains(id))
                    .ok_or_else(|| {
                        self.error(
                            key,
                            "relaxed Sized bounds require a declared type parameter",
                        )
                    })?;
                if !self.relaxed.insert(binding) {
                    return Err(self.error(key, "type parameter has repeated relaxed Sized bounds"));
                }
            } else {
                output.push(Syntax {
                    source: key,
                    subject,
                    bound,
                    scope,
                });
            }
        }
        Ok(())
    }
    pub(in super::super) fn bound_parameter(
        &self,
        mut key: NodeKey,
        parentheses: bool,
    ) -> Option<DefId> {
        loop {
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) {
                return Some(id);
            }
            key.node = match self.kind(key) {
                Kind::Group(node) if parentheses => node,
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => *self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements)
                    .first()?,
                _ => return None,
            };
        }
    }
    /// Only written bounds on the resolved Self parameter are supertraits.
    /// An alias that later normalizes to Self is a separate required predicate.
    pub(in super::super) fn declared_self_bounds(
        &self,
        owner: DefId,
        self_definition: DefId,
    ) -> impl Iterator<Item = (NodeKey, Written)> + '_ {
        self.bound_syntax
            .get(&owner)
            .into_iter()
            .flatten()
            .filter_map(move |syntax| {
                let subject = match syntax.subject {
                    Subject::Parameter(id) => Some(id),
                    Subject::Type(key) => self.bound_parameter(key, true),
                    Subject::Associated(_) => None,
                };
                (subject == Some(self_definition)).then_some((syntax.source, syntax.bound))
            })
    }
    pub(in super::super) fn trait_bound(&self, mut key: NodeKey) -> Result<Written, Error> {
        loop {
            if self.lifetime_argument(key) {
                return Err(self.error(
                    key,
                    "generic type-outlives bounds require symbolic region proofs",
                ));
            }
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) {
                let builtin = self.report.resolution.builtin_traits.identify(id);
                if matches!(builtin, Some(BuiltinTrait::Callable(_))) {
                    if crate::sema::lifetimes::call_arguments(self.report, key)?.is_none() {
                        return Err(
                            self.error(key, "call traits require stable parenthesized arguments")
                        );
                    }
                    return Ok(Written::User {
                        path: key,
                        definition: id,
                    });
                }
                if matches!(
                    builtin,
                    Some(
                        BuiltinTrait::Language(_)
                            | BuiltinTrait::Comparison(_)
                            | BuiltinTrait::Iteration(_)
                    )
                ) {
                    return Ok(Written::User {
                        path: key,
                        definition: id,
                    });
                }
                if builtin.is_some()
                    && crate::sema::lifetimes::arguments(self.report, key, false)?
                        .is_some_and(|list| list.start != list.end)
                {
                    return Err(self.error(key, "builtin traits do not accept generic arguments"));
                }
                return match builtin {
                    Some(BuiltinTrait::Copy) => Ok(Written::Trait(Bound::Copy)),
                    Some(BuiltinTrait::Clone) => Ok(Written::Trait(Bound::Clone)),
                    Some(BuiltinTrait::Sized) => Ok(Written::Trait(Bound::Sized)),
                    Some(BuiltinTrait::Auto(marker)) => Ok(Written::Trait(Bound::Auto(marker))),
                    Some(
                        BuiltinTrait::Language(_)
                        | BuiltinTrait::Comparison(_)
                        | BuiltinTrait::Iteration(_),
                    ) => Ok(Written::User {
                        path: key,
                        definition: id,
                    }),
                    Some(BuiltinTrait::Callable(_)) => {
                        Err(self.error(key, "missing call-trait syntax"))
                    }
                    None if self.report.resolution.definitions[id.0 as usize].kind
                        == crate::resolve::model::DefKind::Trait =>
                    {
                        if crate::sema::lifetimes::call_arguments(self.report, key)?.is_some() {
                            return Err(self.error(
                                key,
                                "parenthesized arguments require a language call trait",
                            ));
                        }
                        Ok(Written::User {
                            path: key,
                            definition: id,
                        })
                    }
                    None => Err(self.error(key, "bound requires a trait declaration")),
                };
            }
            key.node = match self.kind(key) {
                Kind::Group(node) => node,
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => *self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements)
                    .first()
                    .ok_or_else(|| self.error(key, "empty trait bound"))?,
                _ => return Err(self.error(key, "bound requires a resolved trait")),
            };
        }
    }
    pub(in super::super) fn schedule_bounds(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let Some(bounds) = self.bound_syntax.remove(&owner) else {
            return Ok(());
        };
        self.work.push(Work::Bounds(owner, context));
        let mut subjects = crate::support::hash::HashSet::default();
        for bound in bounds.iter().rev() {
            let bound_context = bound.context(context);
            if let Written::User { path, definition } = bound.bound {
                self.bound_subject_lifetimes(path, owner)?;
                self.schedule_associated_bindings(path, bound_context)?;
                self.work
                    .push(Work::PathArguments(path, definition, bound_context));
            }
            if let Subject::Type(key) = bound.subject
                && subjects.insert(key)
            {
                self.bound_subject_lifetimes(key, owner)?;
                self.work
                    .push(Work::Node(key, false, bound.subject_context(context)));
            }
        }
        self.bound_syntax.insert(owner, bounds);
        Ok(())
    }
    pub(in super::super) fn bound_subject_lifetimes(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        let ast = &self.report.package.unit(key.file).configured.ast;
        let raw = matches!(
            self.report.resolution.definitions[owner.0 as usize].kind,
            crate::resolve::model::DefKind::Function
                | crate::resolve::model::DefKind::SelfType
                | crate::resolve::model::DefKind::Struct
                | crate::resolve::model::DefKind::Enum
        ) || self.facts.associated.trait_owner(owner).is_some()
            && self.report.resolution.definitions[owner.0 as usize].kind
                == crate::resolve::model::DefKind::TypeAlias;
        let mut work = vec![(key.node, true)];
        let mut children = Vec::new();
        let mut seen = crate::support::hash::HashSet::default();
        while let Some((node, mut explicit_lifetimes)) = work.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() + work.len() > 1_000_000 {
                return Err(self.error(key, "bound syntax work budget exceeded"));
            }
            let source = NodeKey { node, ..key };
            if raw {
                self.defer_bound_type(source);
            }
            match ast.node(node).kind {
                Kind::FunctionType(_)
                | Kind::Applied {
                    parenthesized: true,
                    ..
                } => {
                    if !raw {
                        continue;
                    }
                    // Callable signatures own lifetime elision, but their
                    // projections still wait for all written caller bounds.
                    explicit_lifetimes = false;
                }
                Kind::Type {
                    flavor: TypeFlavor::Ref | TypeFlavor::RefMut,
                    elements,
                } if explicit_lifetimes => {
                    let children = ast.list(elements);
                    let valid = children.len() == 2
                        && matches!(ast.node(children[0]).kind,Kind::Name(span) if self.report.package.unit(key.file).spelling(span).is_ok_and(|s|s!="'_"));
                    if !valid {
                        return Err(self.error(
                            NodeKey { node, ..key },
                            "bound references require explicit named lifetimes",
                        ));
                    }
                }
                _ => {}
            }
            ast.push_children(node, &mut children);
            work.extend(children.drain(..).map(|child| (child, explicit_lifetimes)));
        }
        Ok(())
    }
    pub(in super::super) fn complete_bounds(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let bounds = self.bound_syntax.remove(&owner).ok_or_else(|| {
            Error::at(crate::resolve::unit::FileId(0), 0, "missing pending bounds")
        })?;
        let mut completed = Vec::with_capacity(bounds.len());
        let mut input_count = 0;
        let work_start = self.work.len();
        let mut pending_bindings = false;
        for &bound in &bounds {
            let ty = match bound.subject {
                Subject::Parameter(id) | Subject::Associated(id) => self.facts.definitions
                    [id.0 as usize]
                    .ok_or_else(|| self.error(bound.source, "missing bounded parameter type"))?,
                Subject::Type(key) => self.ty(key)?,
            };
            let original_ty = match bound.subject {
                Subject::Type(key) => self.facts.unreduced.node(key).unwrap_or(ty),
                Subject::Parameter(_) | Subject::Associated(_) => ty,
            };
            let ty = self.bound_subject(bound.source, bound.scope, ty)?;
            let original_ty = self.bound_subject(bound.source, bound.scope, original_ty)?;
            let (value, original_bound) =
                self.bound_values(bound.bound, ty, original_ty, bound.context(context))?;
            if matches!(bound.subject, Subject::Parameter(_))
                && !matches!(self.facts.types.kind(ty), Some(Type::Parameter(_)))
            {
                return Err(self.error(bound.source, "trait bounds require a type parameter"));
            }
            let value = Predicate { ty, bound: value };
            let original = Predicate {
                ty: original_ty,
                bound: original_bound,
            };
            if original
                .contains_projections(&self.facts.types)
                .map_err(|e| self.error(bound.source, e))?
            {
                let original = self.bound_clause(bound.source, bound.scope, original)?;
                self.facts
                    .unreduced
                    .retain_predicate(bound.source, original);
            }
            completed.push(crate::sema::generics::Predicate {
                source: bound.source,
                value: self.bound_clause(bound.source, bound.scope, value)?,
            });
            if let Written::User { path, .. } = bound.bound {
                pending_bindings |= self.complete_associated_bindings(
                    super::bindings::Binding {
                        owner,
                        path,
                        predicate: value,
                        original,
                        scope: bound.scope,
                        context: bound.context(context),
                    },
                    &mut completed,
                )?;
            }
            if !matches!(bound.subject, Subject::Associated(_)) {
                input_count = completed.len();
            }
        }
        if pending_bindings {
            let arguments = self.work.split_off(work_start);
            self.work.push(Work::Bounds(owner, context));
            self.work.extend(arguments);
            self.bound_syntax.insert(owner, bounds);
            return Ok(());
        }
        if let Some(family) = self.facts.associated.families.get_mut(&owner) {
            // Associated result syntax is appended after the member's inputs.
            // Keep complete clauses, including equalities emitted by one bound.
            family.requirements = Some(completed[..input_count].iter().map(|p| p.value).collect());
        }
        self.facts.generics.predicates.insert(owner, completed);
        self.retain_raw_bounds(owner, bounds, context)?;
        Ok(())
    }

    pub(in crate::sema::declarations) fn bound_values(
        &mut self,
        written: Written,
        subject: crate::types::TypeId,
        original: crate::types::TypeId,
        context: regions::Context,
    ) -> Result<(Bound, Bound), Error> {
        let value = match written {
            Written::Trait(bound) => bound,
            Written::User { path, definition } => {
                let arguments = self.application_arguments(path, definition, context)?;
                let original_arguments = self.facts.unreduced.arguments(path).unwrap_or(arguments);
                let value = self.applied_trait(path, definition, subject, arguments)?;
                let original = if original != subject || original_arguments != arguments {
                    self.applied_trait(path, definition, original, original_arguments)?
                } else {
                    value
                };
                return Ok((Bound::Trait(value), Bound::Trait(original)));
            }
            Written::Outlives(key) => {
                let Kind::Name(span) = self.kind(key) else {
                    return Err(self.error(key, "outlives bound requires a named lifetime"));
                };
                if self
                    .report
                    .package
                    .unit(key.file)
                    .spelling(span)
                    .is_ok_and(|name| name == "'_")
                {
                    return Err(self.error(key, "outlives bound requires a named lifetime"));
                }
                Bound::Outlives(self.region(key, Some(key.node), context)?)
            }
        };
        Ok((value, value))
    }
}

impl Syntax {
    pub(in super::super) fn associated(
        source: NodeKey,
        owner: DefId,
        bound: Written,
        scope: Option<Scope>,
    ) -> Self {
        Self {
            source,
            subject: Subject::Associated(owner),
            bound,
            scope,
        }
    }
    pub(in super::super) fn context(self, parent: regions::Context) -> regions::Context {
        self.scope.map_or(parent, |scope| scope.context(parent))
    }
    pub(in super::super) fn subject_context(self, parent: regions::Context) -> regions::Context {
        self.scope
            .map_or(parent, |scope| scope.subject_context(parent))
    }
    pub(in super::super) fn trait_path(self) -> Option<(NodeKey, DefId)> {
        match self.bound {
            Written::User { path, definition } => Some((path, definition)),
            Written::Trait(_) | Written::Outlives(_) => None,
        }
    }
    pub(in super::super) fn subject(self) -> Option<NodeKey> {
        match self.subject {
            Subject::Type(key) => Some(key),
            Subject::Parameter(_) | Subject::Associated(_) => None,
        }
    }
}
