//! One exact sparse query shares the completed immutable destructor registry.
use super::{Catalog, HashMap, Instance, LIMIT, growth, retained};
use crate::{sema::model::Declarations, types::destruction::NeedsDrop};
pub(super) struct DropResults {
    answers: HashMap<Instance, bool>,
    query: NeedsDrop,
}
impl Catalog {
    #[cfg(test)]
    pub(crate) fn needs_drop_statistics(&self) -> Option<(usize, usize, usize)> {
        let drops = self.source_results.as_deref()?.drops.as_deref()?;
        Some((
            drops.query.steps,
            drops.query.retained_bytes(),
            drops.answers.len(),
        ))
    }
    pub(crate) fn needs_drop_result(&self, instance: Instance) -> Option<bool> {
        self.source_results
            .as_deref()?
            .drops
            .as_deref()?
            .answers
            .get(&instance)
            .copied()
    }
    /// Fresh checked call and actual role gates precede every cache access.
    pub(crate) fn evaluate_needs_drop(
        &mut self,
        facts: &Declarations,
        call: &super::super::SourceNeedsDropCall,
    ) -> Result<bool, &'static str> {
        facts
            .source_bindings
            .as_deref()
            .ok_or("source intrinsic catalog disappeared")?
            .require_needs_drop_roles()?;
        if let Some(value) = self.needs_drop_result(call.instance()) {
            return Ok(value);
        }
        self.prepare_drop_query(facts)?;
        let drops = self
            .source_results
            .as_deref_mut()
            .and_then(|r| r.drops.as_deref_mut())
            .ok_or("source needs_drop query disappeared")?;
        let before = drops.query.retained_bytes();
        let limit = LIMIT
            .saturating_sub(self.bytes)
            .saturating_add(before)
            .checked_sub(growth(&drops.answers))
            .ok_or("source needs_drop cache storage budget exceeded")?;
        let value = drops.query.demand_concrete_bounded(
            &facts.types,
            &facts.aggregates,
            call.selection().element,
            limit,
        );
        self.bytes = self
            .bytes
            .saturating_add(drops.query.retained_bytes().saturating_sub(before));
        let value = value?;
        let before = retained::map(&drops.answers);
        drops.answers.insert(call.instance(), value);
        self.bytes = self
            .bytes
            .saturating_add(retained::map(&drops.answers).saturating_sub(before));
        Ok(value)
    }
    fn prepare_drop_query(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        self.prepare_source_results()?;
        if self
            .source_results
            .as_deref()
            .is_some_and(|r| r.drops.is_some())
        {
            return Ok(());
        }
        // The query accounts its embedded record; charge the rest of its owner once.
        let header = size_of::<DropResults>().saturating_sub(size_of::<NeedsDrop>());
        let limit = LIMIT
            .checked_sub(self.bytes.saturating_add(header))
            .ok_or("source needs_drop query storage budget exceeded")?;
        let query = NeedsDrop::concrete_bounded(
            facts.destructors.implementations().iter().map(|i| i.item),
            limit,
        )?;
        let bytes = self
            .bytes
            .saturating_add(header)
            .saturating_add(query.retained_bytes());
        self.source_results
            .as_deref_mut()
            .ok_or("source results disappeared")?
            .drops = Some(Box::new(DropResults {
            answers: HashMap::default(),
            query,
        }));
        self.bytes = bytes;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/needs_drop_cache.rs"]
mod tests;
