//! Pure source-intrinsic results share one cold owner and execution storage budget.
mod queries;
use super::code::Catalog;
use crate::{
    support::{hash::HashMap, retained},
    types::instance::Instance,
};
const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Default)]
pub(super) struct SourceResults {
    layout: HashMap<Instance, u64>,
    drops: Option<Box<queries::DropResults>>,
}
impl Catalog {
    pub(crate) fn layout_result(&self, instance: Instance) -> Option<u64> {
        self.source_results
            .as_deref()?
            .layout
            .get(&instance)
            .copied()
    }
    /// A layout witness carries no authority for bool queries or another caller.
    pub(crate) fn cache_layout(
        &mut self,
        call: &super::SourceLayoutCall,
        value: u64,
    ) -> Result<(), &'static str> {
        if let Some(previous) = self.layout_result(call.instance()) {
            return if previous == value {
                Ok(())
            } else {
                Err("source layout result changed")
            };
        }
        self.prepare_source_results()?;
        let results = self
            .source_results
            .as_deref_mut()
            .ok_or("source results disappeared")?;
        if self.bytes.saturating_add(growth(&results.layout)) > LIMIT {
            return Err("source layout cache storage budget exceeded");
        }
        let before = retained::map(&results.layout);
        results.layout.insert(call.instance(), value);
        self.bytes = self
            .bytes
            .saturating_add(retained::map(&results.layout).saturating_sub(before));
        Ok(())
    }
    fn prepare_source_results(&mut self) -> Result<(), &'static str> {
        if self.source_results.is_none() {
            let bytes = self.bytes.saturating_add(size_of::<SourceResults>());
            if bytes > LIMIT {
                return Err("source intrinsic result storage budget exceeded");
            }
            self.source_results = Some(Box::default());
            self.bytes = bytes;
        }
        Ok(())
    }
}
/// Additional peak payload while both old and replacement tables may be live.
fn growth<K, V>(map: &HashMap<K, V>) -> usize {
    if map.len() < map.capacity() {
        0
    } else {
        retained::new_map::<K, V>(map.len().saturating_add(1))
    }
}
