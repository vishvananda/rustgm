//! Persistent object storage and relocations use typed identities and flat data.
use super::{
    program::{self, MemorySpan, Program, Range, SymbolId},
    vocabulary::Type,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlobalId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Global(GlobalId),
    Function(SymbolId),
}
#[derive(Clone, Copy, Debug)]
pub enum Item {
    Zero(u64),
    /// Exact target bits, including floating bits stored using integer types.
    Bits {
        ty: Type,
        bits: u128,
    },
    Address {
        target: Target,
        addend: i64,
    },
}
pub struct Global {
    pub name: String,
    pub span: MemorySpan,
    pub readonly: bool,
    pub items: Option<Range>,
}
impl Program {
    /// Reserve identity before traversing dependencies, including forward addresses.
    /// # Errors
    /// Rejects invalid storage spans or exhausted global identities.
    pub fn global(
        &mut self,
        name: String,
        span: MemorySpan,
        readonly: bool,
    ) -> Result<GlobalId, &'static str> {
        if span.bytes == 0
            || span.bytes > i64::MAX as u64
            || u64::from(span.align) > span.bytes
            || !span.align.is_power_of_two()
            || span.align > 1 << 29
        {
            return Err("invalid LowIR global storage span");
        }
        let id = GlobalId(program::index(self.globals.len())?);
        self.globals.push(Global {
            name,
            span,
            readonly,
            items: None,
        });
        Ok(id)
    }
    /// Install final object bytes and addresses once. Typed items follow the
    /// backend's natural alignment; explicit zero items supply raw padding.
    /// # Errors
    /// Rejects malformed data, invalid targets or an initializer exceeding its span.
    pub fn initialize(&mut self, id: GlobalId, items: &[Item]) -> Result<(), &'static str> {
        let global = self
            .globals
            .get(id.0 as usize)
            .ok_or("invalid LowIR global identity")?;
        if global.items.is_some() {
            return Err("LowIR global initialized twice");
        }
        if items.is_empty() {
            return Err("LowIR global initializer is empty");
        }
        let mut offset = 0_u64;
        for &item in items {
            let (bytes, align) = match item {
                Item::Zero(bytes) if bytes != 0 => (bytes, 1),
                Item::Zero(_) => return Err("zero-length LowIR data item"),
                Item::Bits { ty, bits } => {
                    if !ty.is_integer()
                        || (ty == Type::I1 && bits > 1)
                        || (ty.bits() < 128 && bits >> ty.bits() != 0)
                    {
                        return Err("invalid LowIR integer data bits");
                    }
                    let bytes = u64::from(ty.bits().div_ceil(8));
                    (bytes, bytes)
                }
                Item::Address { target, .. } => {
                    self.data_target(target)?;
                    (8, 8)
                }
            };
            if align > u64::from(global.span.align) {
                return Err("LowIR data item exceeds global alignment");
            }
            offset = offset
                .checked_add(align - 1)
                .map(|n| n & !(align - 1))
                .and_then(|n| n.checked_add(bytes))
                .filter(|n| *n <= global.span.bytes)
                .ok_or("LowIR global initializer exceeds storage span")?;
        }
        let start = program::index(self.data.len())?;
        let end = program::index(
            self.data
                .len()
                .checked_add(items.len())
                .ok_or("too much LowIR global data")?,
        )?;
        self.data.extend_from_slice(items);
        self.globals[id.0 as usize].items = Some(Range { start, end });
        Ok(())
    }
    /// # Errors
    /// Rejects a missing global/function target.
    pub fn data_target(&self, target: Target) -> Result<&str, &'static str> {
        match target {
            Target::Global(id) => self.globals.get(id.0 as usize).map(|g| g.name.as_str()),
            Target::Function(id) => self.symbols.get(id.0 as usize).map(|s| s.name.as_str()),
        }
        .ok_or("invalid LowIR data address target")
    }
}
