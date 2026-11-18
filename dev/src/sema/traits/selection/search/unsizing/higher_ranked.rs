//! Quantified object requirements share one binder and keep its evidence local.
use super::super::{Declarations, Frame, Search, inference, mechanical};
use crate::types::{lifetime_proof::ScopeId, substitute::Bound};

pub(super) struct Inputs {
    scope: ScopeId,
    count: u32,
    closing: Bound,
    bridges: usize,
}

impl Search<'_> {
    pub(super) fn open_unsize_child(
        &mut self,
        facts: &mut Declarations,
        child: &mut Frame,
        binder: &mut Option<Box<Inputs>>,
        count: u32,
    ) -> Result<(), String> {
        let first = self.query.allocations.reserve_scoped().max(child.first);
        if self.query.scopes.is_none() {
            facts.charge_caller_inputs(1, size_of::<super::super::scopes::Scopes>())?;
        }
        let scopes = self.query.scopes.get_or_insert_with(Default::default);
        let inputs = if let Some(inputs) = binder {
            if inputs.count != count {
                return Err("object requirements changed their shared binder".into());
            }
            inputs
        } else {
            facts.charge_caller_inputs(1, size_of::<Inputs>() + Bound::region_mapping_bytes())?;
            let scope = scopes.introduce_counted(child.scope, count, first, &|work, bytes| {
                facts.charge_caller_inputs(work, bytes)
            })?;
            self.query.allocations.record(first + count);
            binder.insert(Box::new(Inputs {
                scope,
                count,
                closing: Bound::scoped_export_regions(),
                bridges: 0,
            }))
        };
        child.first = first.max(self.query.allocations.end());
        scopes.open_inferred(facts, child, inputs.scope)?;
        Ok(())
    }

    pub(super) fn match_unsize_child(
        &mut self,
        facts: &mut Declarations,
        parent: &mut Frame,
        child: &mut Frame,
        selected: usize,
    ) -> Result<(), String> {
        child.match_scoped_goal(facts, self.query.nodes[selected].goal)?;
        self.publish_unsize_child(facts, parent, child)?;
        let equalities = std::mem::take(&mut child.inputs.context.region_equalities);
        parent
            .inputs
            .context
            .account_work(equalities.len())
            .map_err(inference)?;
        self.query
            .scopes
            .as_mut()
            .ok_or("missing Unsize proof scopes")?
            .equalities(child.scope, equalities)?;
        Ok(())
    }

    pub(super) fn publish_unsize_child(
        &mut self,
        facts: &mut Declarations,
        parent: &mut Frame,
        child: &mut Frame,
    ) -> Result<(), String> {
        let parent_scope = parent.scope;
        let Some(mechanical::Pending::Unsize(pending)) = parent.language.as_deref_mut() else {
            return Err("missing quantified Unsize parent".into());
        };
        let inputs = pending
            .scoped
            .as_mut()
            .filter(|inputs| inputs.scope == child.scope)
            .ok_or("quantified Unsize child changed its proof scope")?;
        let scopes = self
            .query
            .scopes
            .as_mut()
            .ok_or("missing Unsize proof scopes")?;
        let first = self.query.allocations.end();
        let regions = child
            .inputs
            .publication_regions(&mut facts.types)
            .map_err(inference)?;
        let growth = crate::support::retained::Growth::vector::<(crate::types::RegionId, bool)>(
            0,
            0,
            regions.len(),
        );
        facts.charge_caller_inputs(growth.work.saturating_add(regions.len()), growth.bytes)?;
        let mut work = 0;
        let sources = regions
            .into_iter()
            .map(|region| {
                scopes
                    .export_source(parent_scope, child.scope, region, first, &mut work)
                    .map(|private| (region, private))
            })
            .collect::<Result<Vec<_>, _>>();
        parent
            .inputs
            .context
            .account_work(work)
            .map_err(inference)?;
        let before = inputs.closing.steps;
        inputs
            .closing
            .prepare_scoped_export(&sources?, first, &|work, bytes| {
                facts.charge_caller_inputs(work, bytes)
            })?;
        child
            .inputs
            .publish(
                &mut facts.types,
                &mut parent.inputs.context,
                &mut inputs.closing,
            )
            .map_err(inference)?;
        parent
            .inputs
            .context
            .account_work(inputs.closing.steps.saturating_sub(before))
            .map_err(inference)?;
        let end = inputs
            .closing
            .next_region()
            .ok_or("missing scoped Unsize witness frontier")?;
        for (private, exported) in inputs.closing.exported_regions().skip(inputs.bridges) {
            parent
                .inputs
                .context
                .account_work(scopes.bridge(private, exported)?)
                .map_err(inference)?;
        }
        inputs.bridges = inputs.closing.exported_regions().len();
        self.query.allocations.record(end);
        self.query.retain_scoped_exports(parent_scope, first..end)?;
        Ok(())
    }
}
