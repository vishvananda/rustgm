//! A local allowance owner is distinct from a source-provider role inventory.
use super::{Source, SourceCore, panic_contracts, policy};
use crate::{
    resolve::{
        package::{Error, Package},
        unit::{FileId, NodeKey, Unit},
    },
    support::source::OriginalImage,
};
use std::sync::Arc;

#[cfg(test)]
#[path = "../../../tests/invariants/local_macro_context.rs"]
mod tests;

pub(in crate::resolve) struct LocalMacros {
    root: NodeKey,
    /// The ledger exists only for imported allowance lists, so the crate root
    /// need not carry an attribute owner.
    imported_only: bool,
    image: Arc<OriginalImage>,
    features: policy::Features,
    contracts: panic_contracts::Contracts,
}
impl LocalMacros {
    pub(super) fn new(
        unit: &Unit,
        features: policy::Features,
        limits: crate::expand::config::SourceContractLimits,
        imported_only: bool,
    ) -> Result<Self, Error> {
        let contracts = panic_contracts::Contracts::admitted(limits, &unit.source);
        contracts
            .charge(1, 4 * size_of::<Source>())
            .map_err(|message| Error::at(FileId(0), 0, message))?;
        Ok(Self {
            root: NodeKey {
                file: FileId(0),
                node: unit.configured.ast.root,
            },
            imported_only,
            image: Arc::clone(unit.source.original_image()),
            features,
            contracts,
        })
    }
    fn validate(&self, package: &Package) -> Result<(), &'static str> {
        let unit = package
            .units
            .first()
            .ok_or("local macro root disappeared")?;
        let owner = unit
            .owner_indices
            .get(self.root.node.get() as usize)
            .copied()
            .flatten();
        let owner_changed = match owner {
            Some(index) => unit
                .configured
                .owners
                .get(index.get() as usize - 1)
                .is_none_or(|owner| owner.node != self.root.node || !owner.enabled),
            None => !self.imported_only,
        };
        if self.root.file != FileId(0)
            || self.root.node != unit.configured.ast.root
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || unit.configured.active.get(self.root.node.get() as usize) != Some(&true)
            || owner_changed
        {
            return Err("local macro root incarnation or owner changed");
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(in crate::resolve) enum MacroContext<'a> {
    Provider(&'a SourceCore),
    Local(&'a LocalMacros),
}
impl<'a> MacroContext<'a> {
    pub(in crate::resolve) fn requested(self, feature: policy::Feature) -> bool {
        match self {
            Self::Provider(source) => source.features.has(feature),
            Self::Local(local) => local.features.has(feature),
        }
    }
    pub(in crate::resolve) const fn contracts(self) -> &'a panic_contracts::Contracts {
        match self {
            Self::Provider(source) => &source.macros,
            Self::Local(local) => &local.contracts,
        }
    }
    pub(in crate::resolve) fn require_header(
        self,
        entry: panic_contracts::Declaration,
    ) -> Result<(), &'static str> {
        let features = match self {
            Self::Provider(source) => &source.features,
            Self::Local(local) => {
                if entry.builtin.is_some() {
                    return Err("builtin macro binding requires an explicit source-core provider");
                }
                if entry.unsafe_lint {
                    return Err("local internal unsafe lint headers require their wider profile");
                }
                &local.features
            }
        };
        if entry.builtin.is_some() && !features.has(policy::Feature::RustcAttributes) {
            return Err("builtin macro binding requires rustc_attrs");
        }
        if entry.unsafe_lint && !features.has(policy::Feature::InternalUnsafeLint) {
            return Err("internal unsafe lint attribute requires allow_internal_unsafe");
        }
        if entry.has_allowance && !features.has(policy::Feature::InternalMacroAllowance) {
            return Err("internal macro allowance requires allow_internal_unstable");
        }
        Ok(())
    }
}
impl Source {
    pub(in crate::resolve) const fn owns_bindings(&self) -> bool {
        matches!(self, Self::Core(_) | Self::Client(_) | Self::Staged(_))
    }
}
impl Package {
    /// The authenticated cost owner grants no feature, macro or library permission.
    pub(in crate::resolve) fn source_contract_payer(
        &self,
    ) -> Result<Option<&panic_contracts::Contracts>, &'static str> {
        Ok(self.macro_context()?.map(MacroContext::contracts))
    }

    pub(in crate::resolve) fn macro_context(
        &self,
    ) -> Result<Option<MacroContext<'_>>, &'static str> {
        Ok(match self.source.as_deref() {
            Some(Source::Core(source)) => {
                if source.current != source.provider || source.current.name() != self.crate_name {
                    return Err("source macro provider identity changed");
                }
                Some(MacroContext::Provider(source))
            }
            Some(Source::LocalMacros(local)) => {
                local.validate(self)?;
                Some(MacroContext::Local(local))
            }
            // Source clients and staged consumers own a ledger only for the
            // imported allowance lists their dependencies carry.
            Some(Source::Client(client)) => match &client.allowances {
                Some(local) => {
                    local.validate(self)?;
                    Some(MacroContext::Local(local))
                }
                None => None,
            },
            Some(Source::Staged(staged)) => match &staged.allowances {
                Some(local) => {
                    local.validate(self)?;
                    Some(MacroContext::Local(local))
                }
                None => None,
            },
            None => None,
        })
    }
}
