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
