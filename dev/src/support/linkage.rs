//! Exact symbol spellings are local arena identities, never serialized definition names.
use super::names::{NameId, Names};

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
}
impl SymbolNames {
    /// # Errors
    /// Rejects empty/NUL-containing names and exhausted arena identities.
    pub fn intern(&mut self, spelling: &str) -> Result<SymbolNameId, &'static str> {
        if spelling.is_empty() || spelling.contains('\0') {
            return Err("external symbol spelling must be nonempty and NUL-free");
        }
        self.names.intern_exact(spelling).map(SymbolNameId)
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
