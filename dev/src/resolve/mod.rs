mod anonymous_parameters;
pub mod bindings;
mod bodies;
#[allow(
    dead_code,
    reason = "private imported-original identities are qualified before body-field migration"
)]
pub(crate) mod body_origin;
mod builtins;
mod collision_retention;
mod container_scopes;
mod contextual_lookup;
pub mod crates;
pub mod expansion;
mod hygiene;
mod identifiers;
mod imports;
mod index;
pub mod load;
mod lookup;
pub mod model;
pub use crate::support::names;
pub mod package;
mod paths;
mod patterns;
mod table;
pub mod unit;

/// Resolve configured modules into stable scope, definition and reference facts.
///
/// # Errors
/// Reports duplicate declarations, unresolved or private names and invalid imports.
pub fn analyze(package: &mut package::Package) -> Result<model::Resolution, package::Error> {
    let mut state = index::State::default();
    let mut resolver = state.bind(package);
    resolver.collect()?;
    resolver.imports()?;
    resolver.bodies()?;
    bindings::finish(package, &mut state.facts)?;
    Ok(state.facts)
}
pub mod render;
pub mod stability;

/// Continue from the expansion graph to ordinary names and mixed-site local facts.
/// The expansion scan already declared every scope and item and drove imports
/// to their fixpoint; this declares what it deferred, finishes imports and
/// walks bodies once.
///
/// # Errors
/// Reports unresolved/private ordinary names, invalid imports and hygiene failures.
pub fn analyze_expanded(
    mut report: expansion::Report,
) -> Result<expansion::Report, package::Error> {
    if report.names_resolved {
        return Ok(report);
    }
    let mut state = index::State::default();
    state.facts = std::mem::take(&mut report.resolution);
    state.hygiene = Some(hygiene::Hygiene::new(
        &report.definitions,
        &report.invocations,
        &state.facts.node_scopes,
    ));
    let pending = std::mem::take(&mut report.pending);
    let mut work = report
        .imports
        .take()
        .unwrap_or_else(|| imports::Work::new(&state.facts));
    let mut resolver = state.bind(&mut report.package);
    let result = resolver
        .primitives()
        .and_then(|()| resolver.declare_pending(&pending, &mut work))
        .and_then(|()| resolver.finish_imports(work))
        .and_then(|()| resolver.bodies());
    result.map_err(|error| report.diagnostic(error))?;
    report.resolution = state.facts;
    bindings::finish(&mut report.package, &mut report.resolution)?;
    report.names_resolved = true;
    Ok(report)
}

pub(crate) mod lint_policy;
