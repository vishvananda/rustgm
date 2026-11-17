//! The pinned TLS target fact and its query gate are separate from TLS storage.
use rustgm::expand::config::Config;
use std::{path::Path, process::Output};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa34/tests/spec/target-tls-query";

fn run(name: &str, file: &str, setting: Option<&str>) -> Result<Output> {
    let (folder, mut command) = crate::guest_crates::prepare(name, &[])?;
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(PACKET)
            .join(file),
        folder.join("core.rs"),
    )?;
    if let Some(setting) = setting {
        command.args(["--cfg", setting]);
    }
    Ok(command.arg(folder.join("core.rs")).output()?)
}

#[test]
fn tls_queries_keep_requested_and_literal_origin_permissions_separate_from_storage() -> Result {
    let cases = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(PACKET)
            .join("cases.tsv"),
    )?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, file, expected, message] = columns.as_slice() else {
            return Err("malformed TLS query contract".into());
        };
        let output = run(name, file, None)?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(expected.parse()?),
            "{name}: {diagnostic}"
        );
        if *expected == "1" {
            assert!(diagnostic.contains(message), "{name}: {diagnostic}");
            assert!(output.stdout.is_empty(), "{name}");
        }
        count += 1;
    }
    assert_eq!(count, 15);
    Ok(())
}

#[test]
fn target_tls_fact_is_fixed_and_driver_overrides_cannot_replace_it() -> Result {
    assert!(Config::default().contains("target_thread_local", None));
    for setting in ["target_thread_local", "target_thread_local=\"false\""] {
        let error = Config::default()
            .add(setting)
            .err()
            .ok_or("target fact override accepted")?;
        assert!(error.message.contains("compiler-controlled cfg option"));
        let output = run("target-tls-override", "requested-macro.rs", Some(setting))?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("compiler-controlled cfg option"));
    }
    Ok(())
}
