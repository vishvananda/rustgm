//! Actual provider/codec setup shared by the complete module profile controls.
use crate::{
    lower::instances::{SourcePublication, build_source_library_profile},
    pipeline,
    support::crates::{self, generics::source as s},
};
pub(crate) type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/module-contracts-v2/{name}.rs"
    ))?)
}
pub(crate) fn exported(source: &str) -> Result<crates::Interface> {
    exported_profile(
        source,
        SourcePublication::ModuleContracts,
        "module_contracts",
    )
}
pub(crate) fn exported_profile(
    source: &str,
    profile: SourcePublication,
    disambiguator: &str,
) -> Result<crates::Interface> {
    let report = crate::test_source::source_provider(source, disambiguator)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let (_, interface) =
        build_source_library_profile(&report, &mut facts, identity, prepared, profile)?;
    Ok(interface)
}
pub(crate) fn roundtrip(source: &str) -> Result<crates::Interface> {
    let interface = exported(source)?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(13..15), Some([2, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    Ok(decoded)
}
pub(crate) fn hex(path: &str) -> Result<Vec<u8>> {
    let text = crate::test_source::fixture(path)?;
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(str::split_ascii_whitespace)
        .map(|x| u8::from_str_radix(x, 16).map_err(Into::into))
        .collect()
}
pub(crate) type Raw = (
    s::SourceRows,
    s::HeaderAttributeColumns,
    s::ModuleContractRows,
);
pub(crate) fn raw(interface: &crates::Interface) -> Result<Raw> {
    let descriptor = interface
        .source_descriptor()
        .ok_or("actual module descriptor")?;
    let view = descriptor.rows();
    Ok((
        raw_rows(view),
        view.header_attributes()
            .ok_or("complete advisory columns")?
            .clone(),
        descriptor.module_contracts()?.rows().clone(),
    ))
}
pub(crate) fn raw_rows(view: s::ValidatedView<'_>) -> s::SourceRows {
    let c = (*view).clone();
    s::SourceRows {
        body_owners: view.body_owners().to_vec(),
        policy: c.policy,
        declarations: c.declarations,
        slots: c.slots,
        scopes: c.scopes,
        binders: c.binders,
        types: c.types,
        roots: c.roots,
        regions: c.regions,
        arguments: c.arguments,
        clauses: c.clauses,
        clause_indices: c.clause_indices,
        region_bounds: c.region_bounds,
        region_slots: c.region_slots,
        trait_applications: c.trait_applications,
        function_applications: c.function_applications,
        traits: c.traits,
        implementations: c.implementations,
        functions: c.functions,
        formals: c.formals,
        members: c.members,
        member_links: c.member_links,
        roles: c.roles,
        originals: c.originals,
        body_inputs: c.body_inputs,
        body_units: c.body_units,
        exact_callables: c.exact_callables,
    }
}
