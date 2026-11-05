//! Actual helper declarations preserve source proof and compiler-owned execution.
use crate::resolve as test_source;
use rustgm::{body::templates::Prepared, pipeline};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_helper_calls_keep_source_typing_and_never_publish_placeholder_cfg() -> Result {
    let cases = test_source::fixture("pa43/tests/spec/async-drop-helper/cases.tsv")?;
    let mut count = 0;
    for line in cases.lines() {
        let [path, status, boundary]: [_; 3] = line
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "three helper contract columns")?;
        let source = test_source::fixture(path)?;
        let analysis = (|| {
            let report = test_source::source_provider(&source, "async_helper")?;
            let facts = pipeline::analyze_source(&report)?.facts;
            Ok::<_, Box<dyn std::error::Error>>((report, facts))
        })();
        if status == "1" {
            let error = analysis.err().ok_or("invalid helper consumer accepted")?;
            let message = error.to_string();
            assert!(
                !message.contains("not implemented")
                    && !message.contains("generated future state")
                    && !message.contains("invalid or exhausted"),
                "{path}: {message}"
            );
        } else {
            assert_eq!(status, "0");
            let (report, facts) = analysis.map_err(|error| format!("{path}: {error}"))?;
            let function = facts
                .functions
                .iter()
                .find(|function| {
                    let actual = &report.resolution.definitions[function.definition.0 as usize];
                    report.package.names.get(actual.name) == "cleanup"
                })
                .ok_or("missing actual helper function")?;
            let mut prepared = Prepared::default();
            let before = prepared.work();
            let result = prepared.function(&report, &facts, function);
            if boundary == "ordinary" {
                result.map_err(|error| format!("unmarked ordinary function: {error}"))?;
            } else {
                let error = result
                    .err()
                    .ok_or("helper published its written placeholder CFG")?;
                assert!(error.message.contains(boundary), "{path}: {error}");
                assert_eq!(prepared.work(), before, "{path} published eager work");
            }
        }
        count += 1;
    }
    assert_eq!(count, 9);
    Ok(())
}

#[test]
fn independent_helper_companions_keep_the_actual_authored_bodies() -> Result {
    let cases = test_source::fixture("pa43/tests/spec/async-drop-helper/cases.tsv")?;
    let mut count = 0;
    for line in cases.lines() {
        let path = line.split('\t').next().ok_or("helper fixture path")?;
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("helper source name")?;
        if matches!(name, "ordinary-unused.rs" | "unmarked-ordinary.rs") {
            continue;
        }
        let source = test_source::fixture(path)?;
        let (_, authored) = source.split_once("#[lang=\"future_trait\"] pub trait Future { #[lang=\"future_output\"] type Output; }\n").ok_or("source Future foundation")?;
        let authored = authored
            .strip_prefix("#[lang=\"async_drop_in_place\"]\n")
            .ok_or("actual source helper role")?;
        let companion =
            test_source::fixture(&format!("pa43/tests/spec/async-drop-helper/oracle/{name}"))?;
        let (_, oracle) = companion
            .split_once("use core::future::Future;\n")
            .ok_or("independent core foundation")?;
        assert_eq!(authored.as_bytes(), oracle.as_bytes(), "{name}");
        count += 1;
    }
    assert_eq!(count, 7);
    Ok(())
}
