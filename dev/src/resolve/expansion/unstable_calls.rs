//! An imported definition row's `#[unstable]` feature gates every call of it,
//! through any path or re-export, and every single `use` import that binds
//! it, called or not. The crate must request the feature, or the call or
//! import must sit in an expansion whose definition may use it, as rustc
//! checks macro stability at the call's own span and at every `use` path.
use super::{Error, Location, MacroAllowanceSession, Report};
use crate::{
    resolve::{
        bindings::FeatureName,
        crates::Imported,
        model::{DefId, Namespace, Origin},
    },
    support::{crates::Interface, hash::HashMap},
};

/// The library requests and the expansion allowances one validation pass
/// consults; the allowance session is opened only for a gated site.
struct Gate<'a> {
    report: &'a Report,
    session: Option<MacroAllowanceSession<'a>>,
    steps: usize,
}

impl Gate<'_> {
    fn check(&mut self, site: Location, feature: &str) -> Result<(), Error> {
        let error = |message| Error::at(site.file, site.span.start, message);
        let name = FeatureName::new(feature).map_err(error)?;
        let requested = self
            .report
            .package
            .library_policy()
            .map(|library| library.requested_feature(&name, &mut self.steps))
            .transpose()
            .map_err(error)?
            .unwrap_or(false);
        if requested
            || self
                .session
                .get_or_insert_with(|| self.report.macro_allowances())
                .allows(site, &name)?
        {
            return Ok(());
        }
        Err(Error::at(
            site.file,
            site.span.start,
            format!("use of unstable library macro requires feature `{feature}`"),
        ))
    }
}

/// Check each call of an imported row, then each single import binding one,
/// once the library requests are complete.
/// # Errors
/// Rejects an unrequested, unallowed feature by name, and a changed row.
pub(in crate::resolve) fn validate(report: &Report) -> Result<(), Error> {
    let Some(crates) = report.resolution.crates.as_deref() else {
        return Ok(());
    };
    let artifacts = crates.dependencies().artifacts();
    let mut gate = Gate {
        report,
        session: None,
        steps: 0,
    };
    for invocation in &report.invocations {
        let site = Location {
            file: invocation.node.file,
            span: invocation.original,
        };
        let error = |message| Error::at(site.file, site.span.start, message);
        report
            .definitions
            .get(invocation.definition)
            .ok_or_else(|| error("macro call names an unknown definition"))?;
        let Some(imported) =
            super::transported_macros::imported(&report.resolution, invocation.definition)
        else {
            continue;
        };
        let feature = artifacts
            .get(imported.artifact)
            .and_then(Interface::macros)
            .and_then(|macros| macros.unstable(imported.row))
            .ok_or_else(|| error("imported macro stability row disappeared"))?;
        if let Some(feature) = feature {
            gate.check(site, feature)?;
        }
    }
    validate_imports(crates, &mut gate)
}

/// A single `use` import of an unstable row needs its feature even when the
/// crate never calls it. A glob import names only its module, whose own
/// stability is checked as a path, so its macro bindings are not gated here.
fn validate_imports(crates: &Imported, gate: &mut Gate<'_>) -> Result<(), Error> {
    let artifacts = crates.dependencies().artifacts();
    let mut gated: HashMap<DefId, &str> = HashMap::default();
    for row in crates.macros() {
        let feature = artifacts
            .get(row.artifact)
            .and_then(Interface::macros)
            .and_then(|macros| macros.unstable(row.row))
            .ok_or_else(|| Error::unlocated("imported macro stability row disappeared"))?;
        if let Some(feature) = feature {
            gated.insert(row.definition, feature);
        }
    }
    if gated.is_empty() {
        return Ok(());
    }
    let report = gate.report;
    for (_, namespace, binding) in report.resolution.bindings.all_bindings() {
        let (Namespace::Macro, Origin::Import(index)) = (namespace, binding.origin) else {
            continue;
        };
        let Some(&feature) = gated.get(&binding.target) else {
            continue;
        };
        let node = report
            .resolution
            .imports
            .get(index as usize)
            .ok_or_else(|| Error::unlocated("macro import binding names an unknown import"))?
            .node;
        let span = report
            .package
            .unit(node.file)
            .configured
            .ast
            .node(node.node)
            .span;
        gate.check(
            Location {
                file: node.file,
                span,
            },
            feature,
        )?;
    }
    Ok(())
}
