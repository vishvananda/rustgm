//! Exact symbol spellings are local arena identities, never serialized definition names.
use super::names::{NameId, Names};

/// The reviewed CPPGM GNU initializer-section grammar, preserving spelling.
pub(crate) fn gnu_init_array_section(name: &str) -> bool {
    if name == ".init_array" {
        return true;
    }
    let Some(priority) = name.strip_prefix(".init_array.") else {
        return false;
    };
    !priority.is_empty()
        && priority.bytes().all(|byte| byte.is_ascii_digit())
        && priority.parse::<u16>().is_ok()
}

/// Scoped to one `SymbolNames` pool. Metadata transports the exact string bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolNameId(NameId);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Naming {
    #[default]
    Rust,
    Exact(SymbolNameId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Linkage {
    Import,
    Export,
    Internal,
}

#[derive(Default)]
pub struct SymbolNames {
    names: Names,
    llvm: Option<Box<super::llvm_symbols::Catalog>>,
}
impl SymbolNames {
    /// # Errors
    /// Rejects empty/NUL-containing names and exhausted arena identities.
    pub fn intern(&mut self, spelling: &str) -> Result<SymbolNameId, &'static str> {
        if spelling.is_empty() || spelling.contains('\0') {
            return Err("external symbol spelling must be nonempty and NUL-free");
        }
        let id = SymbolNameId(self.names.intern_exact(spelling)?);
        self.record_llvm(id, spelling, &mut |_| Ok(()))?;
        Ok(id)
    }
    pub(crate) fn find_exact_paid(
        &self,
        spelling: &str,
        pay: &mut impl FnMut(super::retained::Growth) -> Result<(), &'static str>,
    ) -> Result<Option<SymbolNameId>, &'static str> {
        self.names
            .find_exact_paid(spelling, pay)
            .map(|name| name.map(SymbolNameId))
    }
    /// Exact cold interning prepays work and growth without another arena.
    pub(crate) fn intern_paid(
        &mut self,
        spelling: &str,
        pay: &mut impl FnMut(super::retained::Growth) -> Result<(), &'static str>,
    ) -> Result<SymbolNameId, &'static str> {
        pay(super::retained::Growth {
            work: spelling.len().saturating_add(1),
            bytes: 0,
        })?;
        if spelling.is_empty() || spelling.contains('\0') {
            return Err("external symbol spelling must be nonempty and NUL-free");
        }
        let id = SymbolNameId(self.names.intern_exact_paid(spelling, pay)?);
        self.record_llvm(id, spelling, pay)?;
        Ok(id)
    }
    fn record_llvm(
        &mut self,
        id: SymbolNameId,
        spelling: &str,
        pay: &mut impl FnMut(super::retained::Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if self
            .llvm
            .as_deref()
            .and_then(|catalog| catalog.get(id))
            .is_some()
        {
            return Ok(());
        }
        let Some(intrinsic) = super::llvm_symbols::Intrinsic::named(spelling) else {
            return Ok(());
        };
        if self.llvm.is_none() {
            pay(super::retained::Growth {
                work: 1,
                bytes: size_of::<super::llvm_symbols::Catalog>(),
            })?;
            self.llvm = Some(Box::default());
        }
        self.llvm
            .as_mut()
            .ok_or("missing reserved LLVM symbol owner")?
            .record(id, intrinsic, pay)
    }
    #[must_use]
    pub fn llvm(&self, id: SymbolNameId) -> Option<super::llvm_symbols::Intrinsic> {
        self.llvm.as_deref()?.get(id)
    }
    #[must_use]
    pub fn get(&self, id: SymbolNameId) -> Option<&str> {
        (usize::try_from(id.0.0).ok()? < self.names.len()).then(|| self.names.get(id.0))
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.names.byte_len()
    }
}

/// The path root rustc 1.98.0 gives every `#[rustc_std_internal_symbol]`
/// item: v0's `__rustc::NAME` in a synthetic crate whose disambiguator hashes
/// the compiler version alone (`rustc_symbol_mangling::v0::mangle_internal_symbol`),
/// so all crates of a program agree on it. Observed with `nm` in rustc 1.98.0's
/// objects, for example `_RNvCs6rREvFdRhLb_7___rustc12___rust_alloc`.
const INTERNAL_SYMBOL_ROOT: &str = "_RNvCs6rREvFdRhLb_7___rustc";

/// The symbol of a `#[rustc_std_internal_symbol]` item whose unmangled name is
/// `name`: its own name or its `export_name`/`link_name`.
/// # Errors
/// Rejects an empty or non-ASCII name, which v0 would spell in Punycode.
pub fn internal_symbol(name: &str) -> Result<String, &'static str> {
    // LLVM hard-codes the personality routine's name, and C callers name
    // Apple's availability checks, so rustc keeps these names.
    if matches!(
        name,
        "rust_eh_personality" | "__isPlatformVersionAtLeast" | "__isOSVersionAtLeast"
    ) {
        return Ok(name.to_owned());
    }
    if name.is_empty() || !name.is_ascii() {
        return Err("internal symbol name must be nonempty ASCII");
    }
    // v0 separates a decimal length from an identifier that starts with `_`
    // or a digit by one `_`.
    let separator = if name.starts_with(|first: char| first == '_' || first.is_ascii_digit()) {
        "_"
    } else {
        ""
    };
    Ok(format!(
        "{INTERNAL_SYMBOL_ROOT}{}{separator}{name}",
        name.len()
    ))
}
