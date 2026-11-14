//! Source intrinsic kernels are reached only from a checked direct constant call.
mod normalized_fields;
use super::{Declarations, Engine, Error, FileId, Report, const_eval};
use crate::{
    support::bindings::{LayoutIntrinsic, SourceIntrinsic},
    types::instance::Instance,
};

/// One source provider owns this budget across requests, waits and failures.
/// Whole-crate canonical observations include unrelated inference and selection.
#[derive(Default)]
pub(super) struct Work {
    steps: usize,
}
impl Work {
    fn charge(
        &mut self,
        facts: &mut Declarations,
        steps: usize,
        message: &'static str,
    ) -> Result<(), String> {
        self.steps = self.steps.saturating_add(steps);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
        if self.steps > crate::support::crates::MAX_COMPONENTS {
            return Err(message.into());
        }
        Ok(())
    }
    fn remaining(&self) -> usize {
        crate::support::crates::MAX_COMPONENTS.saturating_sub(self.steps)
    }
}
impl Engine {
    fn source_wrapping_layout(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
        let Some(element) = state
            .map(|state| {
                state.source_wrapping_offset_element(report, facts, &self.catalog, instance)
            })
            .transpose()
            .map_err(|error| Error::at(FileId(0), 0, error))?
            .flatten()
        else {
            return Ok(false);
        };
        self.storage_ready(facts, element)
    }
    fn source_field_layout(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
        if let Some(call) = state
            .map(|state| {
                state.source_field_offset_call(
                    facts,
                    &self.catalog,
                    self.const_calls.as_deref(),
                    instance,
                )
            })
            .transpose()
            .map_err(|error| Error::at(FileId(0), 0, error))?
            .flatten()
        {
            facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| Error::at(FileId(0), 0, "source field_offset catalog disappeared"))?
                .require_field_offset_report(report, facts, &call)
                .map_err(|error| Error::at(FileId(0), 0, error))?;
            return self.storage_ready(facts, call.coordinates.base);
        }
        Ok(false)
    }
    /// A source nominal still awaiting its declared fields is surfaced to the
    /// declaration pipeline; the session resumes with this same requirement.
    fn storage_ready(
        &mut self,
        facts: &mut Declarations,
        element: crate::types::TypeId,
    ) -> Result<bool, Error> {
        let pending = prepare_layout(facts, element, &mut self.layout_work)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        self.pending_fields = pending;
        Ok(pending.is_none())
    }
    pub(super) fn source_layout(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
        if self.source_wrapping_layout(report, facts, instance, state)?
            || self.source_field_layout(report, facts, instance, state)?
        {
            return Ok(true);
        }
        if let Some(call) = state
            .map(|state| state.source_offset_call(facts, &self.catalog, instance))
            .transpose()
            .map_err(|error| Error::at(FileId(0), 0, error))?
            .flatten()
        {
            facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| Error::at(FileId(0), 0, "source offset_of catalog disappeared"))?
                .require_offset_report(report, facts, call.definition())
                .map_err(|error| Error::at(FileId(0), 0, error))?;
            return self.storage_ready(facts, call.element());
        }
        if let Some(call) = state
            .map(|state| state.source_transmute_call(report, facts, &self.catalog, instance))
            .transpose()
            .map_err(|error| Error::at(FileId(0), 0, error))?
            .flatten()
        {
            let selected = call.selection();
            return Ok(self.storage_ready(facts, selected.source)?
                && self.storage_ready(facts, selected.destination)?);
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
        if !self.storage_ready(facts, selected.element)? {
            return Ok(false);
        }
        let layout = facts
            .layouts
            .get(selected.element)
            .ok_or_else(|| error("source layout result is not prepared"))?;
        let bits = match selected.kind {
            LayoutIntrinsic::SizeOf => layout.size,
            LayoutIntrinsic::AlignOf => u64::from(layout.align),
            LayoutIntrinsic::AssertInhabited => u64::from(layout.inhabited),
        };
        self.catalog.cache_layout(&call, bits).map_err(error)?;
        Ok(true)
    }
}

/// The reached call supplies permission; this only prepares its closed storage.
fn prepare_layout(
    facts: &mut Declarations,
    element: crate::types::TypeId,
    budget: &mut Work,
) -> Result<Option<crate::types::ItemId>, String> {
    budget.charge(
        facts,
        1,
        "source layout field preparation work budget exceeded",
    )?;
    let before = facts.aggregates.instantiation_steps();
    let fields = if facts.aggregates.has_projected_templates() {
        normalized_fields::prepare(facts, element, budget)
    } else {
        facts
            .aggregates
            .prepare_storage_fields_pending(&mut facts.types, element)
            .map_err(String::from)
    };
    let instantiated = facts
        .aggregates
        .instantiation_steps()
        .saturating_sub(before);
    let charged = budget.charge(
        facts,
        instantiated,
        "source layout field preparation work budget exceeded",
    );
    let pending = fields?;
    charged?;
    if let Some(item) = pending {
        return Ok(Some(item));
    }
    let pending = facts.layouts.pending_state_work(&facts.types, element);
    budget.charge(
        facts,
        pending,
        "source layout state preparation work budget exceeded",
    )?;
    let mut work = 0;
    let readiness = facts.layouts.storage_readiness(
        &facts.types,
        &facts.aggregates,
        element,
        &mut work,
        budget.remaining(),
    );
    let charged = budget.charge(facts, work, "constant storage layout work budget exceeded");
    readiness?;
    charged?;
    if facts.layouts.get(element).is_some() {
        return Ok(None);
    }
    let before = facts.layouts.steps;
    let result =
        facts
            .layouts
            .demand_bounded(&facts.types, &facts.aggregates, element, budget.remaining());
    let steps = facts.layouts.steps.saturating_sub(before);
    let charged = budget.charge(
        facts,
        steps,
        "source layout state preparation work budget exceeded",
    );
    result.map_err(|error| String::from(error.message))?;
    charged?;
    Ok(None)
}

#[cfg(test)]
#[path = "../../../tests/invariants/queried_storage.rs"]
mod tests;
