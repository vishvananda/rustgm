//! Actual imported Contract and Declaration environments keep distinct ownership.
use super::*;
use crate::source_packets;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn require(facts: &Declarations, definition: DefId, steps: &mut usize) -> Result {
    facts
        .crates
        .as_deref()
        .ok_or("actual imported linkage")?
        .require_source_declaration(
            facts,
            definition,
            facts
                .lifetimes
                .functions
                .get(&definition)
                .ok_or("actual callable scheme")?,
            steps,
        )?;
    Ok(())
}
fn replace_environment(
    facts: &mut Declarations,
    definition: DefId,
    environment: Arc<Environment>,
) -> Result<Arc<Environment>> {
    let header = facts
        .crates
        .as_mut()
        .and_then(|linkage| linkage.originals.as_mut())
        .and_then(|headers| headers.entries.get_mut(&definition))
        .ok_or("actual retained imported header")?;
    Ok(std::mem::replace(&mut header.environment, environment))
}
fn actual_origins(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
) -> Result<Vec<DefId>> {
    let linkage = facts.crates.as_deref().ok_or("actual linkage")?;
    linkage
        .originals
        .as_deref()
        .and_then(|headers| headers.source.as_deref())
        .ok_or("actual canonical graph")?;
    let catalog = linkage.origin_catalog().ok_or("actual catalog")?;
    let descriptor = report
        .resolution
        .crates
        .as_deref()
        .ok_or("actual import inventory")?
        .dependencies()
        .artifacts()
        .iter()
        .find_map(crate::support::crates::Interface::source_descriptor)
        .ok_or("actual complete source descriptor")?;
    let rows = descriptor.rows();
    let mut definitions = Vec::new();
    for name in ["clone", "identity", "plain"] {
        let raw = rows
            .functions
            .iter()
            .find(|header| {
                rows.declarations[header.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
            })
            .ok_or("actual named function row")?;
        match (name, raw.body) {
            ("clone", crate::support::crates::generics::source::FunctionBody::Absent) => {}
            (_, crate::support::crates::generics::source::FunctionBody::Original(body)) => {
                let origin = rows.body_inputs[body.0 as usize].environment;
                assert_eq!(
                    origin,
                    if name == "plain" {
                        crate::support::crates::generics::source::EnvironmentOrigin::Contract
                    } else {
                        crate::support::crates::generics::source::EnvironmentOrigin::Declaration(
                            raw.declaration,
                        )
                    }
                );
            }
            _ => return Err("actual required method acquired a body".into()),
        }
        definitions.push(catalog.definition_at(descriptor.identity(), raw.declaration)?);
    }
    Ok(definitions)
}

#[test]
fn imported_method_environment_validation_preserves_actual_contract_origin() -> Result {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    source.push_str("pub fn plain(value:u8)->u8{value}");
    let encoded = crate::support::crates::encode(&source_packets::exported(&source)?)?;
    let interface = crate::support::crates::decode(&encoded)?;
    let (report, mut analysis, _) = source_packets::client_analysis(
        interface,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn idle(){}",
    )?;
    let facts = &mut analysis.facts;
    let definitions = actual_origins(&report, facts)?;
    let mut steps = 0;
    for (index, &definition) in definitions.iter().enumerate() {
        require(facts, definition, &mut steps)?;
        let declaration = Arc::clone(
            facts
                .generics
                .environments
                .get(&definition)
                .ok_or("actual declaration environment")?,
        );
        let body = Arc::clone(
            facts
                .function_header(definition)?
                .and_then(|header| header.environment)
                .ok_or("actual body environment")?,
        );
        assert_eq!(Arc::ptr_eq(&body, &declaration), index != 2);
        let wrong = if index == 2 {
            Arc::clone(&declaration)
        } else {
            Arc::clone(
                facts
                    .function_header(definitions[2])?
                    .and_then(|header| header.environment)
                    .ok_or("actual distinct Contract environment")?,
            )
        };
        let original = replace_environment(facts, definition, wrong)?;
        let before = steps;
        assert!(require(facts, definition, &mut steps).is_err());
        assert!(steps > before);
        replace_environment(facts, definition, original)?;
        let removed = facts
            .generics
            .environments
            .remove(&definition)
            .ok_or("actual declaration environment removed")?;
        assert!(require(facts, definition, &mut steps).is_err());
        facts.generics.environments.insert(definition, removed);
        require(facts, definition, &mut steps)?;
    }
    crate::sema::traits::conformance::Method::imported(facts, definitions[0])?;
    let arena = std::mem::take(&mut facts.types);
    assert!(require(facts, definitions[0], &mut steps).is_err());
    facts.types = arena;
    require(facts, definitions[0], &mut steps)?;
    assert!(steps > 0);
    Ok(())
}
