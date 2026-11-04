//! External dependency manifests leave `LowIR` and the compiler's typed phases unchanged.
use rustgm::{
    resolve::expansion::Report, sema::model::Declarations, support::linker_libraries::Plan,
};

pub fn prepare(report: &Report, facts: &Declarations, path: Option<&str>) -> Result<Plan, String> {
    let local = facts
        .foreign
        .as_deref()
        .and_then(rustgm::sema::foreign::Registry::native_links);
    let imported = report
        .resolution
        .crates
        .as_ref()
        .into_iter()
        .flat_map(|crates| crates.dependencies().artifacts())
        .filter_map(rustgm::support::crates::Interface::native_links);
    let mut plan = Plan::default();
    for source in local.into_iter().chain(imported) {
        for library in source.libraries() {
            plan.push(library.clone()).map_err(str::to_owned)?;
        }
    }
    if path.is_none() && !plan.libraries().is_empty() {
        return Err(
            "native dependencies require --native-links-output for the backend bridge".into(),
        );
    }
    Ok(plan)
}
pub fn write(plan: &Plan, path: Option<&str>) -> Result<(), String> {
    if let Some(path) = path {
        use std::io::Write as _;
        let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
        let mut output = std::io::BufWriter::new(file);
        plan.write(&mut output).map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}
