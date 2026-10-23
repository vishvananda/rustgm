//! A cold generic attachment leaves every existing wire profile unchanged.
use super::{Interface, MAX_COMPONENTS, MAX_NAME_BYTES};
use crate::support::crates::generics;
impl Interface {
    /// Owned headers/boxes/strings count even when immutable. Canonical shared
    /// identity and type-graph Arc pointees retain the existing exclusion.
    pub(crate) fn base_retained_bytes(&self) -> usize {
        use crate::support::retained::{sum, vector};
        size_of::<Self>().saturating_add(sum([
            vector(&self.dependencies),
            vector(&self.functions),
            vector(&self.constants),
            self.functions
                .iter()
                .map(|function| size_of_val(function.signature().parameters()))
                .fold(0_usize, usize::saturating_add),
            self.nominals
                .as_ref()
                .map_or(0, crate::support::crates::Nominals::allocation_bytes),
            self.storage
                .as_ref()
                .map_or(0, crate::support::crates::Storage::allocation_bytes),
            self.public_names
                .as_deref()
                .map_or(0, crate::support::crates::PublicNames::allocation_bytes),
        ]))
    }
    pub(super) fn check_generic_names_storage(
        &self,
        names: &super::PublicNames,
        incoming: bool,
    ) -> Result<(), &'static str> {
        let Some(payload) = &self.generic_payload else {
            return Ok(());
        };
        // PublicNames::validate builds module/definition/dependency indexes and
        // one reached/pending frontier. Pay their conservative hash capacities
        // before validation can allocate them beside the generic originals.
        let modules = names.modules().len();
        let definitions =
            crate::support::crates::public_names::owned_public_definitions(self).count();
        let scratch = modules
            .saturating_mul(4 * size_of::<(&[String], usize)>())
            .saturating_add(modules.saturating_mul(size_of::<bool>() + size_of::<usize>()))
            .saturating_add(
                definitions.saturating_mul(8 * size_of::<&crate::support::crates::ExternalDef>()),
            )
            .saturating_add(
                self.dependencies
                    .len()
                    .saturating_mul(4 * size_of::<&crate::support::crates::CrateIdentity>()),
            );
        if self
            .base_retained_bytes()
            .saturating_add(payload.retained_bytes())
            .saturating_add(if incoming {
                names.allocation_bytes()
            } else {
                0
            })
            .saturating_add(scratch)
            > crate::support::crates::MAX_BYTES
        {
            return Err("generic public namespace exceeds combined storage budget");
        }
        Ok(())
    }
    /// Attach complete generic original data, without granting execution authority.
    /// # Errors
    /// Rejects foreign owners, duplicate publication and combined artifact quotas.
    pub fn with_generic_payload(
        mut self,
        payload: generics::Payload,
    ) -> Result<Self, &'static str> {
        if self.generic_payload.is_some() || payload.owners().owner() != &self.identity {
            return Err("generic payload belongs to another artifact or is already published");
        }
        if self
            .base_retained_bytes()
            .saturating_add(payload.peak_bytes())
            > crate::support::crates::MAX_BYTES
        {
            return Err("generic interface exceeds combined retained and scratch budget");
        }
        let (atoms, names) = payload.budget();
        self.atoms = self.atoms.saturating_add(atoms);
        self.names = self.names.saturating_add(names);
        if self.atoms > MAX_COMPONENTS || self.names > MAX_NAME_BYTES {
            return Err("generic interface exceeds combined artifact budget");
        }
        let mut functions = self
            .functions
            .iter()
            .map(|f| f.definition().path())
            .peekable();
        let mut constants = self
            .constants
            .iter()
            .map(|c| c.definition().path())
            .peekable();
        let mut statics = self
            .storage
            .as_ref()
            .into_iter()
            .flat_map(crate::support::crates::Storage::statics)
            .map(|s| s.definition().path())
            .peekable();
        for declaration in &payload.rows().declarations {
            let generics::Declaration::Function { identity, .. } = declaration else {
                continue;
            };
            if name_conflict(identity.path(), &mut functions)
                || name_conflict(identity.path(), &mut constants)
                || name_conflict(identity.path(), &mut statics)
            {
                return Err("generic declaration duplicates a closed interface name");
            }
        }
        self.generic_payload = Some(Box::new(payload));
        if let Some(names) = self.public_names.as_deref() {
            self.check_generic_names_storage(names, false)?;
            names.validate(&self)?;
        }
        Ok(self)
    }
    #[must_use]
    pub fn generic_payload(&self) -> Option<&generics::Payload> {
        self.generic_payload.as_deref()
    }
}

fn name_conflict<'a>(
    name: &[String],
    names: &mut std::iter::Peekable<impl Iterator<Item = &'a [String]>>,
) -> bool {
    while names.peek().is_some_and(|next| *next < name) {
        names.next();
    }
    names.peek().is_some_and(|next| *next == name)
}
