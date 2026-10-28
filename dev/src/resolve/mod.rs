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
mod gated_closures;
mod hygiene;
mod identifiers;
#[path = "module_domains.rs"]
pub(crate) mod impl_restrictions;
mod imports;
mod index;
pub mod load;
mod lookup;
pub mod model;
pub use crate::support::names;
#[path = "unstable_aliases.rs"]
mod alias_gates;
#[path = "specialization_forms.rs"]
mod default_items;
mod float_features;
mod gate_provenance;
pub mod package;
mod paths;
mod patterns;
#[path = "impl_admission.rs"]
pub(crate) mod specialization_authority;
mod table;
mod try_blocks;
pub(crate) use compiler_syntax::require_offset;
pub(crate) use try_blocks::require as require_try_block;
mod compiler_syntax;
pub mod unit;

/// Resolve configured modules into stable scope, definition and reference facts.
///
/// # Errors
/// Reports duplicate declarations, unresolved or private names and invalid imports.
pub fn analyze(package: &mut package::Package) -> Result<model::Resolution, package::Error> {
    default_items::prepare_plain(package)?;
    let mut state = index::State::default();
    let mut resolver = state.bind(package);
    resolver.collect()?;
    resolver.imports()?;
    resolver.bodies()?;
    bindings::finish(package, &mut state.facts)?;
    impl_restrictions::validate_plain(package)?;
    float_features::validate_plain(package, &state.facts)?;
    default_items::validate_plain(package)?;
    alias_gates::validate_plain(package)?;
    try_blocks::validate_plain(package)?;
    gated_closures::validate_plain(package)?;
    compiler_syntax::validate_plain(package)?;
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
    report.resolution = state.facts;
    result.map_err(|error| report.retain_expansion_warnings(report.diagnostic(error)))?;
    bindings::finish(&mut report.package, &mut report.resolution)
        .map_err(|error| report.retain_expansion_warnings(report.diagnostic(error)))?;
    float_features::validate_paths(&report)
        .map_err(|error| report.retain_expansion_warnings(report.diagnostic(error)))?;
    report.names_resolved = true;
    Ok(report)
}

pub(crate) mod lint_policy;
