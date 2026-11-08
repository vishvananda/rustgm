//! Header contexts survive declaration dependencies without recursive drains or replay.
#[cfg(test)]
#[path = "../../../../tests/invariants/defaulted_impl_arguments.rs"]
mod tests;
use super::{Checker, DefId, Error, NodeKey, Partial, Work, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::associated::{OwnerId, OwnerKind, SourceOwner as Owner},
    types::{
        TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

#[cfg(test)]
thread_local! {
    /// Test-only: the defaulted impl trait reference arguments normalized.
    static DEFAULTED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Clone, Copy)]
pub(in crate::sema::declarations) enum Phase {
    Begin,
    Target,
    Publish,
    Arguments,
    /// Record the trait reference once its path arguments are lowered.
    Reference,
    /// Replace the reference's defaulted arguments by their normal forms.
    Defaults,
    Finish,
}

/// A trait reference argument left to its default: the default as the path
/// instantiated it, and the normal form the annotation routine completes.
struct Defaulted {
    original: TypeId,
    normal: Option<TypeId>,
}

#[derive(Default)]
pub(in crate::sema::declarations) struct Pending {
    active: HashSet<DefId>,
    complete: HashSet<DefId>,
    partial: HashMap<DefId, Partial>,
    /// Trait references, active (`None`) or complete, demanded before Finish.
    references: HashMap<DefId, Option<Predicate>>,
    /// References whose defaulted arguments are still normalizing; they stay
    /// active until `Defaults`, as a reference does while its written ones do.
    unnormalized: HashMap<DefId, Predicate>,
    /// Defaulted arguments by impl and argument index.
    defaulted: HashMap<(DefId, u16), Defaulted>,
    pub(in crate::sema::declarations) unsupported: Option<HashSet<OwnerId>>,
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn advance_impl_header(
        &mut self,
        id: DefId,
        phase: Phase,
    ) -> Result<(), Error> {
        if self.activate_library_impl_header(id)? {
            return Ok(());
        }
        if self.impl_headers.complete.contains(&id) {
            return Ok(());
        }
        let owner = self
            .facts
            .associated
            .owner(id)
            .and_then(|owner| self.facts.associated.get(owner))
            .copied()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing impl header owner",
                )
            })?
            .source_owner()
            .map_err(Error::unlocated)?;
        let OwnerKind::Impl {
            target, trait_type, ..
        } = owner.kind
        else {
            return Err(self.error(owner.source, "impl header requires an implementation owner"));
        };
        match phase {
            Phase::Begin => self.begin_owner_header(owner)?,
            Phase::Target => {
                let context = self
                    .impl_headers
                    .partial
                    .get(&id)
                    .ok_or_else(|| self.error(owner.source, "missing pending impl parameters"))?
                    .context;
                self.work.push(Work::ImplHeader(id, Phase::Publish));
                self.node_work(target.file, target.node, context);
            }
            Phase::Publish => {
                let partial = self
                    .impl_headers
                    .partial
                    .remove(&id)
                    .ok_or_else(|| self.error(owner.source, "missing pending impl target"))?;
                let header = self.finish_impl_header(target, partial)?;
                if trait_type.is_none() {
                    self.validate_inherent_target(id, target, header.schema.self_type)?;
                }
                self.work.push(Work::ImplHeader(id, Phase::Arguments));
                self.publish_impl_owner(id, owner.source, header)?;
            }
            Phase::Arguments => {
                if let Some(path) = trait_type {
                    self.work.push(Work::ImplHeader(id, Phase::Finish));
                    if !self.impl_headers.references.contains_key(&id) {
                        self.begin_trait_reference(owner, path)?;
                    }
                } else {
                    self.impl_parameters(
                        id,
                        owner.source,
                        Predicate {
                            ty: self.ty(target)?,
                            bound: Bound::Sized,
                        },
                    )?;
                    self.publish_inherent_members(id, owner.source)?;
                    self.finish_header_state(id);
                }
            }
            Phase::Reference => {
                let path = trait_type
                    .ok_or_else(|| self.error(owner.source, "missing impl trait path"))?;
                let context = self.impl_context(owner)?;
                let reference =
                    self.impl_trait_reference(owner, path, self.ty(target)?, context)?;
                self.normalize_defaulted_arguments(id, path, reference, context)?;
            }
            Phase::Defaults => {
                let reference = self.impl_headers.unnormalized.remove(&id).ok_or_else(|| {
                    self.error(owner.source, "missing defaulted impl trait reference")
                })?;
                let reference = self.normalized_reference(id, owner.source, reference)?;
                self.impl_headers.references.insert(id, Some(reference));
            }
            Phase::Finish => {
                let path = trait_type
                    .ok_or_else(|| self.error(owner.source, "missing impl trait path"))?;
                let reference = self
                    .impl_headers
                    .references
                    .remove(&id)
                    .flatten()
                    .ok_or_else(|| self.error(owner.source, "missing impl trait reference"))?;
                self.publish_trait_header(owner, path, reference, self.impl_context(owner)?)?;
                self.finish_header_state(id);
            }
        }
        Ok(())
    }

    /// Lower the trait path arguments once; `Self::` shorthand in the impl's
    /// where-clauses may demand this before the `Arguments` phase.
    fn begin_trait_reference(
        &mut self,
        owner: Owner,
        path: crate::resolve::unit::NodeKey,
    ) -> Result<(), Error> {
        let id = owner.self_definition;
        if self.impl_headers.references.insert(id, None).is_some() {
            return Err(self.error(owner.source, "impl trait reference began twice"));
        }
        let (path, definition) = self.resolved_impl_trait_path(path)?;
        self.work.push(Work::ImplHeader(id, Phase::Reference));
        if matches!(
            self.impl_trait_identity(definition),
            crate::sema::traits::implementations::Identity::User(_)
        ) {
            self.work.push(Work::PathArguments(
                path,
                definition,
                self.impl_context(owner)?,
            ));
        }
        Ok(())
    }

    /// The trait reference that `Self::` shorthand reads in an impl whose
    /// header is not yet published. `Ok(None)` means it was demanded now and the
    /// shorthand resumes after it; an active reference is a genuine cycle.
    pub(in crate::sema::declarations) fn demanded_trait_reference(
        &mut self,
        id: DefId,
        source: crate::resolve::unit::NodeKey,
        resume: Work,
    ) -> Result<Option<Predicate>, Error> {
        match self.impl_headers.references.get(&id) {
            Some(Some(reference)) => return Ok(Some(*reference)),
            Some(None) => {
                return Err(self.error(source, "cycle in implementation header dependencies"));
            }
            None => {}
        }
        let owner = self
            .facts
            .associated
            .owner(id)
            .and_then(|owner| self.facts.associated.get(owner))
            .copied()
            .ok_or_else(|| self.error(source, "missing impl header owner"))?
            .source_owner()
            .map_err(Error::unlocated)?;
        let OwnerKind::Impl {
            trait_type: Some(path),
            ..
        } = owner.kind
        else {
            return Err(self.error(
                source,
                "impl trait reference requires a trait implementation",
            ));
        };
        if !self.impl_headers.active.contains(&id) || self.impl_headers.partial.contains_key(&id) {
            return Err(self.error(
                source,
                "shorthand Self associated type requires a prepared trait owner",
            ));
        }
        self.work.push(resume);
        self.begin_trait_reference(owner, path)?;
        Ok(None)
    }

    /// Normalize each argument the trait path leaves to a default that still
    /// names a projection, by the routine a written argument uses, keyed by
    /// (impl, argument index); `site` only locates its diagnostics.
    fn normalize_defaulted_arguments(
        &mut self,
        id: DefId,
        path: NodeKey,
        reference: Predicate,
        context: regions::Context,
    ) -> Result<(), Error> {
        let (site, definition) = self.resolved_impl_trait_path(path)?;
        let defaulted = self.defaulted_projections(site, definition, reference)?;
        if defaulted.is_empty() {
            self.impl_headers.references.insert(id, Some(reference));
            return Ok(());
        }
        self.impl_headers.unnormalized.insert(id, reference);
        self.work.push(Work::ImplHeader(id, Phase::Defaults));
        for (index, original) in defaulted {
            let entry = Defaulted {
                original,
                normal: None,
            };
            if self
                .impl_headers
                .defaulted
                .insert((id, index), entry)
                .is_some()
            {
                return Err(self.error(site, "defaulted impl trait argument began twice"));
            }
            #[cfg(test)]
            DEFAULTED.with(|count| count.set(count.get() + 1));
            let root = super::super::projections::RootKey::Argument {
                implementation: id,
                index,
                site,
            };
            self.normalize_root(root, original, context)?;
        }
        Ok(())
    }

    /// The type arguments after the written ones that still name projections.
    fn defaulted_projections(
        &self,
        path: NodeKey,
        definition: DefId,
        reference: Predicate,
    ) -> Result<Vec<(u16, TypeId)>, Error> {
        let Some((application, _)) = reference.bound.trait_parts() else {
            return Ok(Vec::new());
        };
        if crate::sema::lifetimes::call_arguments(self.report, path)?.is_some() {
            return Ok(Vec::new());
        }
        let (list, omitted) = self.supplied_arguments(path, definition)?;
        let ast = &self.report.package.unit(path.file).configured.ast;
        let written = omitted + list.map_or(0, |list| ast.list(list).len());
        let types = &self.facts.types;
        let arguments = types
            .trait_application(application)
            .and_then(|application| types.arguments(application.arguments))
            .ok_or_else(|| self.error(path, "missing impl trait reference arguments"))?;
        let mut defaulted = Vec::new();
        for (index, argument) in arguments.enumerate().skip(written) {
            if let Argument::Type(ty) = argument
                && types.contains_projections(ty) == Some(true)
            {
                let index = u16::try_from(index)
                    .map_err(|_| self.error(path, "too many impl trait reference arguments"))?;
                defaulted.push((index, ty));
            }
        }
        Ok(defaulted)
    }

    /// The reference with each defaulted argument replaced by its normal form.
    fn normalized_reference(
        &mut self,
        id: DefId,
        source: NodeKey,
        reference: Predicate,
    ) -> Result<Predicate, Error> {
        let (application, constness) = reference
            .bound
            .trait_parts()
            .ok_or_else(|| self.error(source, "defaulted impl arguments require a trait"))?;
        let mut application = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(source, "missing impl trait application"))?;
        let mut arguments = self
            .facts
            .types
            .arguments(application.arguments)
            .ok_or_else(|| self.error(source, "missing impl trait reference arguments"))?
            .collect::<Vec<_>>();
        for (index, argument) in arguments.iter_mut().enumerate() {
            let Some(entry) = u16::try_from(index)
                .ok()
                .and_then(|index| self.impl_headers.defaulted.get(&(id, index)))
            else {
                continue;
            };
            match (*argument, entry.normal) {
                (Argument::Type(ty), Some(normal)) if ty == entry.original => {
                    *argument = Argument::Type(normal);
                }
                _ => {
                    return Err(
                        self.error(source, "defaulted impl trait argument has no normal form")
                    );
                }
            }
        }
        application.arguments = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(application)
            .map_err(|e| self.error(source, e))?;
        Ok(Predicate {
            ty: reference.ty,
            bound: Bound::trait_application(application, constness),
        })
    }

    fn activate_library_impl_header(&mut self, id: DefId) -> Result<bool, Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .cloning()
            .and_then(|c| c.implementation)
            .is_some_and(|i| i.definition == id)
        {
            self.activate_iterator_clone(id)?;
            return Ok(true);
        }
        if super::super::iteration::range_contracts::owns(self.report, id) {
            self.activate_range(id)?;
            return Ok(true);
        }
        if super::super::iteration::collections::owns(self.report, id) {
            self.activate_collection(id)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn begin_owner_header(&mut self, owner: Owner) -> Result<(), Error> {
        let id = owner.self_definition;
        if !self.impl_headers.active.insert(id) {
            return Err(self.error(owner.source, "cycle in implementation header dependencies"));
        }
        let OwnerKind::Impl {
            trait_type,
            unsafe_,
            polarity,
            ..
        } = owner.kind
        else {
            return Err(self.error(owner.source, "impl header requires an implementation owner"));
        };
        if let Some(path) = trait_type {
            self.validate_trait_header(
                owner,
                path,
                unsafe_,
                polarity == crate::sema::associated::ImplPolarity::Negative,
            )?;
        } else if unsafe_ || polarity != crate::sema::associated::ImplPolarity::Positive {
            return Err(self.error(owner.source, "inherent impls cannot be unsafe or negative"));
        }
        self.work.push(Work::ImplHeader(id, Phase::Target));
        let partial = self.begin_impl_parameters(id, owner.source)?;
        self.impl_headers.partial.insert(id, partial);
        Ok(())
    }

    fn impl_context(&self, owner: Owner) -> Result<regions::Context, Error> {
        let binder = *self
            .owner_binders
            .get(&owner.self_definition)
            .ok_or_else(|| self.error(owner.source, "missing impl header binder"))?;
        Ok(regions::Context {
            binder: Some(binder),
            position: regions::Position::Impl,
        })
    }

    pub(in crate::sema::declarations) fn finish_impl_regions(
        &mut self,
        owner: Owner,
        context: regions::Context,
    ) -> Result<(), Error> {
        let definition = owner.self_definition;
        let binder = self
            .owner_binders
            .get(&definition)
            .copied()
            .ok_or_else(|| self.error(owner.source, "missing pending impl region binder"))?;
        if !self.impl_headers.active.contains(&definition)
            || self.impl_headers.complete.contains(&definition)
            || context.binder != Some(binder)
            || self.regions.owner(binder) != Some(definition)
        {
            return Err(self.error(
                owner.source,
                "impl region completion requires its active unfinished header",
            ));
        }
        self.facts
            .finish_impl_regions(definition, self.regions.count(binder))
            .map_err(|e| self.error(owner.source, e))
    }

    fn finish_header_state(&mut self, id: DefId) {
        self.impl_headers.active.remove(&id);
        self.impl_headers.complete.insert(id);
    }
}

impl Pending {
    /// A defaulted argument's normal form, completed once beside its default.
    pub(in crate::sema::declarations) fn complete_argument(
        &mut self,
        implementation: DefId,
        index: u16,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        let entry = self
            .defaulted
            .get_mut(&(implementation, index))
            .ok_or("missing defaulted impl trait argument")?;
        if entry.normal.replace(ty).is_some() {
            return Err("defaulted impl trait argument normalized twice");
        }
        Ok(())
    }
}
