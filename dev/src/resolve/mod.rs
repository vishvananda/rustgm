mod anonymous_parameters;
mod bodies;
mod builtins;
pub mod expansion;
mod hygiene;
mod imports;
mod index;
pub mod load;
mod lookup;
pub mod model;
pub use crate::support::names;
pub mod package;
mod patterns;
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
    Ok(state.facts)
}
pub mod render;

/// Continue from the expansion graph to ordinary names and mixed-site local facts.
/// Earlier macro imports and module identities are consumed directly.
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
    ));
    let mut resolver = state.bind(&mut report.package);
    let result = resolver
        .collect()
        .and_then(|()| resolver.imports())
        .and_then(|()| resolver.bodies());
    result.map_err(|error| report.diagnostic(error))?;
    report.resolution = state.facts;
    report.names_resolved = true;
    Ok(report)
}
