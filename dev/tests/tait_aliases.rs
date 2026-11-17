//! Named opaque inference belongs to selected source bodies, not ordinary callers.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa34/tests/spec/named-opaques";

#[test]
fn defining_aliases_keep_identity_captures_and_crate_authority() -> Result {
    let packet = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    let table = std::fs::read_to_string(packet.join("cases.tsv"))?;
    let mut failures = Vec::new();
    let mut count = 0;
    for row in table.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, host, expected, profile, guests, message] = columns.as_slice() else {
            return Err("invalid named opaque declaration".into());
        };
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command.args([
            "--emit-generic-semantics",
            "--cap-lints=allow",
            "--edition=2024",
        ]);
        if *profile != "ordinary" {
            command
                .args([
                    "--crate-name",
                    "core",
                    "--crate-disambiguator",
                    "named_aliases",
                ])
                .args(["--source-core", "core@named_aliases"]);
        }
        if *guests != "-" {
            let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../obj/named-aliases")
                .join(format!("{name}-{}", std::process::id()));
            std::fs::create_dir_all(&folder)?;
            let skip = folder.join("skip.tsv");
            std::fs::write(&skip, "")?;
            command.arg("--bringup-skip-list").arg(skip);
            for entry in guests.split('|') {
                let (identity, file) = entry.split_once('=').ok_or("invalid named alias guest")?;
                command
                    .arg("--bringup-guest")
                    .arg(format!("{identity}={}", packet.join(file).display()));
            }
        }
        let output = command.arg(packet.join(host)).output()?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        let status = Some(expected.parse::<i32>()?);
        let infrastructure = [
            "budget exceeded",
            "exceed their work",
            "panicked at",
            "internal compiler error",
            "requires `future_trait`",
            "--bringup-guest requires",
            "invalid inference identity",
            "unsupported active source-core feature or undefined library feature request",
        ];
        if output.status.code() != status
            || infrastructure.iter().any(|word| diagnostic.contains(word))
        {
            failures.push(format!(
                "{name}: {:?}, expected{status:?}: {diagnostic}",
                output.status.code()
            ));
        }
        if *expected == "1"
            && *profile != "ordinary"
            && (diagnostic.is_empty() || diagnostic.contains("not implemented"))
        {
            failures.push(format!("{name}: unrelated unsupported fence: {diagnostic}"));
        }
        if *expected == "1"
            && *profile != "ordinary"
            && !message
                .split('|')
                .any(|fragment| diagnostic.contains(fragment))
        {
            failures.push(format!("{name}: expected cause{message}: {diagnostic}"));
        }
        count += 1;
    }
    assert_eq!(count, 27);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
