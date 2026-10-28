//! Selected source identities drive deprecation; lexical policy owns diagnostics.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, Origin},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::diagnostics::WarningCode,
    syntax::item::ItemKind,
};

pub(super) fn imports(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
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
pub(super) fn check(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    target: DefId,
) -> Result<(), Error> {
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
        let needs_whole = pending.iter().any(|row| {
            matches!(
                self.safety_policies.completion(row.owner),
                super::safety::Completion::Whole
            )
        });
        let whole_complete = bodies && needs_whole && self.all_lint_bodies_complete(report)?;
        for pending in pending {
            let complete = match self.safety_policies.completion(pending.owner) {
                super::safety::Completion::Declaration => true,
                super::safety::Completion::Function(id) => {
                    bodies && self.typed_function_bodies.contains(&id)
                }
                super::safety::Completion::Initializer(id) => {
                    bodies
                        && self
                            .initializer_index(id)
                            .is_some_and(|index| self.initializers[index].type_checked())
                }
                super::safety::Completion::Whole => bodies && whole_complete,
            };
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

impl Declarations {
    fn all_lint_bodies_complete(&self, report: &Report) -> Result<bool, Error> {
        let Some(producer) = report.lint_producer() else {
            return Ok(true);
        };
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            let Some(key) = definition.node else {
                continue;
            };
            producer.attribute_read(key, 1)?;
            let unit = report.package.unit(key.file);
            let crate::syntax::ast::Kind::Item(item) = unit.configured.ast.node(key.node).kind
            else {
                continue;
            };
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "lint completion definition overflow"))?,
            );
            match unit.configured.ast.items[item as usize].kind {
                ItemKind::Function { body: Some(_), .. }
                    if !self.typed_function_bodies.contains(&id) =>
                {
                    return Ok(false);
                }
                ItemKind::Const { value: Some(_), .. }
                | ItemKind::Static { value: Some(_), .. }
                    if self
                        .initializer_index(id)
                        .is_none_or(|index| !self.initializers[index].type_checked()) =>
                {
                    return Ok(false);
                }
                _ => {}
            }
        }
        Ok(true)
    }
}
