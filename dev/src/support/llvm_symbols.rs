//! Reserved LLVM names retain typed operation identity instead of C linkage fallback.
use super::{
    hash::HashMap,
    linkage::SymbolNameId,
    retained::{self, Growth},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Population,
    ReverseBits,
    SwapBytes,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intrinsic {
    pub operation: Operation,
    pub bits: u16,
}
impl Intrinsic {
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        let name = name.strip_prefix("llvm.")?;
        let parsed = name.rsplit_once(".i").and_then(|(name, width)| {
            let bits = match width {
                "8" => 8,
                "16" => 16,
                "32" => 32,
                "64" => 64,
                "128" => 128,
                _ => return None,
            };
            let operation = match name {
                "ctpop" => Operation::Population,
                "bitreverse" => Operation::ReverseBits,
                "bswap" if bits != 8 => Operation::SwapBytes,
                _ => return None,
            };
            Some(Self { operation, bits })
        });
        Some(parsed.unwrap_or(Self {
            operation: Operation::Unsupported,
            bits: 0,
        }))
    }
}
#[derive(Default)]
pub(crate) struct Catalog {
    entries: HashMap<SymbolNameId, Intrinsic>,
}
impl Catalog {
    pub(crate) fn record(
        &mut self,
        id: SymbolNameId,
        intrinsic: Intrinsic,
        pay: &mut impl FnMut(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if self.entries.contains_key(&id) {
            return Ok(());
        }
        if self.entries.len() >= 100_000 {
            return Err("LLVM symbol catalog exceeds profile budget");
        }
        pay(retained::Growth::map::<SymbolNameId, Intrinsic>(
            self.entries.len(),
            self.entries.capacity(),
            1,
        ))?;
        self.entries
            .try_reserve(1)
            .map_err(|_| "LLVM symbol allocation failed")?;
        self.entries.insert(id, intrinsic);
        Ok(())
    }
    pub(crate) fn get(&self, id: SymbolNameId) -> Option<Intrinsic> {
        self.entries.get(&id).copied()
    }
}
