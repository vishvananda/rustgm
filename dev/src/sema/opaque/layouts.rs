//! Declaration layout obligations resume only after authentic defining proofs.
use crate::{
    abi::templates::Templates,
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::{
        hash::HashSet,
        retained::{Growth, Payment},
    },
    types::{ItemId, TypeId, opaque::OpaqueId},
};

#[derive(Default)]
pub(crate) struct Pending {
    roots: Vec<(ItemId, TypeId, OpaqueId)>,
    pub(crate) dependencies: Vec<OpaqueId>,
}

impl Pending {
    #[cfg(test)]
    pub(crate) fn last_root(&self) -> Option<TypeId> {
        self.roots.last().map(|&(_, ty, _)| ty)
    }

    pub(crate) fn push(
        &mut self,
        item: ItemId,
        ty: TypeId,
        id: OpaqueId,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        if self.roots.len() >= 1_000_000 {
            return Err("pending nominal layout budget exceeded");
        }
        let growth = Growth::vector::<(ItemId, TypeId, OpaqueId)>(
            self.roots.len(),
            self.roots.capacity(),
            1,
        );
        pay(1 + growth.work, growth.bytes)?;
        self.roots
            .try_reserve(1)
            .map_err(|_| "pending nominal layout allocation failed")?;
        self.roots.push((item, ty, id));
        Ok(())
    }
}

impl Declarations {
    pub(crate) fn retry_pending_layouts(
        &mut self,
        report: &Report,
        mut close: impl FnMut(&mut Self, OpaqueId) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let Some(named) = self.opaques.named.as_deref_mut() else {
            return Ok(());
        };
        if named.pending_layouts.roots.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut named.pending_layouts);
        let mut closed = HashSet::default();
        for id in pending.dependencies {
            if closed.contains(&id) {
                report
                    .package
                    .pay_named_opaque(1, 0)
                    .map_err(Error::unlocated)?;
                continue;
            }
            let growth = Growth::map::<OpaqueId, ()>(closed.len(), closed.capacity(), 1);
            report
                .package
                .pay_named_opaque(1 + growth.work, growth.bytes)
                .map_err(Error::unlocated)?;
            closed
                .try_reserve(1)
                .map_err(|_| Error::unlocated("pending opaque proof allocation failed"))?;
            closed.insert(id);
            if !self.is_named_opaque(id) {
                return Err(Error::unlocated(
                    "pending layout has no authorized opaque definer",
                ));
            }
            close(self, id)?;
        }
        self.prepare_opaque_storage_located().map_err(|failure| {
            let source = failure.owner.and_then(|owner| {
                let entry = report.resolution.definitions.get(owner.0 as usize)?;
                entry.node
            });
            report.body_optional_error(source.map(Into::into), failure.message)
        })?;
        let mut templates = Templates::default();
        for (item, ty, id) in pending.roots {
            let source = report
                .resolution
                .definitions
                .get(item.0 as usize)
                .and_then(|entry| entry.node);
            let error = |message| report.body_optional_error(source.map(Into::into), message);
            if !closed.contains(&id) {
                return Err(error("pending nominal layout lost its defining proof"));
            }
            let before = templates.steps;
            templates
                .declaration(&mut self.types, &mut self.aggregates, &mut self.layouts, ty)
                .map_err(|failure| error(failure.message))?;
            report
                .package
                .pay_named_opaque(1 + templates.steps.saturating_sub(before), 0)
                .map_err(error)?;
        }
        Ok(())
    }
}
