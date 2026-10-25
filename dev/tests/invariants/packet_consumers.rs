//! The public artifact path reconstructs and proves a genuinely source-free client.
use crate::{
    lower, pipeline, resolve,
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency},
    },
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-functions-transport/{name}.rs"
    ))?)
}
fn exported(source: &str) -> Result<crates::Interface> {
    let report = crate::test_source::source_provider(source, "source_transaction")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider policy")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let (_, interface) =
        lower::instances::build_library_prepared(&report, &mut facts, identity, prepared)?;
    Ok(interface)
}
fn client_analysis(
    interface: crates::Interface,
    source: &str,
) -> Result<(
    resolve::expansion::Report,
    pipeline::Analysis,
    CrateIdentity,
)> {
    let provider = interface.identity().clone();
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), provider.clone())?],
    )?);
    let current = CrateIdentity::new("client", "source_transaction")?;
    let context = resolve::bindings::Context {
        crate_name: Some("client".into()),
        current: Some(current.clone()),
        dependencies: Some(dependencies),
        mode: Mode::SourceClient { provider },
    };
    let expanded = resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "client.rs".into(),
            path: "client.rs".into(),
            source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    assert_eq!(
        report.package.units.len(),
        1,
        "provider syntax leaked into fresh client"
    );
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual source inventory")?;
    for definition in imported.generic_definitions() {
        assert!(
            report.resolution.definitions[definition?.0 as usize]
                .node
                .is_none()
        );
    }
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis, current))
}
fn client(interface: crates::Interface, source: &str) -> Result<crate::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
        current,
    ) = client_analysis(interface, source)?;
    let (program, _) =
        lower::instances::build_library_prepared(&report, &mut facts, current, prepared)?;
    Ok(program)
}
#[test]
fn prepared_imported_contracts_preserve_raw_requests_and_reject_foreign_owner_arena() -> Result {
    let bytes = crates::encode(&exported(&fixture("minimal-provider")?)?)?;
    let (_, analysis, _) = client_analysis(crates::decode(&bytes)?, &fixture("runtime-client")?)?;
    let mut facts = analysis.facts;
    let originals = analysis
        .prepared
        .imported
        .ok_or("actual imported originals")?;
    let raw = originals.iter().next().ok_or("actual original")?;
    let raw_count = raw
        .contract
        .builtins
        .as_ref()
        .ok_or("mandatory raw requests")?
        .predicates
        .len();
    assert!(raw_count > 0);
    let prepared = crate::body::copy_requirements::ImportedInputs::build(
        &mut facts,
        std::rc::Rc::clone(&originals),
        pipeline::OriginalInput::prepare_imported,
    )?;
    let inputs = prepared.inputs(&facts.types, Some(&originals))?;
    assert_eq!(inputs.len(), originals.len());
    assert!(
        inputs[0]
            .builtins
            .as_ref()
            .ok_or("prepared input")?
            .predicates
            .iter()
            .all(|(_, goal)| !matches!(
                goal.bound,
                crate::types::parameters::predicates::Bound::Trait(_)
            ))
    );
    assert_eq!(
        raw.contract
            .builtins
            .as_ref()
            .ok_or("raw")?
            .predicates
            .len(),
        raw_count
    );
    let (_, other, _) = client_analysis(crates::decode(&bytes)?, &fixture("runtime-client")?)?;
    assert!(
        prepared
            .inputs(&other.facts.types, Some(&originals))
            .is_err()
    );
    assert!(
        prepared
            .inputs(&facts.types, other.prepared.imported.as_ref())
            .is_err()
    );
    let saved = facts.constant_work.template_bytes;
    facts.constant_work.template_bytes = 64 * 1024 * 1024;
    let mut produced = 0;
    let rejected = crate::body::copy_requirements::ImportedInputs::build(
        &mut facts,
        std::rc::Rc::clone(&originals),
        |_, _, raw| {
            produced += 1;
            Ok(raw.clone())
        },
    );
    facts.constant_work.template_bytes = saved;
    assert!(rejected.is_err());
    assert_eq!(produced, 0, "budget rejected after cloning contracts");
    assert_eq!(
        raw.contract
            .builtins
            .as_ref()
            .ok_or("raw")?
            .predicates
            .len(),
        raw_count
    );
    Ok(())
}
#[test]
fn prepared_imported_contracts_scale_with_originals_and_borrow_repeatedly() -> Result {
    let mut previous = None;
    for count in [1, 8, 32] {
        let mut source = fixture("minimal-provider")?;
        for index in 1..count {
            use std::fmt::Write;
            write!(&mut source, "pub fn extra{index}<T>(x:T)->T{{x}}")?;
        }
        let (_, analysis, _) = client_analysis(exported(&source)?, &fixture("runtime-client")?)?;
        let mut facts = analysis.facts;
        let originals = analysis.prepared.imported.ok_or("actual originals")?;
        assert_eq!(originals.len(), count);
        let before = facts.body_work.canonical_steps;
        let prepared = crate::body::copy_requirements::ImportedInputs::build(
            &mut facts,
            std::rc::Rc::clone(&originals),
            pipeline::OriginalInput::prepare_imported,
        )?;
        let work = facts.body_work.canonical_steps - before;
        let before = facts.body_work.canonical_steps;
        for _ in 0..128 {
            assert_eq!(
                prepared.inputs(&facts.types, Some(&originals))?.len(),
                count
            );
        }
        assert_eq!(
            facts.body_work.canonical_steps, before,
            "borrowed lookup reproved originals"
        );
        let bytes = prepared.retained_bytes();
        if let Some((prior_count, prior_work, prior_bytes)) = previous {
            assert!(work <= prior_work * count / prior_count + 128);
            assert!(bytes <= prior_bytes * count / prior_count + 128);
        }
        println!(
            "prepared_imported_originals={count} prepare_work={work} retained_contract_bytes={bytes} repeated_borrows=128 extra_prepare_work=0"
        );
        previous = Some((count, work, bytes));
    }
    Ok(())
}
#[test]
fn actual_source_functions_cross_public_codec_and_fresh_client_proof() -> Result {
    let source = fixture("minimal-provider")?;
    let original_client = fixture("runtime-client")?;
    for (source, name) in [
        (source.clone(), "identity"),
        (source.replace("pub fn identity<T>(x: T) -> T { x }",
            "pub fn second<U>(other: U) -> U { other }\npub fn first<T>(value: T) -> T { value }"), "first"),
    ] {
        let interface = exported(&source)?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(bytes.get(12), Some(&super::TAG));
        let decoded = crates::decode(&bytes)?;
        assert_eq!(crates::encode(&decoded)?, bytes);
        assert_eq!(decoded.source_descriptor().ok_or("source descriptor")?.profile(), crates::SourceProfile::Source2024Functions);
        let program = client(decoded, &original_client.replace("base::identity", &format!("base::{name}")))?;
        assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    }
    Ok(())
}
#[test]
fn public_source_envelope_rejects_version_edition_truncation_and_extra_bytes() -> Result {
    let bytes = crates::encode(&exported(&fixture("minimal-provider")?)?)?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "truncation={end}");
    }
    for index in [12, 13, 14, 15] {
        let mut changed = bytes.clone();
        changed[index] = u8::MAX;
        assert!(crates::decode(&changed).is_err(), "envelope byte={index}");
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(crates::decode(&extra).is_err());
    Ok(())
}
#[test]
fn public_source_envelope_rejects_foreign_root_identity_and_column_domains() -> Result {
    let bytes = crates::encode(&exported(&fixture("minimal-provider")?)?)?;
    let mut foreign = bytes.clone();
    // Prefix16, identity name length4, then its exact8-byte minicore spelling.
    assert_eq!(&foreign[20..28], b"minicore");
    foreign[20..28].copy_from_slice(b"foreignx");
    assert!(crates::decode(&foreign).is_err());
    let mut input = super::Reader::new(&bytes);
    input.bytes(16)?;
    input.identity()?;
    let dependencies = input.u32()?;
    for _ in 0..dependencies {
        input.identity()?;
    }
    let columns = bytes.len() - input.remaining();
    for offset in [columns, columns + 4, columns + 8, columns + 12] {
        let mut changed = bytes.clone();
        changed[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(
            crates::decode(&changed).is_err(),
            "source column offset={offset}"
        );
    }
    Ok(())
}
#[test]
fn source_export_rejects_invalid_unused_body_and_marked_provenance() -> Result {
    assert!(exported(&fixture("invalid-move-provider")?).is_err());
    let source = fixture("minimal-provider")?;
    let changed = source.replace(
        "pub fn identity<T>(x: T) -> T { x }",
        "macro_rules! make {() => {pub fn identity<T>(x:T)->T{x}}} make!();",
    );
    let error = exported(&changed)
        .err()
        .ok_or("marked provider exported")?
        .to_string();
    assert!(
        error.contains("provenance") || error.contains("source declaration kind"),
        "{error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires configured pinned object backend and host C compiler; original sources already independently qualified"]
fn source_function_artifact_executes_fresh_native_client_seven() -> Result {
    use std::{path::Path, process::Command};
    let interface = exported(&fixture("minimal-provider")?)?;
    let bytes = crates::encode(&interface)?;
    let program = client(crates::decode(&bytes)?, &fixture("runtime-client")?)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let output = root
        .join("obj/parallel/pa29-source-client-staging/lib007/packet-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&output)?;
    std::fs::write(output.join("provider.rgm"), bytes)?;
    let lowir = output.join("client.lowir");
    crate::lowir::writer::write(&program, &mut std::fs::File::create(&lowir)?)?;
    for (name, mut command) in [
        ("backend", {
            let mut command = Command::new("python3");
            command
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&lowir)
                .arg("-o")
                .arg(output.join("client.o"));
            command
        }),
        ("host-link", {
            let mut command = Command::new("cc");
            command
                .arg("-std=c11")
                .arg("-Wall")
                .arg("-Wextra")
                .arg("-Werror")
                .arg(root.join("pa31/tests/spec/source-functions-transport/native-driver.c"))
                .arg(output.join("client.o"))
                .arg("-o")
                .arg(output.join("client.native"));
            command
        }),
        ("execute", Command::new(output.join("client.native"))),
    ] {
        let result = command.output()?;
        std::fs::write(output.join(format!("{name}.stdout")), &result.stdout)?;
        std::fs::write(output.join(format!("{name}.stderr")), &result.stderr)?;
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if name == "execute" {
            assert_eq!(result.stdout, b"7\n");
        }
    }
    Ok(())
}
