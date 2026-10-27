//! First producer encodings bind the old profile bytes before a new profile exists.
use super::*;
use crate::lower::instances::SourcePublication;

#[test]
fn existing_profiles_capture_first_bytes_before_provided_contracts() -> Result {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/provided-contracts-baseline");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(
        folder.join(".keep-artifacts"),
        "Immutable first producer bytes before ProvidedContracts v1\n",
    )?;
    let plain = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let marked = crate::test_source::fixture(
        "pa31/tests/spec/source-publication-attributes/must-use-reason.rs",
    )?;
    for (name, source, profile, expected) in [
        (
            "functions-v0",
            &plain,
            SourcePublication::LegacyFunctions,
            s::Profile::Source2024Functions,
        ),
        (
            "contracts-v1",
            &plain,
            SourcePublication::FunctionContracts,
            s::Profile::Source2024FunctionContracts,
        ),
        (
            "contracts-marked-v1",
            &marked,
            SourcePublication::FunctionContracts,
            s::Profile::Source2024FunctionContracts,
        ),
        (
            "modules-v2",
            &plain,
            SourcePublication::ModuleContracts,
            s::Profile::Source2024ModuleContracts,
        ),
    ] {
        let interface =
            crate::module_packets::exported_profile(source, profile, "provided_compatibility")?;
        assert_eq!(
            interface
                .source_descriptor()
                .ok_or("actual descriptor")?
                .profile(),
            expected
        );
        capture(&folder, name, source, &interface)?;
    }
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    let (owner, rows) = actual(true)?;
    let interface = interface(owner, rows)?;
    assert_eq!(
        interface
            .source_descriptor()
            .ok_or("Provided v0 descriptor")?
            .profile(),
        s::Profile::Source2024Provided
    );
    capture(&folder, "provided-v0", &source, &interface)?;
    Ok(())
}

fn capture(
    folder: &std::path::Path,
    name: &str,
    source: &str,
    interface: &crates::Interface,
) -> Result {
    // Encoding the actual producer Interface precedes any decode; no roundtrip
    // can substitute for this original byte contract.
    let bytes = codec::encode(interface)?;
    std::fs::write(folder.join(format!("{name}.rgcrate")), &bytes)?;
    std::fs::write(folder.join(format!("{name}.rs")), source)?;
    let descriptor = interface
        .source_descriptor()
        .ok_or("actual source descriptor")?;
    std::fs::write(
        folder.join(format!("{name}.binding.txt")),
        format!(
            "producer={}\nname={}\ndisambiguator={}\nmode=SourceCore\nconfig=Config::default\nprofile={:?}\nedition={:?}\nencoded_bytes={}\n",
            if name == "provided-v0" {
                "inspect_source_transport_with_numbering+from_source_provided"
            } else {
                "build_source_library_profile"
            },
            interface.identity().name(),
            interface.identity().disambiguator(),
            descriptor.profile(),
            descriptor.edition(),
            bytes.len()
        ),
    )?;
    assert_eq!(codec::decode(&bytes)?, *interface);
    Ok(())
}
