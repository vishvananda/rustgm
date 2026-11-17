//! Sparse crate-owned selectors stage before their actual module lookup commits.
use super::{Context, Error, Package, Prelude, Resolution, pay};
use crate::{
    resolve::{guests::CrateSlot, model::Introductions, package::ModuleId},
    support::{hash::HashMap, retained::Growth},
};

#[derive(Default)]
pub(in crate::resolve) struct Preludes {
    entries: HashMap<CrateSlot, Entry>,
}
#[derive(Default)]
struct Entry {
    selected: Option<Prelude>,
    pending: Option<Prelude>,
}
impl Preludes {
    pub(super) fn has_any(&self) -> bool {
        !self.entries.is_empty()
    }
    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.entries.capacity()
    }
    pub(super) fn selected(&self, slot: CrateSlot) -> Option<&Prelude> {
        self.entries.get(&slot)?.selected.as_ref()
    }
    #[cfg(test)]
    pub(super) fn selected_mut(&mut self, slot: CrateSlot) -> Option<&mut Prelude> {
        self.entries.get_mut(&slot)?.selected.as_mut()
    }
    #[cfg(test)]
    pub(super) fn take_selected(&mut self, slot: CrateSlot) -> Option<Prelude> {
        self.entries.get_mut(&slot)?.selected.take()
    }
    pub(super) fn pending(&self, slot: CrateSlot) -> Option<&Prelude> {
        self.entries.get(&slot)?.pending.as_ref()
    }
    pub(super) fn validate(
        &self,
        package: &Package,
        facts: &Resolution,
    ) -> Result<(), &'static str> {
        // Hash-table iteration also visits empty buckets in the retained allocation.
        pay(package, self.entries.capacity(), 0)?;
        for (&slot, entry) in &self.entries {
            if entry.pending.is_some() {
                return Err("source prelude selection is unfinished");
            }
            if let Some(prelude) = &entry.selected {
                if crate_slot(package, prelude.context.module)? != slot
                    || package.file_crate(prelude.owner.file) != slot
                {
                    return Err("source prelude belongs to another crate");
                }
                prelude.validate(package, facts)?;
            }
        }
        Ok(())
    }
    pub(super) fn commit(
        &mut self,
        slot: CrateSlot,
        context: Context,
        selected: (super::DefId, super::ScopeId),
    ) -> Result<(), &'static str> {
        let entry = self
            .entries
            .get_mut(&slot)
            .ok_or("source prelude crate disappeared")?;
        let pending = entry
            .pending
            .as_mut()
            .ok_or("source prelude selector executed twice")?;
        if pending.context.scope != context.scope || pending.context.module != context.module {
            return Err("source prelude actual use scope changed");
        }
        pending.selected = Some(selected);
        entry.selected = entry.pending.take();
        Ok(())
    }
}

pub(super) fn pending_for_use<'a>(
    package: &Package,
    facts: &'a Resolution,
    context: Context,
    tree: super::NodeId,
) -> Result<Option<(CrateSlot, &'a Prelude)>, Error> {
    let slot = crate_slot(package, context.module).map_err(Error::unlocated)?;
    let state = facts
        .introductions
        .item_scopes
        .as_deref()
        .and_then(|s| s.prelude.as_deref());
    let matches = |prelude: &Prelude| {
        prelude.item.kind == crate::syntax::item::ItemKind::Use(tree)
            && prelude.item_key.file == context.file
    };
    let Some(prelude) = state.and_then(|state| state.pending(slot)) else {
        if state
            .and_then(|state| state.selected(slot))
            .is_some_and(matches)
        {
            return Err(Error::at(
                context.file,
                0,
                "source prelude selector executed twice",
            ));
        }
        return Ok(None);
    };
    Ok(matches(prelude).then_some((slot, prelude)))
}

pub(super) fn crate_slot(package: &Package, module: ModuleId) -> Result<CrateSlot, &'static str> {
    package
        .modules
        .get(module.0 as usize)
        .map(|row| row.krate)
        .ok_or("source prelude lookup module is absent")
}

pub(super) fn stage(
    package: &Package,
    introductions: &mut Introductions,
    prelude: Prelude,
) -> Result<(), Error> {
    let error = |message| Error::at(prelude.owner.file, prelude.attribute.span.start, message);
    let slot = crate_slot(package, prelude.context.module).map_err(error)?;
    if package.file_crate(prelude.owner.file) != slot {
        return Err(error("source prelude owner belongs to another crate"));
    }
    let old = introductions
        .item_scopes
        .as_deref()
        .and_then(|s| s.prelude.as_deref());
    let entry = old.and_then(|state| state.entries.get(&slot));
    if entry.is_some_and(|entry| {
        entry.pending.is_some()
            || entry
                .selected
                .as_ref()
                .is_some_and(|old| old.owner == prelude.owner && old.effect == prelude.effect)
    }) {
        return Err(error("source prelude selector executed twice"));
    }
    let (length, capacity) = old.map_or((0, 0), |state| {
        (state.entries.len(), state.entries.capacity())
    });
    let growth = Growth::map::<CrateSlot, Entry>(length, capacity, usize::from(entry.is_none()));
    let bytes = growth
        .bytes
        .saturating_add(if old.is_none() {
            size_of::<Preludes>()
        } else {
            0
        })
        .saturating_add(if introductions.item_scopes.is_none() {
            size_of::<crate::resolve::container_scopes::ItemScopes>()
        } else {
            0
        });
    pay(package, growth.work.saturating_add(1), bytes).map_err(error)?;
    let state = introductions
        .item_scopes
        .get_or_insert_with(Box::default)
        .prelude
        .get_or_insert_with(Box::default);
    state
        .entries
        .try_reserve(usize::from(!state.entries.contains_key(&slot)))
        .map_err(|_| error("source prelude owner allocation failed"))?;
    state.entries.entry(slot).or_default().pending = Some(prelude);
    Ok(())
}

pub(super) fn require_own_request(package: &Package, context: Context) -> Result<(), Error> {
    if package.source_core().is_none() {
        return Ok(());
    }
    let root = package.file_crate_root(context.file);
    let module = package
        .modules
        .get(root.0 as usize)
        .ok_or_else(|| Error::at(context.file, 0, "source prelude request has no crate root"))?;
    let unit = package.unit(module.file);
    let attributes = unit.owner_indices[module.root.get() as usize].map(|index| {
        unit.configured.owners[index.get() as usize - 1]
            .attributes
            .clone()
    });
    if let Some(attributes) = attributes {
        // Prepay attribute visits before measuring their extent. That exact
        // extent pays the existing admission grammar's name/token traversal.
        pay(package, attributes.len(), 0).map_err(Error::unlocated)?;
        let width = unit
            .retained_attributes(attributes)
            .fold(0usize, |sum, (attribute, _)| {
                sum.saturating_add(
                    attribute.span.end.saturating_sub(attribute.span.start) as usize + 1,
                )
            });
        pay(package, width, 0).map_err(Error::unlocated)?;
    }
    if !crate::resolve::bindings::policy::root_feature(
        unit,
        module.file,
        crate::resolve::bindings::policy::Feature::PreludeImport,
    )? {
        return Err(Error::at(
            context.file,
            0,
            "source prelude requires its own crate prelude_import request",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/prelude_replacements.rs"]
mod tests;
