//! Source weak statics hold an external symbol's address in actual Rust storage.
use super::{Context, GlobalId, GlobalStorage, Key, Layout, MemorySpan, Shape, Target, TypeId};
use crate::{
    abi::layout::Carrier,
    lowir::data::Item,
    resolve::model::DefId,
    sema::foreign::WeakStatic,
    types::{Kind, callable::Convention},
};

const POINTER: MemorySpan = MemorySpan { bytes: 8, align: 8 };

impl Context<'_> {
    pub(super) fn weak_static_data(
        &mut self,
        id: DefId,
        ty: TypeId,
        weak: WeakStatic,
    ) -> Result<GlobalId, &'static str> {
        let layout = self.layout(ty)?;
        if self.thread_local_static(id)
            || !layout.inhabited
            || layout.size != POINTER.bytes
            || layout.align != POINTER.align
        {
            return Err("extern_weak static requires ordinary pointer-sized storage");
        }
        self.data.charge(4)?;
        let name = self.data_name(Key::Definition(id))?;
        let target = self.weak_address_target(name, &layout, ty, weak)?;
        let cell = self.internal_name(format!("rg.weak.storage.{}", id.0));
        let global = self
            .output
            .internal_global(cell, POINTER, GlobalStorage::Readonly)?;
        self.output
            .initialize(global, &[Item::Address { target, addend: 0 }])?;
        self.data.globals.insert(Key::Definition(id), global);
        Ok(global)
    }

    fn weak_address_target(
        &mut self,
        name: String,
        layout: &Layout,
        ty: TypeId,
        weak: WeakStatic,
    ) -> Result<Target, &'static str> {
        match weak {
            WeakStatic::Data => {
                if !matches!(self.types.kind(ty), Some(Kind::Pointer { .. }))
                    || !matches!(layout.shape, Shape::Scalar)
                {
                    return Err("extern_weak data requires its checked thin raw pointer");
                }
                self.output
                    .declare_weak_global(name, POINTER, GlobalStorage::Readonly)
                    .map(Target::Global)
            }
            WeakStatic::Function(function) => {
                let niche = self
                    .layouts
                    .niche(layout)?
                    .ok_or("extern_weak callback requires its checked null niche")?;
                if niche.owner != ty
                    || niche.field != 0
                    || niche.offset != 0
                    || niche.ty != function
                    || !matches!(niche.carrier, Carrier::NonNullPointer { leaf } if leaf == function)
                    || niche.carrier.empty_bits(self.types)? != 0
                    || !matches!(
                        self.types.kind(function),
                        Some(Kind::Function {
                            convention: Convention::C,
                            ..
                        })
                    )
                {
                    return Err(
                        "extern_weak callback niche changed its checked C function pointer",
                    );
                }
                let signature = self.signature(function)?;
                self.output
                    .declare_weak_function(name, signature)
                    .map(Target::Function)
            }
        }
    }
}
