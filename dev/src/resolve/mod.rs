mod bodies;
mod imports;
mod index;
pub mod load;
mod lookup;
pub mod model;
pub mod names;
pub mod package;
mod patterns;
pub mod unit;

/// Resolve configured modules into stable scope, definition and reference facts.
///
/// # Errors
/// Reports duplicate declarations, unresolved or private names and invalid imports.
pub fn analyze(package: &mut package::Package) -> Result<model::Resolution, package::Error> {
    let mut resolver = index::Resolver {
        package,
        facts: model::Resolution::default(),
        scratch: Vec::new(),
        pending: Vec::new(),
        visits: Vec::new(),
    };
    resolver.collect()?;
    resolver.imports()?;
    resolver.bodies()?;
    Ok(resolver.facts)
}
pub mod render;
