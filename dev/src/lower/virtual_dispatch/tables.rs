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
            self.internal_name(format!("rg.vtable.{}", id.0)),
            MemorySpan { bytes, align: 8 },
            crate::lowir::data::GlobalStorage::Readonly,
        )?;
        self.table_symbols.insert(id, global);
        self.table_queue.push(id);
        Ok(global)
    }
    fn object_method_target(
        &mut self,
        method: crate::sema::vtable_plans::Method,
        instance: crate::types::instance::Instance,
    ) -> Result<crate::lowir::program::SymbolId, &'static str> {
        use crate::{sema::vtable_plans::Method, types::Kind};
        if let Method::Declared(definition) = method
            && self.constants.caller_tracking(definition)?
        {
            let Some(Kind::FunctionItem { item, .. }) = self.types.kind(instance.ty()) else {
                return Err("tracked virtual slot lost its selected function");
            };
            if !self
                .constants
                .caller_tracking(crate::resolve::model::DefId(item.0))?
            {
                return Err("tracked virtual slot requires a tracked implementation");
            }
            return self.demand(instance);
        }
        self.demand_reified(instance)
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
        for (index, &method) in table.methods.iter().enumerate() {
            items.push(if let Some(method) = method {
                Item::Address {
                    target: Target::Function(match method {
                        crate::sema::vtable_plans::Target::Function(instance) => {
                            let slot = shape
                                .slots
                                .get(index)
                                .ok_or("missing selected object slot")?;
                            self.object_method_target(slot.method, instance)?
                        }
                        crate::sema::vtable_plans::Target::Callable(callable) => {
                            let slot = shape.slots.get(index).ok_or("missing callable slot")?;
                            self.callable_target(callable, slot.method.require_callable()?)?
                        }
                        crate::sema::vtable_plans::Target::Captured(entry) => {
                            self.demand_captured(entry)?
                        }
                        crate::sema::vtable_plans::Target::Forwarded(operator) => {
                            self.object_operator(operator)?
                        }
                    }),
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
