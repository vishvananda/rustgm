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
    image: Arc<OriginalImage>,
    features: policy::Features,
    contracts: panic_contracts::Contracts,
}
impl LocalMacros {
    pub(super) fn new(unit: &Unit, features: policy::Features) -> Result<Self, Error> {
        let contracts = panic_contracts::Contracts::default();
        contracts
            .charge(1, 4 * size_of::<Source>())
            .map_err(|message| Error::at(FileId(0), 0, message))?;
        Ok(Self {
            root: NodeKey {
                file: FileId(0),
                node: unit.configured.ast.root,
            },
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
        if self.root.file != FileId(0)
            || self.root.node != unit.configured.ast.root
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || unit.configured.active.get(self.root.node.get() as usize) != Some(&true)
            || unit
                .owner_indices
                .get(self.root.node.get() as usize)
                .copied()
                .flatten()
                .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
                .is_none_or(|owner| owner.node != self.root.node || !owner.enabled)
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
        matches!(self, Self::Core(_) | Self::Client(_))
    }
}
impl Package {
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
            Some(Source::Client(_)) | None => None,
        })
    }
}
