//! Separate compilation retains calling convention, safety and borrowed result identity.
use super::{artifacts, crates};

const ACCEPTED: [&str; 6] = [
    "pass-direct",
    "pass-pointer",
    "pass-unsafe",
    "pass-borrowed",
    "pass-real",
    "pass-record-reference",
];
const REJECTED: [(&str, &str); 6] = [
    ("reject-c-as-rust", "E0308"),
    ("reject-rust-as-c", "E0308"),
    ("reject-unsafe-call", "E0133"),
    ("reject-unsafe-pointer", "E0308"),
    ("reject-dangling", "E0515"),
    ("reject-exclusive-result", "E0499"),
];

#[test]
fn foreign_signature_metadata_preserves_abi_safety_and_borrows()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_foreign("foreign-metadata-source")?;
    build.qualify()?;
    for (path, shared) in [(&build.first, false), (&build.second, true)] {
        let bytes = std::fs::read(path)?;
        assert_eq!(&bytes[12..14], &[4, u8::from(shared)]);
        let interface = crates::decode(&bytes)?;
        assert_eq!(crates::encode(&interface)?, bytes);
        for function in interface.functions() {
            let expected = if function.definition().path() == ["ordinary"] {
                crates::Abi::Rust
            } else {
                crates::Abi::C
            };
            assert_eq!(function.signature().abi(), expected);
            if let Some(nominals) = interface.nominals() {
                assert!(
                    function
                        .signature()
                        .types()
                        .shares_storage(nominals.types())
                );
            }
        }
    }
    for name in ACCEPTED {
        build.qualify_client(name, None)?;
        let direct = artifacts::checked(name, &build.folder, &mut build.client(name, false))?;
        let reordered = artifacts::checked(
            &format!("{name}.reordered"),
            &build.folder,
            &mut build.client(name, true),
        )?;
        assert_eq!(direct.stdout, reordered.stdout);
    }
    for (name, code) in REJECTED {
        build.qualify_client(name, Some(code))?;
        let output = build.client(name, false).output()?;
        std::fs::write(build.folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(output.status.code(), Some(1), "{name}: {output:?}");
        let text = String::from_utf8_lossy(&output.stderr);
        assert!(
            !text.contains("profile") && !text.contains("missing"),
            "{name}: {text}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires configured pinned CPPGM object backend and Rust toolchain"]
fn foreign_signature_libraries_execute_through_the_shared_backend()
-> Result<(), Box<dyn std::error::Error>> {
    let build = artifacts::build_foreign("foreign-metadata-native")?;
    build.qualify()?;
    for name in ACCEPTED {
        build.execute(name)?;
    }
    Ok(())
}
