//! Caller input witnesses cross a universal boundary only with retained scoped equalities.
use super::{Context, Declarations, Frame, Query, Search, inference};
use crate::types::{
    lifetime_proof::ScopeId,
    substitute::{Bound, ExportSource},
};

pub(super) struct InputUniverse {
    scope: ScopeId,
    closing: Option<Bound>,
    pub sources: Vec<ExportSource>,
    bridges: usize,
}

impl Query {
    pub(super) fn input_scope(&self) -> ScopeId {
        self.universal
            .as_ref()
            .map_or(ScopeId::ROOT, |universal| universal.scope)
    }
}

impl Search<'_> {
    pub(super) fn open_universal(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        count: u32,
    ) -> Result<(), String> {
        if self.query.universal.is_some() || self.caller.retain_sized {
            return Err("invalid universal input query mode".into());
        }
        let first = self.query.allocations.reserve_scoped();
        let scopes = self.query.scopes.get_or_insert_with(Default::default);
        let scope = scopes.introduce(ScopeId::ROOT, count, first)?;
        let end = scopes.open_inferred(facts, frame, scope)?;
        self.query.allocations.record(end);
        self.query.universal = Some(Box::new(InputUniverse {
            scope,
            closing: None,
            sources: if first > 1 {
                vec![ExportSource::Caller(1..first)]
            } else {
                Vec::new()
            },
            bridges: 0,
        }));
        Ok(())
    }

    pub(super) fn complete_universal(
        &mut self,
        facts: &mut Declarations,
        caller: &mut Context,
        selected: usize,
    ) -> Result<(), String> {
        // Match inside the opened scope. Equating the original bound slots with
        // this node directly in the caller would turn local universals into inputs.
        let frame = self
            .query
            .stack
            .first_mut()
            .ok_or("missing universal root frame")?;
        frame
            .inputs
            .context
            .equate_proved_predicate(
                &mut facts.types,
                &frame.goal,
                self.query.nodes[selected].goal,
            )
            .map_err(inference)?;
        self.publish_inputs(facts, caller)?;
        let frame = self
            .query
            .stack
            .first_mut()
            .ok_or("missing completed universal frame")?;
        let equalities = std::mem::take(&mut frame.inputs.context.region_equalities);
        caller.account_work(equalities.len()).map_err(inference)?;
        self.query
            .scopes
            .as_mut()
            .ok_or("missing universal proof scopes")?
            .equalities(frame.scope, equalities)?;
        Ok(())
    }
}

impl InputUniverse {
    pub fn publish(
        &mut self,
        facts: &mut Declarations,
        caller: &mut Context,
        frame: &mut Frame,
        allocations: &mut super::allocations::Allocations,
        scopes: &mut super::scopes::Scopes,
    ) -> Result<bool, String> {
        let first = allocations.end();
        if let Some(closing) = &mut self.closing {
            let before = closing.steps;
            closing.resume_export(&self.sources, first)?;
            caller
                .account_work(closing.steps.saturating_sub(before))
                .map_err(inference)?;
        } else {
            let closing = Bound::export_regions(&self.sources, first)?;
            caller.account_work(closing.steps).map_err(inference)?;
            self.closing = Some(closing);
        }
        self.sources.clear();
        let closing = self
            .closing
            .as_mut()
            .ok_or("missing universal input export")?;
        let changed = frame
            .inputs
            .publish(&mut facts.types, caller, closing)
            .map_err(inference)?;
        allocations.record(
            closing
                .next_region()
                .ok_or("missing exported lifetime frontier")?,
        );
        for (private, exported) in closing.exported_regions().skip(self.bridges) {
            caller
                .account_work(scopes.bridge(private, exported)?)
                .map_err(inference)?;
        }
        self.bridges = closing.exported_regions().len();
        Ok(changed)
    }
}
