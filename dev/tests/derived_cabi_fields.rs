//! Source derive field transfer owns its selected ABI literal and byte pool.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa31/tests/spec/derive-cabi-fields";

#[test]
fn source_c_abi_fields_preserve_literal_ownership_and_closed_profiles() -> Result {
    let packet = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    let cases = std::fs::read_to_string(packet.join("cases.tsv"))?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, file, expected, profile, message] = columns.as_slice() else {
            return Err("invalid C ABI derive contract".into());
        };
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        let emitter = if *profile == "source" {
            "--emit-expanded-crate"
        } else {
            "--emit-body-ir"
        };
        command.args([emitter, "--cap-lints=allow", "--edition=2024"]);
        if *profile == "source" {
            command.args([
                "--crate-name",
                "core",
                "--crate-disambiguator",
                "derive_cabi_observer",
            ]);
            command.args(["--source-core", "core@derive_cabi_observer"]);
        }
        let output = command.arg(packet.join(file)).output()?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(expected.parse()?),
            "{name}: {diagnostic}"
        );
        if *expected == "1" {
            assert!(output.stdout.is_empty(), "{name}");
            if *message != "-" {
                assert!(diagnostic.contains(message), "{name}: {diagnostic}");
            }
        }
        count += 1;
    }
    assert_eq!(count, 12);
    Ok(())
}
