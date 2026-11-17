//! Source prelude traits reuse the ordinary per-scope member index.
use super::{DefId, ItemContext, NameId, Report, ScopeId};
impl super::Index {
    pub(super) fn source_prelude_members(
        &mut self,
        report: &Report,
        scope: ScopeId,
        name: NameId,
        context: ItemContext,
        result: &mut Vec<DefId>,
    ) -> Result<(), &'static str> {
        let module = report.resolution.scopes[scope.0 as usize].module;
        if !report.package.modules[module.0 as usize].implicit_prelude {
            return Ok(());
        }
        // Method tokens use the same invoking crate prelude as type/value
        // fallback. Lexical definition-site traits retain their own index above.
        let selected = report
            .resolution
            .selected_crate_prelude(&report.package, module)?;
        if report.package.modules[module.0 as usize].krate
            == crate::resolve::guests::CrateSlot::HOST
            && selected != self.source_prelude
        {
            return Err("source prelude changed after its method index was built");
        }
        let Some(scope) = selected else {
            return Ok(());
        };
        self.index_scope(scope)?;
        let members = self.contextual_local(report, scope, name, context)?;
        let count = result.len().saturating_add(members.len());
        self.account(count.saturating_mul(1 + count.checked_ilog2().unwrap_or(0) as usize))?;
        result.extend(members);
        result.sort_unstable_by_key(|id| id.0);
        result.dedup();
        Ok(())
    }
}
