//! Shared concrete runtime properties are data; each reached operation proves its call.
use super::{Catalog, nounwind};
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report},
    sema::{model::Declarations, text},
    support::{bindings::LayoutIntrinsic, hash::HashMap},
    types::{Kind, TypeId, instance::Instance},
};
#[derive(Clone, Copy)]
pub(crate) struct Assertion {
    pub panic: nounwind::Entry,
    pub message: Option<text::Id>,
}
#[derive(Clone, Copy)]
pub(crate) struct Ready {
    pub signature: TypeId,
    pub result: TypeId,
    pub bits: u64,
    pub assertion: Option<Assertion>,
}
#[derive(Default)]
pub(super) struct Plans {
    entries: HashMap<Instance, Ready>,
}
impl Catalog {
    pub(crate) fn has_runtime_layouts(&self) -> bool {
        self.native_layouts.is_some()
    }
    pub(crate) fn runtime_layout(&self, instance: Instance) -> Option<Ready> {
        self.native_layouts
            .as_deref()?
            .entries
            .get(&instance)
            .copied()
    }
}
pub(crate) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    instance: Instance,
    origin: BodyOrigin,
) -> Result<Ready, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("runtime layout requires its source catalog")?;
    catalog.require_provider_report(report)?;
    if origin.source().is_none() {
        return Err("runtime layout requires its actual source body");
    }
    let existing = catalog.runtime_layout(instance);
    crate::sema::header_attributes::charge_source_contract_query(facts, 1)
        .map_err(|_| "runtime layout query work budget exceeded")?;
    if let Some(plan) = existing {
        return Ok(plan);
    }
    let selection = facts
        .source_bindings
        .as_deref()
        .ok_or("runtime layout catalog disappeared")?
        .layout_selection(facts, instance)?
        .ok_or("runtime layout lost its source intrinsic declaration")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(instance.ty()) else {
        return Err("runtime layout lost its complete function item");
    };
    let id = facts
        .layouts
        .demand(&facts.types, &facts.aggregates, selection.element)
        .map_err(|_| "runtime layout requires its completed physical type")?;
    let layout = facts
        .layouts
        .at(id)
        .ok_or("runtime layout result disappeared")?;
    let bits = match selection.kind {
        LayoutIntrinsic::SizeOf => layout.size,
        LayoutIntrinsic::AlignOf => u64::from(layout.align),
        LayoutIntrinsic::AssertInhabited => u64::from(layout.inhabited),
    };
    let assertion = if selection.kind == LayoutIntrinsic::AssertInhabited {
        // Pinned Rust collects the hook even when this concrete assertion is a NOP.
        let panic = nounwind::prepare(report, facts)?;
        let message = if bits == 0 {
            let name = super::inhabited_names::name(report, &facts.types, selection.element)?;
            let message = facts
                .texts
                .intern(format!("attempted to instantiate uninhabited type `{name}`").as_bytes())?;
            Some(message)
        } else {
            None
        };
        Some(Assertion { panic, message })
    } else {
        None
    };
    let plan = Ready {
        signature,
        result: selection.result_type(),
        bits,
        assertion,
    };
    publish(facts, instance, plan)?;
    Ok(plan)
}
fn publish(facts: &mut Declarations, instance: Instance, plan: Ready) -> Result<(), &'static str> {
    let prior = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.native_layouts.as_deref());
    let (len, capacity) = prior.map_or((0, 0), |plans| {
        (plans.entries.len(), plans.entries.capacity())
    });
    let growth = crate::support::retained::Growth::map::<Instance, Ready>(len, capacity, 1);
    if crate::support::retained::new_map::<Instance, Ready>(len.saturating_add(1))
        > 64 * 1024 * 1024
    {
        return Err("runtime layout property storage budget exceeded");
    }
    let fixed = usize::from(prior.is_none()) * size_of::<Plans>();
    crate::sema::header_attributes::reserve_source_contracts(
        facts,
        growth.work.saturating_add(1),
        growth.bytes.saturating_add(fixed),
    )
    .map_err(|_| "runtime layout property producer budget exceeded")?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or("runtime layout catalog disappeared")?;
    let plans = catalog.native_layouts.get_or_insert_with(Default::default);
    plans
        .entries
        .try_reserve(1)
        .map_err(|_| "runtime layout property allocation failed")?;
    plans.entries.insert(instance, plan);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/inhabited_plans.rs"]
mod tests;
