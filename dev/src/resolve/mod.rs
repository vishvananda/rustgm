mod bodies;
pub mod expansion;
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
