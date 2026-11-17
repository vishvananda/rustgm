//! Checked source crate markers retain root feature and loader identity ownership.
use std::{
    path::Path,
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa34/tests/spec/panic-runtime-marker";

fn run(name: &str, host: &str, profile: &str, guests: &str) -> Result<Output> {
    let packet = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    if profile == "ordinary" {
        return Ok(Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-body-ir", "--cap-lints=allow", "--edition=2024"])
            .arg(packet.join(host))
            .output()?);
    }
    // The existing expanded view calls both public expansion and name-resolution
    // APIs. Each CLI process owns its guest selection; no body/lowering phase runs.
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/loader-markers")
        .join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args([
            "--emit-expanded-crate",
            "--cap-lints=allow",
            "--edition=2024",
        ])
        .args([
            "--crate-name",
            "core",
            "--crate-disambiguator",
            "panic_marker",
        ])
        .args(["--source-core", "core@panic_marker"]);
    std::fs::copy(packet.join(host), folder.join("core.rs"))?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    command.arg("--bringup-skip-list").arg(skip);
    if guests != "-" {
        for (index, entry) in guests.split('|').enumerate() {
            let (guest, file) = entry.split_once('=').ok_or("invalid panic marker guest")?;
            let path = folder.join(format!("guest-{index}.rs"));
            std::fs::copy(packet.join(file), &path)?;
            command
                .arg("--bringup-guest")
                .arg(format!("{guest}@2024={}", path.display()));
        }
    }
    Ok(command.arg(folder.join("core.rs")).output()?)
}

#[test]
fn panic_runtime_is_one_checked_source_root_marker_with_its_own_feature() -> Result {
    let packet = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    let cases = std::fs::read_to_string(packet.join("cases.tsv"))?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, host, expected, profile, guests, message] = columns.as_slice() else {
            return Err("malformed panic marker contract".into());
        };
        let output = run(name, host, profile, guests)?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(expected.parse()?),
            "{name}: {diagnostic}"
        );
        if *expected == "1" {
            assert!(output.stdout.is_empty(), "{name}");
            assert!(diagnostic.contains(message), "{name}: {diagnostic}");
        }
        count += 1;
    }
    assert_eq!(count, 36);
    Ok(())
}
