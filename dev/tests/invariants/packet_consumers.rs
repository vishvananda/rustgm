//! The public artifact path reconstructs and proves a genuinely source-free client.
use crate::{
    pipeline,
    source_packets::{client, client_analysis, exported},
    support::crates,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-functions-transport/{name}.rs"
    ))?)
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
fn ordinary_source_profiles_capture_plain_contract_bytes_before_adapter() -> Result {
    use crate::lower::instances::{SourcePublication, build_source_library_profile};
    use crates::generics::source as s;
    let mut source = fixture("minimal-provider")?;
    source.push_str("\npub fn plain(x:u8)->u8{x}\npub fn unused<U>(x:u8)->u8{x}\n");
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/empty-contract-baseline");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(
        folder.join(".keep-artifacts"),
        b"Actual raw pre-adapter v0/v1 bytes and binding\n",
    )?;
    std::fs::write(folder.join("provider.rs"), source.as_bytes())?;
    for (version, profile, expected) in [
        (
            "v0",
            SourcePublication::LegacyFunctions,
            crates::SourceProfile::Source2024Functions,
        ),
        (
            "v1",
            SourcePublication::FunctionContracts,
            crates::SourceProfile::Source2024FunctionContracts,
        ),
    ] {
        let report = crate::test_source::source_provider(&source, "plain_contract_pair")?;
        let identity = report
            .package
            .source_core()
            .ok_or("actual provider")?
            .provider
            .clone();
        assert_eq!(identity.name(), "minicore");
        assert_eq!(identity.disambiguator(), "plain_contract_pair");
        let mut analysis = pipeline::analyze_source(&report)?;
        let (_, interface) = build_source_library_profile(
            &report,
            &mut analysis.facts,
            identity.clone(),
            analysis.prepared,
            profile,
        )?;
        let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
        assert_eq!(descriptor.profile(), expected);
        let rows = descriptor.rows();
        assert_eq!(
            rows.header_attributes().is_some(),
            profile == SourcePublication::FunctionContracts
        );
        let plain = rows
            .functions
            .iter()
            .find(|header| {
                rows.declarations[header.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some("plain")
            })
            .ok_or("actual nongeneric plain header")?;
        let s::FunctionBody::Original(body) = plain.body else {
            return Err("actual plain original".into());
        };
        let input = &rows.body_inputs[body.0 as usize];
        assert_eq!(input.environment, s::EnvironmentOrigin::Contract);
        assert!(input.parameters.is_empty());
        assert_eq!(plain.regions.total, 0);
        let bytes = crates::encode(&interface)?;
        // Retain the actual first encoding. Roundtrip output is not this oracle.
        std::fs::write(folder.join(format!("{version}.rgcrate")), &bytes)?;
        std::fs::write(
            folder.join(format!("{version}.rows.txt")),
            format!("{:#?}\nattributes={:#?}\n", *rows, rows.header_attributes()),
        )?;
        std::fs::write(
            folder.join(format!("{version}.binding.txt")),
            format!(
                "producer=build_source_library_profile\nmode=SourceCore\nconfig=Config::default\ninput=source-provider.rs\nname={}\ndisambiguator={}\npublication={profile:?}\nprofile={expected:?}\nedition={:?}\nplain_origin={:?}\nplain_builtin_origin={:?}\nencoded_bytes={}\n",
                identity.name(),
                identity.disambiguator(),
                descriptor.edition(),
                input.environment,
                input.builtins.as_ref().map(|builtins| builtins.environment),
                bytes.len()
            ),
        )?;
        let program = client(
            crates::decode(&bytes)?,
            &fixture("runtime-client")?.replace("base::identity", "base::plain"),
        )?;
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

#[test]
fn unrestricted_import_policy_keeps_real_functions_roundtrip_and_denies_restricted_export() -> Result
{
    let source = fixture("minimal-provider")?;
    let bytes = crates::encode(&exported(&source)?)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    assert_eq!(
        decoded
            .source_descriptor()
            .ok_or("actual descriptor")?
            .profile(),
        crates::SourceProfile::Source2024Functions
    );
    let consumer = "#![feature(no_core)] #![no_core] extern crate base; struct Local; impl base::Clone for Local {fn clone(&self)->Self{Local}} impl base::Copy for Local {}";
    client_analysis(decoded, consumer)?;
    for changed in [
        source
            .replace("no_core,lang_items", "no_core,impl_restriction,lang_items")
            .replace("pub trait Copy:", "pub impl(crate) trait Copy:"),
        format!(
            "{}\nimpl(self) trait PrivateUnused {{}}",
            source.replace("no_core,lang_items", "no_core,impl_restriction,lang_items")
        ),
    ] {
        let error = exported(&changed)
            .err()
            .ok_or("restricted policy was erased during export")?;
        assert!(
            error
                .to_string()
                .contains("restricted trait implementation policy transport"),
            "{error}"
        );
    }
    let inactive = format!(
        "{}\n#[cfg(any())] impl(self) trait PrivateUnused {{}}",
        source.replace("no_core,lang_items", "no_core,impl_restriction,lang_items")
    );
    exported(&inactive)?;
    for index in [12, 13, 14, 15] {
        let mut changed = bytes.clone();
        changed[index] = u8::MAX;
        assert!(crates::decode(&changed).is_err());
    }
    Ok(())
}
