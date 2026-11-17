//! Source host/guest preludes consume one invoking crate's actual selection.
use std::{path::PathBuf, process::Output};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa34/tests/spec/crate-preludes";

fn run(name: &str, host: &str, guests: &str) -> Result<Output> {
    let inputs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(PACKET);
    let (folder, mut command) = crate::guest_crates::prepare(&format!("authored-{name}"), &[])?;
    std::fs::copy(inputs.join(host), folder.join("core.rs"))?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    command.arg("--bringup-skip-list").arg(skip);
    for (index, guest) in guests.split('|').enumerate() {
        let (identity, input) = guest.split_once('=').ok_or("invalid guest declaration")?;
        let root = folder.join(format!("guest-{index}.rs"));
        std::fs::copy(inputs.join(input), &root)?;
        command
            .arg("--bringup-guest")
            .arg(format!("{identity}={}", root.display()));
    }
    Ok(command.arg(folder.join("core.rs")).output()?)
}

#[test]
fn each_source_crate_owns_its_prelude_and_last_selector() -> Result {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(PACKET);
    let cases = std::fs::read_to_string(root.join("cases.tsv"))?;
    let mut count = 0;
    for line in cases.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let [name, host, expected, guests, message] = fields.as_slice() else {
            return Err("invalid authored prelude case".into());
        };
        let output = run(name, host, guests)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(expected.parse()?),
            "{name}: {stderr}"
        );
        if *expected == "1" {
            assert!(!message.is_empty(), "missing opposing diagnostic: {name}");
            assert!(stderr.contains(*message), "{name}: {stderr}");
            assert!(!stderr.contains("budget exceeded"), "{name}: {stderr}");
        }
        count += 1;
    }
    assert_eq!(count, 20);
    Ok(())
}
