//! Selected source identities drive deprecation; lexical policy owns diagnostics.
mod completion;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, Origin},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::diagnostics::WarningCode,
};

pub(super) fn imports(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    compatibility(report, facts, None)?;
    for (_, _, binding) in report.resolution.bindings.all_bindings() {
        let Origin::Import(index) = binding.origin else {
            continue;
        };
        let import = report
            .resolution
            .imports
            .get(index as usize)
            .ok_or_else(|| Error::unlocated("deprecated import lost actual identity"))?;
        check(report, facts, import.node, binding.target)?;
    }
    for (index, import) in report.resolution.imports.iter().enumerate() {
        if let Some(binding) = report.resolution.anonymous_types.get(&index) {
            check(report, facts, import.node, binding.target)?;
        }
    }
    Ok(())
}
pub(in crate::sema) fn compatibility(
    report: &Report,
    facts: &mut Declarations,
    site: Option<(NodeKey, DefId)>,
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    crate::resolve::stability::visit_compatibility_warnings(report, site, |warning| {
        if let Some(producer) = report.lint_producer() {
            producer.diagnostic_read(warning.site, warning.message.len().saturating_add(40))?;
        }
        let offset = report
            .package
            .unit(warning.site.file)
            .configured
            .ast
            .node(warning.site.node)
            .span
            .start;
        facts.must_use_diagnostic(
            report,
            warning.site,
            offset,
            warning.prefix.0,
            WarningCode::Deprecated,
            format!(
                "use of stable declaration through an unstable module: {}",
                warning.message
            ),
        )
    })
}
pub(super) fn check(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    target: DefId,
) -> Result<(), Error> {
    compatibility(report, facts, Some((site, target)))?;
    let declared = report
        .resolution
        .definitions
        .get(target.0 as usize)
        .ok_or_else(|| Error::at(site.file, 0, "deprecated use lost actual declaration"))?;
    let Some(key) = declared.node else {
        // This finite producer admits local/source identities. Source-free
        // artifacts retain the legacy profile; an explicit deprecation request
        // cannot treat their missing contract as known absence.
        if !facts.safety_policies.explicit_deprecation(site)
            || facts.safety_policies.deprecated(site) == super::safety::Level::Allow
        {
            return Ok(());
        }
        if crate::resolve::stability::declaration_deprecation_absent(report, target)? {
            return Ok(());
        }
        if super::header_attributes::bootstrap_owner(report, target)? {
            return Ok(());
        }
        return Err(Error::at(
            site.file,
            0,
            "imported deprecation contract transport is not implemented",
        ));
    };
    let Some(producer) = report.lint_producer() else {
        return Ok(());
    };
    let Some(origin) = facts.safety_policies.deprecated_origin(key) else {
        return Ok(());
    };
    let note = producer.deprecation_note(&report.package, origin, key)?;
    if facts.safety_policies.deprecated_origin(site) == Some(origin) {
        return Ok(());
    }
    producer.diagnostic_read(site, note.len().saturating_add(40))?;
    let message = format!("use of deprecated declaration: {note}");
    let offset = report
        .package
        .unit(site.file)
        .configured
        .ast
        .node(site.node)
        .span
        .start;
    facts.must_use_diagnostic(
        report,
        site,
        offset,
        target.0,
        WarningCode::Deprecated,
        message,
    )
}

impl Declarations {
    /// Close only lint producers completed by this request. Header-only callers
    /// close import owners; a complete body request may close all source owners.
    pub(crate) fn finalize_lints(&mut self, report: &Report, bodies: bool) -> Result<(), Error> {
        self.require_lint_context(report)?;
        if report.package.lint_cap().suppresses_lints() {
            return Ok(());
        }
        let Some(producer) = report.lint_producer() else {
            return Ok(());
        };
        if !producer.has_expectations() {
            return Ok(());
        }
        let pending = producer.pending(
            &report.package,
            self.safety_policies
                .progress
                .as_deref()
                .ok_or_else(|| Error::unlocated("lint request progress is missing"))?,
        )?;
        let scopes = completion::Scopes::collect(report, self, &pending, bodies)?;
        for pending in pending {
            let complete = pending.resolution_complete && report.names_resolved
                || scopes.completed(
                    self,
                    self.safety_policies.completion(pending.owner),
                    pending.id,
                    bodies,
                );
            if !complete {
                continue;
            }
            let prefix = if pending.self_expect {
                "this lint expectation is unfulfilled; unfulfilled_lint_expectations cannot be expected"
            } else {
                "this lint expectation is unfulfilled"
            };
            producer.diagnostic_read(
                pending.owner,
                prefix
                    .len()
                    .saturating_add(pending.reason.map_or(0, str::len))
                    .saturating_add(2),
            )?;
            let message = pending
                .reason
                .map_or_else(|| prefix.to_owned(), |reason| format!("{prefix}: {reason}"));
            self.must_use_diagnostic(
                report,
                pending.owner,
                pending.span.start,
                pending.ordinal,
                WarningCode::UnfulfilledLintExpectations,
                message,
            )?;
            producer.finalized(
                &report.package,
                pending.id,
                pending.owner,
                self.safety_policies
                    .progress
                    .as_deref_mut()
                    .ok_or_else(|| Error::unlocated("lint request progress is missing"))?,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/compatibility_paths.rs"]
mod compatibility_tests;
