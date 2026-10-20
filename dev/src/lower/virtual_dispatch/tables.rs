//! Flat vtable data refers only to selected methods and proved destruction recipes.
use crate::{
    lower::{context::Context, virtual_dispatch::HEADER_BYTES},
    lowir::{
        data::{GlobalId, Item, Target},
        program::MemorySpan,
        vocabulary::Type,
    },
    sema::vtable_plans::TableId,
    types::TypeId,
};

impl Context<'_> {
    pub(in crate::lower) fn object_table(
        &mut self,
        concrete: TypeId,
        object: TypeId,
    ) -> Result<GlobalId, &'static str> {
        let id = self
            .objects
            .conversion(concrete, object)
            .ok_or("object conversion lacks a checked table plan")?;
        self.table(id)
    }
    fn table(&mut self, id: TableId) -> Result<GlobalId, &'static str> {
        if let Some(&global) = self.table_symbols.get(&id) {
            return Ok(global);
        }
        let table = self
            .objects
            .table(id)
            .ok_or("missing selected object table")?;
        let shape = self
            .objects
            .shape(table.shape)
            .ok_or("missing table object shape")?;
        let bytes = u64::try_from(table.methods.len().saturating_add(shape.upcasts.len()))
            .ok()
            .and_then(|n| n.checked_mul(8))
            .and_then(|n| n.checked_add(HEADER_BYTES))
            .ok_or("object table exceeds target address space")?;
        let global = self.output.global(
            format!("rg.vtable.{}", id.0),
            MemorySpan { bytes, align: 8 },
            true,
        )?;
        self.table_symbols.insert(id, global);
        self.table_queue.push(id);
        Ok(global)
    }
    pub(in crate::lower) fn initialize_table(&mut self, id: TableId) -> Result<(), &'static str> {
        let global = *self
            .table_symbols
            .get(&id)
            .ok_or("missing demanded vtable storage")?;
        let table = self
            .objects
            .table(id)
            .ok_or("missing selected object table")?;
        let concrete = table.concrete;
        let shape = self
            .objects
            .shape(table.shape)
            .ok_or("missing table object shape")?;
        let layout = self.layout(concrete)?;
        let drop = if let Some(proof) = self.destruction {
            proof.recipes().exposed(concrete)?
        } else {
            None
        };
        let mut items = vec![
            if let Some(drop) = drop {
                Item::Address {
                    target: Target::Function(self.demand_drop(drop)?),
                    addend: 0,
                }
            } else {
                Item::Zero(8)
            },
            Item::Bits {
                ty: Type::I64,
                bits: u128::from(layout.size),
            },
            Item::Bits {
                ty: Type::I64,
                bits: u128::from(layout.align),
            },
        ];
        for &method in &table.methods {
            items.push(if let Some(method) = method {
                Item::Address {
                    target: Target::Function(self.demand(method)?),
                    addend: 0,
                }
            } else {
                Item::Zero(8)
            });
        }
        for &target in &shape.upcasts {
            let target = self
                .objects
                .table_for_shape(concrete, target)
                .ok_or("upcast table dependency was not prepared")?;
            let global = self.table(target)?;
            items.push(Item::Address {
                target: Target::Global(global),
                addend: 0,
            });
        }
        self.output.initialize(global, &items)?;
        Ok(())
    }
}
