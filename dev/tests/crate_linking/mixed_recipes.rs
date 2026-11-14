//! Version fourteen keeps node originals and nonexecutable operations in one packet.
use super::fresh_packets::{Packet, Result};
use crate::resolve as source;
use rustgm::{
    lower::instances::{SourcePublication, build_source_library_profile},
    pipeline,
    support::crates::{self, generics::source as s},
};
const SPEC: &str = "pa43/tests/spec/body-operations";
const PROFILE: &str = "module-body-operations-v14";
fn interface(name: &str) -> Result<(crates::Interface, [usize; 2])> {
    let text = source::fixture(&format!("{SPEC}/{name}.rs"))?;
    publish(&text, "body_operations_v14")
}
fn publish(text: &str, tag: &str) -> Result<(crates::Interface, [usize; 2])> {
    let report = source::source_provider(text, tag)?;
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
    let (_, interface) = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )?;
    Ok((
        interface,
        [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ],
    ))
}
#[test]
fn mixed_node_originals_and_operation_retain_distinct_actual_owners() -> Result {
    for name in ["provider", "provider-last"] {
        let (interface, _) = interface(name)?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(bytes.get(12..15), Some([10, 14, 0].as_slice()));
        let decoded = crates::decode(&bytes)?;
        assert_eq!(crates::encode(&decoded)?, bytes);
        let descriptor = decoded.source_descriptor().ok_or("mixed descriptor")?;
        assert!(descriptor.profile().body_nodes());
        let rows = descriptor.compiler_operations()?;
        let [operation] = rows.operations() else {
            return Err("one genuine operation".into());
        };
        assert!(matches!(
            rows.function_body(operation.declaration)?,
            s::FunctionBody::CompilerOperation(_)
        ));
        assert!(!rows.ordinary_bodies().any(|body| body == operation.body));
        assert!(rows.ordinary_bodies().len() > 4);
        crate::compiler_recipes::client(crates::decode(&bytes)?, "consumer")?;
        for (name, cause) in [
            ("unsafe-consumer", "unsafe context"),
            ("wrong-output-consumer", "incompatible types"),
        ] {
            let error = crate::compiler_recipes::client(crates::decode(&bytes)?, name)
                .err()
                .ok_or_else(|| format!("accepted {name}"))?;
            assert!(error.to_string().contains(cause), "{name}: {error}");
        }
        let (
            report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = crate::compiler_recipes::client_source(
            crates::decode(&bytes)?,
            source::fixture(&format!("{SPEC}/lifetime-consumer.rs"))?,
        )?;
        let error =
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .err()
                .ok_or("borrowed helper authorized a static future")?;
        assert!(
            error
                .to_string()
                .contains("required lifetime does not outlive its target"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn mixed_wire_rejects_collisions_forged_results_and_legacy_relabelling() -> Result {
    let (interface, _) = interface("provider")?;
    let rows = interface
        .source_descriptor()
        .ok_or("mixed descriptor")?
        .compiler_operations()?;
    let operation = rows.operations().first().ok_or("operation")?;
    let ordinary = rows.ordinary_bodies().next().ok_or("ordinary body")?;
    let bytes = crates::encode(&interface)?;
    for (name, tail, value) in [
        ("ordinary body collision", 21, ordinary.0),
        ("foreign owner", 17, u32::MAX),
        ("wrong future", 13, operation.output.0),
        ("nonunit Output", 9, operation.future.0),
        ("wrong target formal", 4, 1),
    ] {
        let mut changed = bytes.clone();
        let at = changed.len() - tail;
        changed[at..at + 4].copy_from_slice(&value.to_le_bytes());
        assert!(crates::decode(&changed).is_err(), "accepted {name}");
    }
    let mut unknown = bytes.clone();
    let at = unknown.len() - 5;
    unknown[at] = 255;
    assert!(crates::decode(&unknown).is_err());
    let mut missing = bytes.clone();
    let at = missing.len() - 25;
    missing[at..at + 4].copy_from_slice(&0_u32.to_le_bytes());
    missing.truncate(at + 4);
    assert!(crates::decode(&missing).is_err());
    for version in [11_u16, 13] {
        let mut changed = bytes.clone();
        changed[13..15].copy_from_slice(&version.to_le_bytes());
        assert!(crates::decode(&changed).is_err());
    }
    assert!(crates::decode(&bytes[..bytes.len() - 1]).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    Ok(())
}
#[test]
fn mixed_publication_work_and_output_scale_with_ordinary_bodies() -> Result {
    let mut previous: Option<[usize; 3]> = None;
    for count in [8, 32, 128] {
        let (interface, [inference, canonical]) = interface(&format!("scaling-{count}"))?;
        let bytes = crates::encode(&interface)?;
        let decoded = crates::decode(&bytes)?;
        let rows = decoded
            .source_descriptor()
            .ok_or("scaled mixed descriptor")?
            .compiler_operations()?;
        assert_eq!(rows.operations().len(), 1);
        assert_eq!(rows.ordinary_bodies().len(), count);
        let work = [inference, canonical, bytes.len()];
        if let Some(old) = previous {
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        println!("mixed bodies={count} work/bytes={work:?}");
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned object backend and host C linker"]
fn ordinary_node_body_executes_from_mixed_metadata_in_a_fresh_cli() -> Result {
    for provider in ["provider", "provider-last"] {
        let packet = Packet::new("body-operations", SPEC, provider, "provider")?;
        super::artifacts::checked(
            "provider",
            &packet.folder,
            &mut packet.export(PROFILE, &packet.fixture(provider), &packet.metadata),
        )?;
        super::artifacts::checked(
            "ordinary-client",
            &packet.folder,
            &mut packet.client("ordinary-client"),
        )?;
        let output = packet.native_with(
            "ordinary-client",
            &source::fixture(&format!("{SPEC}/observe.c"))?,
        )?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(output.stdout, b"41\n");
    }
    Ok(())
}

const DECLARATIONS: &str = "pa43/tests/spec/mixed-declarations";
const FAMILIES: [&str; 4] = [
    "struct-constructors",
    "enum-match",
    "const-parameters",
    "named-constants",
];

fn mixed_body_owners(interface: &crates::Interface, operations: usize) -> Result {
    let descriptor = interface
        .source_descriptor()
        .ok_or("declaration descriptor")?;
    let rows = descriptor.compiler_operations()?;
    assert_eq!(rows.operations().len(), operations);
    assert!(rows.ordinary_bodies().len() > 2);
    for operation in rows.operations() {
        assert!(!rows.ordinary_bodies().any(|body| body == operation.body));
        assert!(matches!(
            rows.function_body(operation.declaration)?,
            s::FunctionBody::CompilerOperation(_)
        ));
    }
    Ok(())
}

fn helper_conditions(bytes: &[u8]) -> Result {
    crate::compiler_recipes::client(crates::decode(bytes)?, "consumer")?;
    for (name, cause) in [
        ("unsafe-consumer", "unsafe context"),
        ("wrong-output-consumer", "incompatible types"),
    ] {
        let error = crate::compiler_recipes::client(crates::decode(bytes)?, name)
            .err()
            .ok_or("accepted invalid helper call")?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = crate::compiler_recipes::client_source(
        crates::decode(bytes)?,
        source::fixture("pa43/tests/spec/body-operations/lifetime-consumer.rs")?,
    )?;
    let error =
        rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
            .err()
            .ok_or("helper erased its borrowed lifetime")?;
    assert!(
        error
            .to_string()
            .contains("required lifetime does not outlive its target"),
        "{error}"
    );
    Ok(())
}

#[test]
fn mixed_profile_retains_all_legacy_declaration_kinds_with_and_without_operations() -> Result {
    for family in FAMILIES {
        for mixed in [false, true] {
            let packet = Packet::new(
                "mixed-declarations",
                DECLARATIONS,
                &format!("{family}-{mixed}"),
                "provider",
            )?;
            let input = if mixed {
                packet.fixture(family)
            } else {
                packet
                    .root
                    .join(format!("pa29/tests/spec/{family}/provider.rs"))
            };
            super::artifacts::checked(
                "provider",
                &packet.folder,
                &mut packet.export(PROFILE, &input, &packet.metadata),
            )?;
            let bytes = std::fs::read(&packet.metadata)?;
            assert_eq!(bytes.get(12..15), Some([10, 14, 0].as_slice()));
            let decoded = crates::decode(&bytes)?;
            assert_eq!(crates::encode(&decoded)?, bytes);
            mixed_body_owners(&decoded, usize::from(mixed))?;
            if mixed {
                super::artifacts::checked(
                    "client",
                    &packet.folder,
                    &mut packet.client(&format!("{family}-client")),
                )?;
                let cause = if matches!(family, "struct-constructors" | "named-constants") {
                    "private"
                } else {
                    "incompatible types"
                };
                packet.rejected(
                    "opponent",
                    &mut packet.client(&format!("{family}-reject")),
                    cause,
                )?;
                helper_conditions(&bytes)?;
            }
        }
    }
    Ok(())
}

#[test]
fn mixed_const_parameters_and_unit_constructors_scale_with_actual_bodies() -> Result {
    let mut previous: Option<[usize; 3]> = None;
    for count in [8, 32, 128] {
        let text = source::fixture(&format!("{DECLARATIONS}/scaling-{count}.rs"))?;
        let (interface, [inference, canonical]) = publish(&text, "mixed_declaration_scaling")?;
        let bytes = crates::encode(&interface)?;
        let decoded = crates::decode(&bytes)?;
        let descriptor = decoded.source_descriptor().ok_or("scaled declarations")?;
        let rows = descriptor.compiler_operations()?;
        assert_eq!(rows.operations().len(), 1);
        assert_eq!(rows.ordinary_bodies().len(), count);
        let work = [inference, canonical, bytes.len()];
        println!("mixed declarations={count} work/bytes={work:?}");
        if let Some(old) = previous {
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned object backend and host C linker"]
fn mixed_declarations_execute_four_unchanged_legacy_consumers() -> Result {
    for family in FAMILIES {
        let packet = Packet::new("mixed-declarations", DECLARATIONS, family, "provider")?;
        super::artifacts::checked(
            "provider",
            &packet.folder,
            &mut packet.export(PROFILE, &packet.fixture(family), &packet.metadata),
        )?;
        let client = format!("{family}-client");
        super::artifacts::checked("client", &packet.folder, &mut packet.client(&client))?;
        let output = packet.native_with(
            &client,
            &source::fixture(&format!("{DECLARATIONS}/observe.c"))?,
        )?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(
            output.stdout,
            source::fixture(&format!("{DECLARATIONS}/expected.stdout"))?.as_bytes()
        );
    }
    Ok(())
}
