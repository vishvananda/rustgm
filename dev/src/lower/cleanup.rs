//! Demand shared glue and emit selected cleanup with bounded iterative work.
mod emit;
use super::{address::Address, context::Context};
use crate::{
    borrow::destruction::{Condition, Plan, glue},
    lowir::{
        builder::Builder,
        program::{Function, Operand, SlotId, SymbolId},
        vocabulary::Type,
    },
    types::TypeId,
};

#[derive(Default)]
pub(super) struct State {
    flags: Vec<SlotId>,
    work: Vec<emit::Work>,
    steps: usize,
}
impl State {
    pub fn allocate(&mut self, builder: &mut Builder, plan: &Plan) -> Result<(), &'static str> {
        for _ in &plan.flags {
            self.flags.push(builder.slot(Type::U8)?);
        }
        Ok(())
    }
    pub fn initialize(&mut self, builder: &mut Builder) -> Result<(), &'static str> {
        let end = u32::try_from(self.flags.len()).map_err(|_| "too many cleanup flags")?;
        self.write(builder, glue::Range { start: 0, end }, false)
    }
    pub fn write(
        &mut self,
        builder: &mut Builder,
        range: glue::Range,
        initialized: bool,
    ) -> Result<(), &'static str> {
        self.charge((range.end.saturating_sub(range.start)) as usize)?;
        let slots = self
            .flags
            .get(range.start as usize..range.end as usize)
            .ok_or("invalid cleanup flag interval")?;
        if slots.is_empty() {
            return Ok(());
        }
        let value = builder.integer(Type::U8, u128::from(initialized))?;
        for &slot in slots {
            builder.store(value, Operand::Slot(slot))?;
        }
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 32_000_000)
            .ok_or("cleanup emission work budget exceeded")?;
        Ok(())
    }
}
impl Context<'_> {
    pub fn demand_drop(&mut self, id: glue::Id) -> Result<SymbolId, &'static str> {
        if let Some(Some(symbol)) = self.drop_symbols.get(id.0 as usize) {
            return Ok(*symbol);
        }
        self.destruction
            .and_then(|proof| proof.recipes().get(id))
            .ok_or("missing proved destructor recipe")?;
        let signature = self.output.signature(&[Type::Ptr], Type::Void, false)?;
        let symbol = self.output.declare(
            self.internal_name(format!("rg.drop.{}", id.0)),
            signature,
            false,
        )?;
        *self
            .drop_symbols
            .get_mut(id.0 as usize)
            .ok_or("invalid destructor demand index")? = Some(symbol);
        self.drop_queue.push(id);
        Ok(symbol)
    }
}
pub(super) fn cleanup(
    context: &mut Context<'_>,
    builder: &mut Builder,
    state: &mut State,
    plan: &Plan,
    node: crate::borrow::destruction::NodeId,
    ty: TypeId,
    base: Address,
) -> Result<(), &'static str> {
    let mut emitter = emit::Emitter {
        context,
        builder,
        state,
        plan: Some(plan),
    };
    emitter.selected(node, ty, base)
}
pub(super) fn glue(context: &mut Context<'_>, id: glue::Id) -> Result<Function, &'static str> {
    let mut builder = Builder::default();
    let entry = builder.block()?;
    builder.enter(entry)?;
    let mut state = State::default();
    emit::Emitter {
        context,
        builder: &mut builder,
        state: &mut state,
        plan: None,
    }
    .recipe(id)?;
    builder.terminate(crate::lowir::program::Terminator::Return(None))?;
    builder.finish()
}
