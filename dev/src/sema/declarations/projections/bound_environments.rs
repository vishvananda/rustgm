//! Typed bound subjects and arguments form assumptions before normalization.
use super::{Checker, DefId, Error, NodeKey, Work, regions};
use crate::sema::declarations::generics::bounds::Syntax;
use crate::support::hash::HashSet;
use crate::types::arguments::Argument;
use std::sync::Arc;

type Root = (NodeKey, regions::Context);
pub(super) struct Bounds {
    syntax: Vec<Syntax>,
    roots: Vec<(NodeKey, regions::Context)>,
}
impl Checker<'_> {
    pub(super) fn header_projection_context(&self, context: regions::Context) -> bool {
        if !self.header_names_active() {
            return false;
        }
        let mut binder = context.binder;
        while let Some(current) = binder {
            if let Some(owner) = self.regions.owner(current) {
                return self.report.resolution.definitions[owner.0 as usize].kind
                    == crate::resolve::model::DefKind::SelfType
                    && self.facts.associated.trait_owner(owner).is_some();
            }
            binder = self.regions.parent(current);
        }
        false
    }

    /// The complete written environment stays immutable. Formation proofs are
    /// demanded after implementation declarations, even if a result erases inputs.
    pub(in crate::sema::declarations) fn retain_header_projection_requirements(
        &mut self,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            let crate::sema::associated::OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            self.retain_bound_projection_requirements(definition, context)?;
        }
        Ok(())
    }
    /// Trait headers retain their written equations until all result contracts
    /// exist. The final normalization pass proves formation in the original binder.
    pub(in crate::sema::declarations) fn retain_bound_projection_requirements(
        &mut self,
        owner: DefId,
        _context: regions::Context,
    ) -> Result<(), Error> {
        if let Some(bounds) = self.projections.raw_bounds.remove(&owner) {
            for (key, context) in bounds.roots {
                let ty = self.ty(key)?;
                self.projections
                    .deferred
                    .push((key, context, super::scoped::Ambient::Root, ty));
            }
        }
        Ok(())
    }

    /// All sibling result contracts exist before member bounds are validated.
    /// Prove their retained roots at the written type before checking the enclosing
    /// predicate, preserving both projection debts and source diagnostics.
    pub(in crate::sema::declarations) fn validate_bound_projection_requirements(
        &mut self,
        owner: DefId,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let Some(bounds) = self.projections.raw_bounds.remove(&owner) else {
            return Ok(());
        };
        for (key, context) in bounds.roots {
            self.start_scoped_normalization(
                key,
                super::Root {
                    ty: self.ty(key)?,
                    normalized_subject: None,
                },
                context,
                super::Destination::Requirement,
                super::scoped::Ambient::Root,
            )?;
            self.drain(steps, limit)?;
        }
        Ok(())
    }
    pub(in super::super) fn defer_bound_type(&mut self, key: NodeKey) {
        use crate::resolve::model::{DefKind, Target};
        let application = self
            .facts
            .node(key)
            .and_then(|facts| facts.reference)
            .is_some_and(|target| {
                matches!(target, Target::Definition(id) if matches!(
                    self.report.resolution.definitions[id.0 as usize].kind,
                    DefKind::Struct | DefKind::Enum | DefKind::TypeAlias | DefKind::SelfType
                ))
            });
        if application
            || self.report.resolution.qualified_paths.contains_key(&key)
            || self
                .report
                .resolution
                .associated_paths
                .get(&key)
                .is_some_and(|path| path.namespace == crate::resolve::model::Namespace::Type)
        {
            self.projections.raw_bound_nodes.insert(key);
        }
    }
    pub(in super::super) fn bound_environment_pending(&self, owner: DefId) -> bool {
        self.projections.raw_bounds.contains_key(&owner)
    }
    pub(in super::super) fn retain_raw_bounds(
        &mut self,
        owner: DefId,
        syntax: Vec<Syntax>,
        context: regions::Context,
    ) -> Result<(), Error> {
        let mut roots = Vec::new();
        let mut seen = HashSet::default();
        for &bound in &syntax {
            self.collect_raw_bound(bound, context, false, &mut seen, &mut roots)?;
        }
        let staged = self.take_projected_roots(owner, crate::support::retained::vector(&syntax))?;
        let completed = staged.is_some();
        if let Some(mut staged) = staged {
            let growth = crate::support::retained::Growth::vector::<Root>(
                roots.len(),
                roots.capacity(),
                staged.len(),
            );
            self.charge_bound_staging(growth.work.saturating_add(staged.len()), growth.bytes)?;
            roots.append(&mut staged);
        }
        if !roots.is_empty() || completed {
            if completed {
                let table = &self.projections.raw_bounds;
                let growth = crate::support::retained::Growth::map::<DefId, Bounds>(
                    table.len(),
                    table.capacity(),
                    usize::from(!table.contains_key(&owner)),
                );
                self.charge_bound_staging(growth.work.saturating_add(1), growth.bytes)?;
            }
            self.projections
                .raw_bounds
                .insert(owner, Bounds { syntax, roots });
        }
        Ok(())
    }

    fn collect_raw_bound(
        &self,
        bound: Syntax,
        context: regions::Context,
        projected: bool,
        seen: &mut HashSet<(NodeKey, regions::Context)>,
        roots: &mut Vec<(NodeKey, regions::Context)>,
    ) -> Result<(), Error> {
        if !projected && let Some(key) = bound.subject() {
            self.retain_bound_root(key, bound.subject_context(context), seen, roots)?;
        }
        if let Some((path, definition)) = bound.trait_path() {
            if let Some((inputs, output)) =
                crate::sema::lifetimes::call_arguments(self.report, path)?
            {
                let ast = &self.report.package.unit(path.file).configured.ast;
                for node in ast.list(inputs).iter().copied().chain(output) {
                    self.retain_bound_root(
                        NodeKey { node, ..path },
                        bound.context(context),
                        seen,
                        roots,
                    )?;
                }
                return Ok(());
            }
            if let Some(list) = self.associated_bindings.bindings(path) {
                for offset in list.start..list.end {
                    let node =
                        self.report.package.unit(path.file).configured.ast.edges[offset as usize];
                    let source = NodeKey { node, ..path };
                    if self
                        .facts
                        .node(source)
                        .is_some_and(|facts| facts.ty.is_some())
                    {
                        self.retain_bound_root(source, bound.context(context), seen, roots)?;
                    }
                    if let Some(value) = self.associated_binding_rhs(source)? {
                        self.retain_bound_root(
                            NodeKey {
                                node: value,
                                ..path
                            },
                            bound.context(context),
                            seen,
                            roots,
                        )?;
                    }
                }
            }
            let (list, omitted) = self.supplied_arguments(path, definition)?;
            if let Some(list) = list {
                let ast = &self.report.package.unit(path.file).configured.ast;
                for (&node, parameter) in ast.list(list).iter().zip(
                    self.facts.generics.parameters[&definition]
                        .iter()
                        .skip(omitted),
                ) {
                    if matches!(parameter.argument, Argument::Type(_)) {
                        self.retain_bound_root(
                            NodeKey { node, ..path },
                            bound.context(context),
                            seen,
                            roots,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn retain_projected_raw_bounds(
        &mut self,
        owner: DefId,
        syntax: &[Syntax],
        context: regions::Context,
    ) -> Result<(), Error> {
        let mut maximum = 0_usize;
        for &bound in syntax {
            if let Some((path, definition)) = bound.trait_path() {
                if let Some((inputs, output)) =
                    crate::sema::lifetimes::call_arguments(self.report, path)?
                {
                    maximum = maximum
                        .saturating_add((inputs.end - inputs.start) as usize)
                        .saturating_add(usize::from(output.is_some()));
                } else {
                    let (arguments, _) = self.supplied_arguments(path, definition)?;
                    maximum =
                        maximum.saturating_add(arguments.map_or(0, |a| (a.end - a.start) as usize));
                    if let Some(bindings) = self.associated_bindings.bindings(path) {
                        maximum = maximum.saturating_add(
                            ((bindings.end - bindings.start) as usize).saturating_mul(2),
                        );
                    }
                }
            }
        }
        let bytes = maximum
            .saturating_mul(size_of::<Root>())
            .saturating_add(crate::support::retained::new_map::<Root, ()>(maximum));
        self.charge_bound_staging(maximum.saturating_add(syntax.len()), bytes)?;
        let mut roots = Vec::with_capacity(maximum);
        let mut seen = HashSet::with_capacity_and_hasher(
            maximum,
            crate::support::hash::BuildFxHasher::default(),
        );
        for &bound in syntax {
            self.collect_raw_bound(bound, context, true, &mut seen, &mut roots)?;
        }
        if roots.is_empty() {
            return Ok(());
        }
        self.stage_projected_roots(owner, &mut roots)?;
        Ok(())
    }

    pub(in crate::sema::declarations) fn append_completed_bound_roots(
        &mut self,
        owner: DefId,
        roots: &mut Vec<Root>,
    ) -> Result<(), Error> {
        let prior = self.projections.raw_bounds.get(&owner).ok_or_else(|| {
            Error::unlocated("projected roots follow their completed owner phase")
        })?;
        let growth = crate::support::retained::Growth::vector::<Root>(
            prior.roots.len(),
            prior.roots.capacity(),
            roots.len(),
        );
        self.charge_bound_staging(growth.work.saturating_add(roots.len()), growth.bytes)?;
        self.projections
            .raw_bounds
            .get_mut(&owner)
            .ok_or_else(|| Error::unlocated("projected roots lost their completed owner"))?
            .roots
            .append(roots);
        Ok(())
    }

    fn retain_bound_root(
        &self,
        key: NodeKey,
        context: regions::Context,
        seen: &mut HashSet<(NodeKey, regions::Context)>,
        roots: &mut Vec<(NodeKey, regions::Context)>,
    ) -> Result<(), Error> {
        if self.facts.types.contains_projections(self.ty(key)?) == Some(true)
            && seen.insert((key, context))
        {
            roots.push((key, context));
        }
        Ok(())
    }

    pub(in super::super) fn prepare_bound_environment(
        &mut self,
        owner: DefId,
        _context: regions::Context,
        continuation: Work,
    ) -> Result<bool, Error> {
        if self.projections.raw_environments.contains_key(&owner) {
            return Ok(false);
        }
        let Some(bounds) = self.projections.raw_bounds.get(&owner) else {
            return Ok(false);
        };
        let Some(&(key, _)) = bounds.roots.first() else {
            return Ok(false);
        };
        let roots = bounds.roots.clone();
        let environment = self.elaborated_source_environment(key, owner)?;
        self.projections
            .raw_environments
            .insert(owner, Arc::new(environment));
        self.work.push(continuation);
        for (key, context) in roots.into_iter().rev() {
            self.work.push(Work::NormalizeBound(key, context));
        }
        Ok(true)
    }

    pub(in super::super) fn complete_bound_environment(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if let Some(bounds) = self.projections.raw_bounds.remove(&owner) {
            self.bound_syntax.insert(owner, bounds.syntax);
            self.complete_bounds(owner, context)?;
            // Successfully proved opaque roots remain projections. Their rebuilt
            // predicates must not schedule this completed normalization again.
            self.projections.raw_bounds.remove(&owner);
        }
        self.projections.raw_environments.remove(&owner);
        Ok(())
    }
}
