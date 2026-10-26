//! Source intrinsic kernels are reached only from a checked direct constant call.
use super::{Declarations, Engine, Error, FileId, Report, const_eval};
use crate::{
    support::bindings::{LayoutIntrinsic, SourceIntrinsic},
    types::instance::Instance,
};
impl Engine {
    pub(super) fn source_layout(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
        if let Some(call) = state
            .map(|state| state.source_transmute_call(report, facts, &self.catalog, instance))
            .transpose()
            .map_err(|error| Error::at(FileId(0), 0, error))?
            .flatten()
        {
            let selected = call.selection();
            prepare_layout(facts, selected.source)
                .map_err(|error| Error::at(FileId(0), 0, error))?;
            prepare_layout(facts, selected.destination)
                .map_err(|error| Error::at(FileId(0), 0, error))?;
            return Ok(true);
        }
        if let Some(selection) = facts
            .source_bindings
            .as_deref()
            .map(|c| c.runtime_check_selection(facts, instance))
            .transpose()
            .map_err(|e| Error::at(FileId(0), 0, e))?
            .flatten()
        {
            state
                .ok_or_else(|| {
                    Error::at(
                        FileId(0),
                        0,
                        "source ub_checks requires a reached checked caller",
                    )
                })?
                .source_runtime_check_call(facts, &self.catalog, instance)
                .map_err(|e| Error::at(FileId(0), 0, e))?
                .ok_or_else(|| {
                    Error::at(
                        FileId(0),
                        0,
                        "source ub_checks lost its actual checked target",
                    )
                })?;
            return if selection.fallback {
                self.function(report, facts, instance)
            } else {
                Ok(true)
            };
        }
        let selected = facts
            .source_bindings
            .as_deref()
            .map(|c| c.selected_intrinsic(facts, instance))
            .transpose()
            .map_err(|e| Error::at(FileId(0), 0, e))?
            .flatten();
        let Some(selected) = selected else {
            return self.function(report, facts, instance);
        };
        let error = |message| Error::at(FileId(0), 0, message);
        if selected.kind == SourceIntrinsic::NeedsDrop {
            return self.source_needs_drop(report, facts, instance, state);
        }
        let call = state
            .ok_or_else(|| error("source intrinsic requires a reached checked caller"))?
            .source_layout_call(facts, &self.catalog, instance)
            .map_err(error)?
            .ok_or_else(|| error("source intrinsic caller lost its validated target"))?;
        let selected = call.selection();
        // A source fallback remains an ordinary original and specialized body.
        // Its proof is mandatory even though layout supplies the executed result.
        if selected.fallback && !self.function(report, facts, instance)? {
            return Ok(false);
        }
        prepare_layout(facts, selected.element).map_err(error)?;
        let layout = facts
            .layouts
            .get(selected.element)
            .ok_or_else(|| error("source layout result is not prepared"))?;
        let bits = match selected.kind {
            LayoutIntrinsic::SizeOf => layout.size,
            LayoutIntrinsic::AlignOf => u64::from(layout.align),
        };
        self.catalog.cache_layout(&call, bits).map_err(error)?;
        Ok(true)
    }
}

/// The reached call supplies permission; this only prepares its closed storage.
fn prepare_layout(
    facts: &mut Declarations,
    element: crate::types::TypeId,
) -> Result<(), &'static str> {
    let limit = crate::support::crates::MAX_COMPONENTS;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    if facts.body_work.canonical_steps > limit {
        return Err("source layout field preparation work budget exceeded");
    }
    let before = facts.aggregates.instantiation_steps();
    let fields = facts
        .aggregates
        .prepare_storage_fields(&mut facts.types, element);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(
        facts
            .aggregates
            .instantiation_steps()
            .saturating_sub(before),
    );
    fields?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(facts.layouts.pending_state_work(&facts.types, element));
    if facts.body_work.canonical_steps > limit {
        return Err("source layout state preparation work budget exceeded");
    }
    let limit = limit.saturating_sub(facts.body_work.canonical_steps);
    let mut work = 0;
    let readiness =
        facts
            .layouts
            .storage_readiness(&facts.types, &facts.aggregates, element, &mut work, limit);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    readiness?;
    if facts.layouts.get(element).is_some() {
        return Ok(());
    }
    let before = facts.layouts.steps;
    let result = facts.layouts.demand_bounded(
        &facts.types,
        &facts.aggregates,
        element,
        limit.saturating_sub(work),
    );
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(facts.layouts.steps.saturating_sub(before));
    result.map(|_| ()).map_err(|error| error.message)
}

#[cfg(test)]
#[path = "../../../tests/invariants/queried_storage.rs"]
mod tests;
