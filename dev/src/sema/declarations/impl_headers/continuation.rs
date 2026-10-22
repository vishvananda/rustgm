//! Header contexts survive declaration dependencies without recursive drains or replay.
use super::{Checker, DefId, Error, Partial, Work, regions};
use crate::{
    sema::associated::{Owner, OwnerId, OwnerKind},
    types::{
        Kind,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub(in crate::sema::declarations) enum Phase {
    Begin,
    Target,
    Publish,
    Arguments,
    Finish,
}

#[derive(Default)]
pub(in crate::sema::declarations) struct Pending {
    active: HashSet<DefId>,
    complete: HashSet<DefId>,
    partial: HashMap<DefId, Partial>,
    pub(in crate::sema::declarations) unsupported: Option<HashSet<OwnerId>>,
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn advance_impl_header(
        &mut self,
        id: DefId,
        phase: Phase,
    ) -> Result<(), Error> {
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
            })?;
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
                if trait_type.is_none()
                    && !matches!(
                        self.facts.types.kind(header.schema.self_type),
                        Some(Kind::Adt(..))
                    )
                {
                    return Err(self.error(target, "inherent impl requires a local nominal type"));
                }
                self.work.push(Work::ImplHeader(id, Phase::Arguments));
                self.publish_impl_owner(id, owner.source, header)?;
            }
            Phase::Arguments => {
                if let Some(path) = trait_type {
                    let (path, definition) = self.resolved_impl_trait_path(path)?;
                    self.work.push(Work::ImplHeader(id, Phase::Finish));
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
            Phase::Finish => {
                let path = trait_type
                    .ok_or_else(|| self.error(owner.source, "missing impl trait path"))?;
                self.publish_trait_header(
                    owner,
                    path,
                    self.ty(target)?,
                    self.impl_context(owner)?,
                )?;
                self.finish_header_state(id);
            }
        }
        Ok(())
    }

    fn begin_owner_header(&mut self, owner: Owner) -> Result<(), Error> {
        let id = owner.self_definition;
        if !self.impl_headers.active.insert(id) {
            return Err(self.error(owner.source, "cycle in implementation header dependencies"));
        }
        let OwnerKind::Impl {
            trait_type,
            unsafe_,
            negative,
            ..
        } = owner.kind
        else {
            return Err(self.error(owner.source, "impl header requires an implementation owner"));
        };
        if let Some(path) = trait_type {
            self.validate_trait_header(owner, path, unsafe_, negative)?;
        } else if unsafe_ || negative {
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
