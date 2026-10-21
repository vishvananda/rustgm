//! One symbol selector separates canonical Rust identity from explicit foreign spelling.
use crate::{
    resolve::model::DefId,
    support::linkage::{Naming, SymbolNames},
    types::instance::Instance,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Identity {
    Callable(Instance),
    Static(DefId),
}

/// Rust naming is supplied lazily by the compilation's linkage context.
/// Exact spellings never enter that callback or its crate-name encoding.
///
/// # Errors
/// Rejects missing exact-name pools, invalid local identities, spellings outside
/// the pinned `LowIR` symbol vocabulary, and Rust-name provider failures.
pub fn select(
    names: Option<&SymbolNames>,
    naming: Naming,
    identity: Identity,
    rust_name: impl FnOnce(Identity) -> Result<String, &'static str>,
) -> Result<String, &'static str> {
    let Naming::Exact(id) = naming else {
        return rust_name(identity);
    };
    let spelling = names
        .and_then(|names| names.get(id))
        .ok_or("missing exact symbol spelling in its owning pool")?;
    if !spelling
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$'))
    {
        return Err("exact symbol spelling is outside the pinned LowIR vocabulary");
    }
    Ok(spelling.to_owned())
}

impl super::context::Context<'_> {
    pub(super) fn internal_name(&self, legacy: String) -> String {
        if let Some(prefix) = self.crates.and_then(crate::sema::crates::Linkage::prefix) {
            format!("{prefix}internal${legacy}")
        } else {
            legacy
        }
    }
    pub(super) fn rust_name(
        &self,
        identity: Identity,
        specialization: Option<usize>,
    ) -> Result<String, &'static str> {
        let definition = match identity {
            Identity::Callable(instance) => {
                let Some(crate::types::Kind::FunctionItem { item, .. }) =
                    self.types.kind(instance.ty())
                else {
                    return Err("invalid callable Rust symbol identity");
                };
                DefId(item.0)
            }
            Identity::Static(definition) => definition,
        };
        if let Some(base) = self
            .crates
            .and_then(|crates| crates.rust_symbol(definition))
        {
            return Ok(
                specialization.map_or_else(|| base.to_owned(), |index| format!("{base}.i{index}"))
            );
        }
        if self
            .foreign
            .and_then(|registry| registry.function(definition))
            .is_some_and(|function| function.linkage == crate::support::linkage::Linkage::Import)
            || self
                .foreign
                .is_some_and(|registry| registry.imports_global(definition))
        {
            return Err("Rust import is missing external symbol identity");
        }
        let legacy = specialization.map_or_else(
            || format!("rg.{}", definition.0),
            |index| format!("rg.{}.i{index}", definition.0),
        );
        Ok(self.internal_name(legacy))
    }
}
