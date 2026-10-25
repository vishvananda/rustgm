//! Source intrinsic kernels are reached only from a checked direct constant call.
use super::{Declarations, Engine, Error, FileId, Kind, Report, const_eval};
use crate::{
    support::bindings::{LayoutIntrinsic, SourceIntrinsic},
    types::{Primitive, instance::Instance},
};
impl Engine {
    pub(super) fn source_layout(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
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
        if !matches!(facts.types.kind(selected.element), Some(Kind::Primitive(p)) if p != Primitive::Str)
        {
            return Err(error(
                "source layout kernel currently requires a closed primitive type",
            ));
        }
        facts
            .layouts
            .demand(&facts.types, &facts.aggregates, selected.element)
            .map_err(|e| error(e.message))?;
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
