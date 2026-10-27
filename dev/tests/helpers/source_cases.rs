//! Independently qualified outcomes pass through the complete command-line pipeline.
use crate::qualify as rustc;
pub use rustc::{qualify, qualify_at};
use rustgm::support::diagnostics::WarningCode;
use std::{collections::HashMap, path::Path, process::Command};

pub struct WarningCase<'a> {
    pub name: &'a str,
    pub stderr: &'a str,
    pub codes: &'a [(WarningCode, usize)],
}

pub fn validate_warnings(
    cases: &[(&str, bool, &str)],
    warnings: &[WarningCase<'_>],
) -> Result<(), Box<dyn std::error::Error>> {
    let cases_by_name: HashMap<_, _> = cases
        .iter()
        .map(|&(name, accepted, _)| (name, accepted))
        .collect();
    if cases_by_name.len() != cases.len() {
        return Err("duplicate source case name".into());
    }
    let mut seen = std::collections::HashSet::new();
    for warning in warnings {
        if !seen.insert(warning.name) || cases_by_name.get(warning.name) != Some(&true) {
            return Err("warning expectation must name one unique accepted case".into());
        }
        let declared: HashMap<_, _> = warning
            .codes
            .iter()
            .map(|&(code, count)| (code.name(), count))
            .collect();
        if declared.is_empty()
            || declared.len() != warning.codes.len()
            || declared.values().any(|&count| count == 0)
        {
            return Err("warning codes require unique positive counts".into());
        }
        let mut actual = HashMap::new();
        if !warning.stderr.ends_with('\n') {
            return Err("warning reference requires a final newline".into());
        }
        for line in warning.stderr.lines() {
            let (code, remainder) = line
                .strip_prefix("warning[")
                .and_then(|line| line.split_once("]: file "))
                .ok_or("invalid warning reference header")?;
            let (file, remainder) = remainder.split_once(": ").ok_or("missing warning file")?;
            let (offset, message) = remainder.split_once(": ").ok_or("missing warning offset")?;
            file.parse::<u32>()?;
            offset.parse::<u32>()?;
            if message.is_empty() {
                return Err("empty warning message".into());
            }
            *actual.entry(code).or_insert(0) += 1;
        }
        if actual != declared {
            return Err("warning reference disagrees with declared codes/counts".into());
        }
    }
    Ok(())
}

pub fn verify_stderr(
    actual: &[u8],
    warning: Option<&WarningCase<'_>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = warning.map_or("", |warning| warning.stderr);
    if actual != expected.as_bytes() {
        return Err(format!(
            "source stderr differs from its exact declaration: {}",
            String::from_utf8_lossy(actual)
        )
        .into());
    }
    Ok(())
}

pub fn check(suite: &str, cases: &[(&str, bool, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    check_with_warnings(suite, cases, &[])
}

pub fn check_with_warnings(
    suite: &str,
    cases: &[(&str, bool, &str)],
    warnings: &[WarningCase<'_>],
) -> Result<(), Box<dyn std::error::Error>> {
    validate_warnings(cases, warnings)?;
    let warnings: HashMap<_, _> = warnings
        .iter()
        .map(|warning| (warning.name, warning))
        .collect();
    let base = if suite.starts_with("parallel/") {
        "../obj"
    } else {
        "../obj/language-qualification"
    };
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("{base}/{suite}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, expected, source) in cases {
        if suite.starts_with("parallel/") {
            qualify_at(&folder.join("rustc"), name, source, expected)?;
        } else {
            qualify(&format!("{suite}-{name}"), source, expected)?;
        }
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, format!("{source}fn main(){{}}"))?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&input)
            .output()?;
        std::fs::write(input.with_extension("stderr"), &result.stderr)?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!expected)),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if expected {
            verify_stderr(&result.stderr, warnings.get(name).copied())?;
        } else {
            assert!(!result.stderr.is_empty(), "{name}");
            assert!(
                String::from_utf8_lossy(&result.stderr)
                    .lines()
                    .any(|line| line.starts_with("ERROR: ")),
                "{name}: rejection lost its primary error"
            );
        }
    }
    Ok(())
}
