//! Inventory costs share the actual source owner; accounting grants no capability.
use super::{Draft, Inventory, Library, PublicBudget};
use crate::resolve::package::{Error, Package};

impl<'a> Draft<'a> {
    pub(in crate::resolve) fn for_package(package: &'a Package) -> Result<Self, Error> {
        if package
            .library_policy()
            .and_then(Library::inventory)
            .is_some()
        {
            return Err(Error::unlocated(
                "library stability inventory was already completed",
            ));
        }
        Ok(Self {
            payer: package.source_contract_payer().map_err(Error::unlocated)?,
            ..Self::default()
        })
    }

    pub(super) fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        if let Some(payer) = self.payer {
            // Attempted storage bounds the live draft conservatively. Completion
            // releases capture-only indexes and retains only published storage.
            return payer
                .inventory_charge(steps, bytes, self.bytes)
                .map_err(Error::unlocated);
        }
        if self.steps > 1_000_000 || self.bytes > 64 * 1024 * 1024 {
            return Err(Error::unlocated(
                "library stability inventory exceeds its work or storage budget",
            ));
        }
        Ok(())
    }

    pub(super) fn retain_inventory(
        mut self,
        library: &Library,
        inventory: &mut Inventory,
    ) -> Result<(), Error> {
        let Some(payer) = self.payer else {
            return Ok(());
        };
        self.charge(
            library
                .requests
                .len()
                .saturating_add(inventory.policies.len())
                .saturating_add(inventory.relationships.len()),
            0,
        )?;
        let mut retained = Retained::default();
        library.charge_inventory_retained(inventory, &mut retained)?;
        inventory.steps = self.steps;
        drop(self);
        payer
            .inventory_charge(0, 0, retained.bytes)
            .map_err(Error::unlocated)
    }
}

#[derive(Default)]
struct Retained {
    bytes: usize,
}
impl PublicBudget for Retained {
    fn charge(&mut self, _steps: usize, bytes: usize) -> Result<(), Error> {
        self.bytes = self.bytes.saturating_add(bytes);
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/inventory_budget.rs"]
mod tests;
