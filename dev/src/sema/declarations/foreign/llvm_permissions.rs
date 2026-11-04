//! LLVM link permission belongs to the actual foreign item, including macro origins.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{hash::HashSet, retained},
};
#[derive(Default)]
pub(super) struct Draft {
    enabled: Option<bool>,
    foreign: Option<Box<HashSet<NodeKey>>>,
}
impl Draft {
    pub(super) fn require(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let unit = report.package.unit(key.file);
        let span = unit.configured.ast.node(key.node).span;
        let error = |message| Error::at(key.file, span.start, message);
        let enabled = if let Some(enabled) = self.enabled {
            enabled
        } else {
            let enabled = report.package.requested_llvm_linkage().map_err(error)?;
            self.enabled = Some(enabled);
            enabled
        };
        if enabled {
            return Ok(());
        }
        if self.foreign.is_none() {
            let count = report.resolution.foreign_owners.len();
            crate::sema::header_attributes::reserve_source_contracts(
                facts,
                count.saturating_add(1),
                retained::new_map::<NodeKey, ()>(count),
            )?;
            let mut keys = HashSet::default();
            keys.try_reserve(count)
                .map_err(|_| error("foreign linkage index allocation failed"))?;
            for id in report.resolution.foreign_owners.keys() {
                let node = report
                    .resolution
                    .definitions
                    .get(id.0 as usize)
                    .and_then(|item| item.node)
                    .ok_or_else(|| error("foreign linkage lost its source declaration"))?;
                keys.insert(node);
            }
            self.foreign = Some(Box::new(keys));
        }
        if self
            .foreign
            .as_ref()
            .is_some_and(|foreign| foreign.contains(&key))
            && !allowances.allows_at(
                key.file,
                span,
                &FeatureName::new("link_llvm_intrinsics").map_err(error)?,
            )?
        {
            return Err(error(
                "LLVM linkage requires link_llvm_intrinsics or the actual item's macro allowance",
            ));
        }
        Ok(())
    }
}
