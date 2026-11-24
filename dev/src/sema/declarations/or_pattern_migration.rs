//! Diagnose migration only for macros defined in this source compilation.
use crate::{
    resolve::{
        expansion::{Body, Report},
        lint_policy::Level,
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{diagnostics::WarningCode, edition::Edition},
};

pub(super) fn check(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if !facts.safety_policies.checks_legacy_pattern_lint() {
        return Ok(());
    }
    let mut steps = 0_usize;
    for macro_ in &report.definitions {
        if !matches!(macro_.edition, Edition::Rust2015 | Edition::Rust2018) {
            continue;
        }
        let Some(key) = macro_.node else {
            continue;
        };
        let root = report.package.file_crate_root(key.file);
        let module = &report.package.modules[root.0 as usize];
        let scope = NodeKey {
            file: module.file,
            node: module.root,
        };
        if facts
            .safety_policies
            .rust_2021_incompatible_or_patterns(scope)
            == Level::Allow
        {
            continue;
        }
        let definition = match &macro_.body {
            Body::Declarative(compiled) => &compiled.definition,
            Body::Scoped { compiled, .. } => &compiled.definition,
            Body::Builtin(_) => continue,
        };
        steps = steps
            .saturating_add(definition.nodes.len())
            .saturating_add(1);
        if steps > 1_000_000 {
            return Err(Error::at(
                key.file,
                0,
                "or-pattern lint work budget exceeded",
            ));
        }
        for offset in report.legacy_pattern_sites(key, definition)? {
            facts.must_use_diagnostic_at(
                report,
                (scope, key),
                offset,
                0,
                WarningCode::Rust2021IncompatibleOrPatterns,
                (
                    "use pat_param for a single alternative in a pre-2021 macro matcher".into(),
                    macro_.introduction,
                ),
            )?;
        }
    }
    Ok(())
}
