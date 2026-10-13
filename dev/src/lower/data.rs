//! Demand persistent objects by semantic identity, before traversing their data.
mod initializer;
mod zero;
use super::context::Context;
use crate::{
    abi::layout::{Layout, Shape},
    body::ir::ValueId,
    lowir::{
        builder::Builder,
        data::{GlobalId, Target},
        program::{MemorySpan, Operand},
        vocabulary::Type,
    },
    resolve::model::DefId,
    sema::{
        model::InitializerKind,
        values::{Constant, LocationId, Origin, View},
    },
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Definition(DefId),
    Location(LocationId),
    Temporary(DefId, ValueId),
    Bytes(crate::sema::text::Id),
}
#[derive(Clone, Copy)]
pub(super) struct Pointer {
    pub global: GlobalId,
    pub offset: u64,
}
#[derive(Clone, Copy)]
pub(super) struct Pending {
    global: GlobalId,
    value: Constant,
    ty: TypeId,
}
struct Fixup {
    global: GlobalId,
    offset: u64,
    target: Target,
    addend: i64,
}
#[derive(Default)]
pub(super) struct Storage {
    globals: HashMap<Key, GlobalId>,
    locations: HashMap<LocationId, Pointer>,
    static_indices: Option<HashMap<DefId, usize>>,
    pub pending: Vec<Pending>,
    fixups: Vec<Fixup>,
    zeroes: HashMap<(TypeId, Constant), bool>,
    steps: usize,
}
impl Storage {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 1_000_000)
            .ok_or("persistent data work budget exceeded")?;
        Ok(())
    }
}
impl Context<'_> {
    pub fn static_data(&mut self, id: DefId) -> Result<GlobalId, &'static str> {
        if let Some(&global) = self.data.globals.get(&Key::Definition(id)) {
            return Ok(global);
        }
        let indices = self.data.static_indices.get_or_insert_with(|| {
            self.initializers
                .iter()
                .enumerate()
                .filter_map(|(index, i)| {
                    matches!(i.kind, InitializerKind::Static { .. })
                        .then_some((i.definition, index))
                })
                .collect()
        });
        let initializer =
            self.initializers[*indices.get(&id).ok_or("missing static initializer")?];
        let value = self
            .constants
            .definition(id)
            .ok_or("static initializer was not evaluated")?;
        self.constant_data(
            Key::Definition(id),
            value,
            initializer.ty,
            matches!(initializer.kind, InitializerKind::Static { mutable: false }),
        )
    }
    pub fn constant_data(
        &mut self,
        key: Key,
        value: Constant,
        ty: TypeId,
        readonly: bool,
    ) -> Result<GlobalId, &'static str> {
        if let Some(&global) = self.data.globals.get(&key) {
            return Ok(global);
        }
        let layout = self.layout(ty)?;
        self.data.charge(1)?;
        let global = self.output.global(
            format!("rg.data.{}", self.output.globals.len()),
            MemorySpan {
                bytes: layout.size.max(u64::from(layout.align)),
                align: layout.align,
            },
            readonly,
        )?;
        self.data.globals.insert(key, global);
        self.data.pending.push(Pending { global, value, ty });
        Ok(global)
    }
    fn constant_pointer(&mut self, mut id: LocationId) -> Result<Pointer, &'static str> {
        let mut path = Vec::new();
        let mut pointer = loop {
            self.data.charge(1)?;
            if let Some(&pointer) = self.data.locations.get(&id) {
                break pointer;
            }
            let location = self.constants.location(id)?;
            match location.origin {
                Origin::Projection { base, index } => {
                    let parent = self.constants.location(base)?;
                    let layout = self.layout(parent.ty)?;
                    let (offset, ty) = self.constant_field(&layout, parent.value, index)?;
                    if ty != location.ty {
                        return Err("constant projection differs from its canonical field type");
                    }
                    path.push((id, offset));
                    id = base;
                }
                Origin::Static(definition) => {
                    let pointer = Pointer {
                        global: self.static_data(definition)?,
                        offset: 0,
                    };
                    self.data.locations.insert(id, pointer);
                    break pointer;
                }
                Origin::Allocation => {
                    let value = location.value.ok_or("constant allocation lacks a value")?;
                    let global = if self.types.is_sized(location.ty) == Some(false) {
                        let (View::Text(text) | View::Bytes(text)) = self.constants.view(value)?
                        else {
                            return Err("unsized constant allocation lacks byte storage");
                        };
                        self.bytes_data(text)?
                    } else {
                        self.constant_data(Key::Location(id), value, location.ty, true)?
                    };
                    let pointer = Pointer { global, offset: 0 };
                    self.data.locations.insert(id, pointer);
                    break pointer;
                }
            }
        };
        while let Some((id, offset)) = path.pop() {
            pointer.offset = pointer
                .offset
                .checked_add(offset)
                .filter(|n| i64::try_from(*n).is_ok())
                .ok_or("constant subobject offset overflow")?;
            self.data.locations.insert(id, pointer);
        }
        Ok(pointer)
    }
    fn constant_variant(&self, value: Constant) -> Result<usize, &'static str> {
        let View::Product { variant, .. } = self.constants.view(value)? else {
            return Err("aggregate storage requires a product constant");
        };
        variant.map_or(Ok(0), |id| {
            self.members
                .variants
                .get(id.0 as usize)
                .map(|v| v.ordinal as usize)
                .ok_or("missing constant variant")
        })
    }
    fn constant_field(
        &self,
        layout: &Layout,
        value: Option<Constant>,
        index: u64,
    ) -> Result<(u64, TypeId), &'static str> {
        match layout.shape {
            Shape::Array {
                stride,
                length,
                element,
            } if index < length => Ok((
                stride
                    .checked_mul(index)
                    .ok_or("constant element offset overflow")?,
                element,
            )),
            Shape::Aggregate { .. } => {
                let ordinal = if self.layouts.variants(layout).len() == 1 {
                    0
                } else {
                    self.constant_variant(value.ok_or("constant variant lacks a selected value")?)?
                };
                let variant = self
                    .layouts
                    .variants(layout)
                    .get(ordinal)
                    .ok_or("missing constant variant layout")?;
                let index = usize::try_from(index).map_err(|_| "constant field index overflow")?;
                let offset = *self
                    .layouts
                    .offsets(variant)
                    .get(index)
                    .ok_or("missing constant field offset")?;
                let ty = *self
                    .types
                    .list(variant.fields)
                    .and_then(|f| f.get(index))
                    .ok_or("missing constant field type")?;
                Ok((offset, ty))
            }
            _ => Err("constant field exceeds its canonical storage"),
        }
    }
    pub fn initialize_addresses(&self, builder: &mut Builder) -> Result<(), &'static str> {
        // The backend has naturally aligned data relocations only. Packed address
        // fields are zero-filled and fixed up once, before any Rust entry code.
        for fixup in &self.data.fixups {
            let target = match fixup.target {
                Target::Global(id) => Operand::Global(id),
                Target::Function(id) => Operand::Symbol(id),
            };
            let mut value = builder.address(target)?;
            if fixup.addend != 0 {
                let bits = u64::from_ne_bytes(fixup.addend.to_ne_bytes());
                let delta = builder.integer(Type::I64, u128::from(bits))?;
                value = builder.index(Type::U8, value, delta)?;
            }
            let mut destination = builder.address(Operand::Global(fixup.global))?;
            if fixup.offset != 0 {
                let offset = builder.integer(Type::I64, u128::from(fixup.offset))?;
                destination = builder.index(Type::U8, destination, offset)?;
            }
            builder.store(value, destination.operand)?;
        }
        Ok(())
    }
}
