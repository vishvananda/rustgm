//! A selected override can bind more late regions than its actual trait call.
use crate::{
    lower::instances::SourcePublication,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
    types::{Kind, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source(fixture: &str) -> Result<String> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/source-enum-role-transport/provider.rs")?;
    let prefix = foundation
        .split_once("#[rustc_diagnostic_item = \"Option\"]")
        .ok_or("actual foundation boundary")?
        .0;
    let tail = crate::test_source::fixture(&format!("pa31/tests/spec/{fixture}/provider.rs"))?;
    Ok(format!("#![feature(auto_traits)]\n{prefix}{tail}"))
}

#[test]
fn trait_call_openings_survive_override_binder_changes_and_reject_forgery() -> Result {
    let text = source("source-override-late-binders")?;
    let report = crate::test_source::source_provider(&text, "override_openings")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, artifact) = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )?;
    let count = |ty: TypeId| match facts.types.kind(ty) {
        Some(Kind::FunctionItem { signature, .. }) => match facts.types.kind(signature) {
            Some(Kind::Function { bound_regions, .. }) => Some(bound_regions),
            _ => None,
        },
        _ => None,
    };
    assert!(
        facts.body_regions.log().iter().any(|row| {
            facts
                .methods
                .calls
                .get(&row.region.source)
                .is_some_and(|method| {
                    count(method.callable) == Some(2)
                        && facts
                            .methods
                            .dispatched(row.region.body, row.region.source, method.callable)
                            .is_some_and(|selected| count(selected) == Some(3))
                })
        }),
        "the fixture must actually select a differently bound override"
    );
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->bool{base::references(&11,&22)}",
    )?;
    check_opening_forgeries(&artifact)
}

fn check_opening_forgeries(artifact: &crates::Interface) -> Result {
    let (rows, attributes, modules) = crate::module_packets::raw(artifact)?;
    let availability = artifact
        .source_descriptor()
        .ok_or("actual descriptor")?
        .availability()?
        .clone();
    let (body, request) = rows
        .originals
        .iter()
        .enumerate()
        .find_map(|(body, original)| {
            original
                .calls
                .iter()
                .enumerate()
                .find_map(|(request, call)| {
                    let declaration = rows.function_applications[call.target.0 as usize]
                        .declaration
                        .declaration;
                    let header = rows
                        .functions
                        .iter()
                        .find(|header| header.declaration == declaration)?;
                    let parent = header.parent?;
                    let goal = rows.trait_applications.get(call.method?.0 as usize)?;
                    (rows.declarations[parent.0 as usize].name.as_deref() == Some("PartialEq")
                        && matches!(
                            rows.types.get(goal.subject.0 as usize),
                            Some(s::Type::Reference { .. })
                        ))
                    .then_some((body, request))
                })
        })
        .ok_or("actual retained trait call")?;
    let call = rows.originals[body].calls[request];
    let opening = call.opening.ok_or("actual trait opening")?;
    assert_eq!(opening.regions.end - opening.regions.start, 2);
    let foreign = rows
        .functions
        .iter()
        .find(|header| {
            header.callable.binder != opening.binder
                && header.regions.late.end - header.regions.late.start == 2
        })
        .ok_or("equal-arity foreign binder")?
        .callable
        .binder;
    for case in 0..3 {
        let mut forged = rows.clone();
        let opening = forged.originals[body].calls[request]
            .opening
            .as_mut()
            .ok_or("opening")?;
        match case {
            0 => opening.binder = foreign,
            1 => opening.regions.end -= 1,
            _ => forged.originals[body].calls[request].opening = None,
        }
        assert!(
            g::Payload::from_source_module_body_operations(
                artifact.identity(),
                forged,
                attributes.clone(),
                modules.clone(),
                availability.clone(),
                0,
                &mut g::Work::default(),
            )
            .is_err(),
            "forged opening {case}"
        );
    }
    Ok(())
}

#[test]
fn shared_method_sites_keep_separate_protocol_targets_and_complete_openings() -> Result {
    let text = source("source-shared-method-sites")?;
    let artifact = crate::module_packets::exported_profile(
        &text,
        SourcePublication::ModuleBodyOperations,
        "shared_method_sites",
    )?;
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    let (rows, _, _) = crate::module_packets::raw(&fresh)?;
    for (name, count) in [
        ("through", 2),
        ("through_nested", 3),
        ("through_generic", 2),
        ("through_trait", 2),
    ] {
        let original = rows
            .originals
            .iter()
            .find(|body| {
                rows.declarations[body.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
            })
            .ok_or("actual shared-site method body")?;
        assert_eq!(original.calls.len(), count, "{name}");
        let site = original.calls[0].site;
        let mut openings = Vec::new();
        let mut targets = Vec::new();
        for call in &original.calls {
            assert_eq!(
                call.site, site,
                "calls must really share their semantic source node"
            );
            let opening = call.opening.ok_or("actual call opening")?;
            openings.push(opening.regions.slice(&original.openings)?.len());
            let target = rows.function_applications[call.target.0 as usize]
                .declaration
                .declaration;
            targets.push(
                rows.declarations[target.0 as usize]
                    .name
                    .as_deref()
                    .ok_or("target name")?,
            );
        }
        assert_eq!(openings.last(), Some(&2));
        assert!(openings[..count - 1].iter().all(|&count| count == 1));
        assert!(targets[..count - 1].iter().all(|&name| name == "deref"));
        assert_eq!(
            targets[count - 1],
            if name == "through_trait" {
                "select"
            } else {
                "choose"
            }
        );
    }
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->u32{base::through(&base::Outer(base::Value(11)),&base::Value(22),true)}",
    )?;
    Ok(())
}
