//! A serialized compiler-operation recipe supplies contracts, never a future CFG.
use crate::resolve as source;
use rustgm::{
    expand::config::Config,
    lower::instances::{SourcePublication, build_source_library_profile},
    pipeline, resolve,
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency, generics::source as s},
        source::SourceFile,
    },
    types::Kind,
};
use std::{path::Path, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(source::fixture(&format!(
        "pa43/tests/spec/deferred-compiler-op/{name}.rs"
    ))?)
}
fn exported(name: &str) -> Result<crates::Interface> {
    let report = source::source_provider(&fixture(name)?, "deferred_op_v11")?;
    let identity = report
        .package
        .source_core()
        .ok_or("source provider")?
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
        SourcePublication::CompilerOperations,
    )?;
    Ok(interface)
}
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn client(
    interface: crates::Interface,
    name: &str,
) -> Result<(resolve::expansion::Report, pipeline::Analysis)> {
    let provider = interface.identity().clone();
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("minicore".into(), provider.clone())?],
    )?);
    let current = CrateIdentity::new("client", "deferred_op_v11")?;
    let context = resolve::bindings::Context {
        crate_name: Some("client".into()),
        current: Some(current),
        dependencies: Some(dependencies),
        mode: Mode::SourceClient { provider },
    };
    let input = resolve::load::InputFile {
        identity: "recipe-client.rs".into(),
        path: "recipe-client.rs".into(),
        source: SourceFile::new(fixture(name)?.into_bytes())?,
    };
    let expanded = resolve::expansion::analyze_with_context(
        input,
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    assert_eq!(report.package.units.len(), 1);
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn serialized_operation_keeps_real_owner_and_fresh_output_proof() -> Result {
    let interface = exported("provider")?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(12), Some(&13));
    assert_eq!(bytes.get(13..15), Some([11, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let rows = decoded
        .source_descriptor()
        .ok_or("source descriptor")?
        .compiler_operations()?;
    assert_eq!(rows.operations().len(), 1);
    let operation = &rows.operations()[0];
    assert!(matches!(
        rows.function_body(operation.declaration)?,
        s::FunctionBody::CompilerOperation(_)
    ));
    assert!(!rows.ordinary_bodies().any(|body| body == operation.body));
    assert_eq!(rows.ordinary_bodies().len(), 1);
    assert_ne!(rows.ordinary_bodies().next().ok_or("ordinary body")?.0, 0);
    assert_eq!(rows.ordinary_input_count(), rows.ordinary_bodies().len());
    let (report, analysis) = client(decoded, "consumer")?;
    let facts = analysis.facts;
    let imported = facts
        .opaques
        .owners
        .iter()
        .filter(|(owner, _)| {
            report.resolution.definitions[owner.0 as usize]
                .node
                .is_none()
        })
        .collect::<Vec<_>>();
    assert_eq!(imported.len(), 1);
    assert_eq!(imported[0].1.len(), 1);
    let identity = imported[0].1[0];
    let applications = facts
        .nodes
        .iter()
        .flatten()
        .filter_map(|node| node.ty)
        .filter_map(|ty| match facts.types.kind(ty) {
            Some(Kind::Opaque(id, args)) if id == identity => Some(args),
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>();
    assert!(
        applications.len() >= 2,
        "distinct T/lifetime applications were erased"
    );
    for application in applications {
        let captures = facts
            .types
            .arguments(application)
            .ok_or("actual captures")?
            .collect::<Vec<_>>();
        assert!(matches!(
            captures.as_slice(),
            [
                rustgm::types::arguments::Argument::Lifetime(_),
                rustgm::types::arguments::Argument::Type(_)
            ]
        ));
    }
    let declaration = facts
        .opaques
        .declarations
        .get(&identity)
        .ok_or("actual imported opaque contract")?;
    assert!(declaration.hidden.is_none());
    assert_eq!(declaration.bounds.len(), 2);
    Ok(())
}
#[test]
fn wire_operation_rejects_changed_body_owner_result_target_and_opcode() -> Result {
    let interface = exported("provider")?;
    let bytes = crates::encode(&interface)?;
    let view = interface
        .source_descriptor()
        .ok_or("recipe descriptor")?
        .compiler_operations()?;
    let operation = view.operations().first().ok_or("actual operation")?;
    // v11's final operation table is count + the exact typed 21-byte opcode row.
    let start = bytes.len() - 21;
    assert_eq!(&bytes[start..start + 4], &operation.body.0.to_le_bytes());
    for (label, tail, value) in [
        ("ordinary body collision", 21, 1),
        ("foreign owner", 17, u32::MAX),
        ("wrong constructor future", 13, operation.output.0),
        ("nonunit Output", 9, operation.future.0),
        ("wrong target formal", 4, 1),
    ] {
        let mut changed = bytes.clone();
        let offset = changed.len() - tail;
        changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(crates::decode(&changed).is_err(), "accepted {label}");
    }
    let mut unknown = bytes.clone();
    let opcode = unknown.len() - 5;
    unknown[opcode] = 255;
    assert!(crates::decode(&unknown).is_err());
    let mut missing = bytes;
    missing[start - 4..start].copy_from_slice(&0_u32.to_le_bytes());
    missing.truncate(start);
    assert!(
        crates::decode(&missing).is_err(),
        "accepted missing operation coverage"
    );
    Ok(())
}
#[test]
fn operation_envelope_preserves_native_wrapper_and_rejects_legacy_dispatch() -> Result {
    use rustgm::support::linker_libraries::{Kind as LinkKind, Library, Plan};
    let interface = exported("provider")?;
    let plain = crates::encode(&interface)?;
    let mut links = Plan::default();
    links.push(Library::new("recipe_dependency", LinkKind::Dylib)?)?;
    let wrapped = crates::encode(&interface.with_native_links(links)?)?;
    assert_eq!(wrapped.get(12..14), Some([12, 1].as_slice()));
    assert!(wrapped.ends_with(&plain));
    let decoded = crates::decode(&wrapped)?;
    assert_eq!(crates::encode(&decoded)?, wrapped);
    assert_eq!(
        decoded
            .source_descriptor()
            .ok_or("wrapped recipe descriptor")?
            .compiler_operations()?
            .operations()
            .len(),
        1
    );
    for tag in [10, 11, 12, 14] {
        let mut forged = plain.clone();
        forged[12] = tag;
        assert!(crates::decode(&forged).is_err(), "accepted envelope {tag}");
    }
    for version in [9_u16, 10, 12] {
        let mut forged = plain.clone();
        forged[13..15].copy_from_slice(&version.to_le_bytes());
        assert!(
            crates::decode(&forged).is_err(),
            "accepted version {version}"
        );
    }
    for length in [0, 12, 13, 14, plain.len() - 1] {
        assert!(crates::decode(&plain[..length]).is_err());
    }
    let mut trailing = plain;
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    Ok(())
}
#[test]
fn historical_source_and_provided_packets_keep_exact_bytes() -> Result {
    for name in [
        "functions-v0",
        "contracts-v1",
        "contracts-marked-v1",
        "modules-v2",
        "provided-v0",
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/provided-contracts-v1/{name}.hex"))?;
        let bytes = text
            .lines()
            .filter(|line| !line.starts_with('#'))
            .flat_map(str::split_ascii_whitespace)
            .map(|value| u8::from_str_radix(value, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes, "{name}");
    }
    Ok(())
}
#[test]
fn publication_checks_real_body_and_recipe_shape_before_symbolic_exclusion() -> Result {
    for (name, cause) in [
        ("wrong-output", "actual async unit output"),
        ("invalid-body", "incompatible types"),
        ("unmarked", "async state-machine construction"),
        ("wrong-unsafe", "unsafe Rust borrow signature"),
    ] {
        let error = exported(name)
            .err()
            .ok_or_else(|| format!("published {name}"))?;
        assert!(
            error.to_string().contains(cause),
            "{name}: unrelated rejection: {error}"
        );
        println!("{name}: {error}");
    }
    let bytes = crates::encode(&exported("provider")?)?;
    client(crates::decode(&bytes)?, "consumer")?;
    for (name, cause) in [
        ("wrong-output-consumer", "incompatible types"),
        (
            "unsafe-consumer",
            "call to unsafe function requires an unsafe context",
        ),
    ] {
        let error = client(crates::decode(&bytes)?, name)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        let message = error.to_string();
        assert!(
            message.contains(cause),
            "{name}: unrelated rejection: {message}"
        );
        assert!(
            ![
                "not implemented",
                "unavailable",
                "unsupported",
                "unknown source",
                "future state"
            ]
            .iter()
            .any(|fence| message.contains(fence)),
            "unrelated rejection: {message}"
        );
        println!("{name}: {message}");
    }
    Ok(())
}
