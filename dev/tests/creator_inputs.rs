//! Creator aliases, public call binders and inferred input ports keep distinct scopes.
use crate::provider_traits::lower;
use std::{fmt::Write as _, path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "pa31/tests/spec/inherited-input-regions";
const ORDINARY: [&str; 2] = [
    "pa20/tests/spec/reference-tail-inference",
    "pa24/tests/spec/inherited-input-regions",
];
struct Case {
    path: String,
    accepted: bool,
    reason: String,
}
fn cases(packet: &str) -> Result<Vec<Case>> {
    crate::resolve::fixture(&format!("{packet}/cases.tsv"))?
        .lines()
        .map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            let [path, outcome, reason] = columns.as_slice() else {
                return Err(format!("invalid creator input row {line}").into());
            };
            let accepted = match *outcome {
                "0" => true,
                "1" => false,
                _ => return Err("invalid creator outcome".into()),
            };
            Ok(Case {
                path: (*path).into(),
                accepted,
                reason: (*reason).into(),
            })
        })
        .collect()
}
fn root() -> Result<&'static Path> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?)
}
fn output(case: &Case, source: bool) -> Result<std::process::Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command.args([
        if source {
            "--emit-types"
        } else {
            "--emit-lowir"
        },
        "--edition=2024",
        "--cap-lints=allow",
    ]);
    if source {
        command.args([
            "--source-core",
            "minicore@creator_inputs",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "creator_inputs",
        ]);
    }
    let output = command.arg(root()?.join(&case.path)).output()?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(format!("{}: infrastructure failure {output:?}", case.path).into());
    }
    Ok(output)
}

#[test]
fn ordinary_inferred_inputs_keep_their_reference_depth_and_lifetime_proof() -> Result {
    let mut count = 0;
    for packet in ORDINARY {
        for case in cases(packet)? {
            let output = output(&case, false)?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(
                output.status.success(),
                case.accepted,
                "{}: {stderr}",
                case.path
            );
            if case.accepted {
                assert!(
                    !output.stdout.is_empty() && output.stderr.is_empty(),
                    "{}",
                    case.path
                );
            } else {
                assert!(
                    output.stdout.is_empty() && stderr.contains(&case.reason),
                    "{}: {stderr}",
                    case.path
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 38);
    Ok(())
}

#[test]
fn source_inferred_inputs_prove_every_original_body() -> Result {
    let mut failures = Vec::new();
    let rows = cases(SOURCE)?;
    for case in &rows {
        let typed = output(case, true)?;
        let error = if typed.status.success() {
            lower(&crate::resolve::fixture(&case.path)?)
                .err()
                .map(|error| error.to_string())
        } else {
            Some(String::from_utf8(typed.stderr)?)
        };
        let matches = match &error {
            None => case.accepted,
            Some(error) => !case.accepted && error.contains(&case.reason),
        };
        if !matches {
            failures.push(format!("{}: {error:?}", case.path));
        }
    }
    assert_eq!(rows.len(), 37);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

const NATIVES: [(&str, [&str; 3], &str); 2] = [
    (
        "core-pattern-native",
        ["search_needle", "search_haystack", "contains"],
        "13 29 0 13 13 1\n",
    ),
    (
        "generic-native",
        ["relay", "ref_relay", "mixed_relay"],
        "13 13 13 29 29 29\n",
    ),
];

#[test]
fn source_inferred_inputs_prepare_every_native_export() -> Result {
    for (name, exports, _) in NATIVES {
        let source = crate::resolve::fixture(&format!("{SOURCE}/{name}.rs"))?;
        let program = lower(&source)?;
        for export in exports {
            assert!(
                program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == export && symbol.body.is_some()),
                "{export}"
            );
        }
    }
    Ok(())
}

#[test]
fn inferred_port_specialization_work_tracks_creators() -> Result {
    let mut previous: Option<[usize; 3]> = None;
    for width in [8, 32, 128] {
        let mut source = String::new();
        let mut calls = String::from("fn main(){let n=7u8;let p=&n;");
        for index in 0..width {
            writeln!(
                source,
                "fn g{index}<'b,'c,T>(x:&'c &'b T)->&'b T{{(|&s|s)(x)}}"
            )?;
            write!(calls, "let _=g{index}(&p);")?;
        }
        source.push_str(&calls);
        source.push('}');
        let mut report = crate::resolve::resolve(&source, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("inferred ports width={width} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= 4 * old + 2048, "{width}: {before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
fn source_marker_witness_work_tracks_distinct_input_lifetimes() -> Result {
    let template = crate::resolve::fixture(&format!("{SOURCE}/generic-projected.rs"))?;
    let (prefix, _) = template
        .split_once("trait Element{")
        .ok_or("source prefix")?;
    let mut previous: Option<[usize; 3]> = None;
    for width in [8, 32, 128] {
        let mut parameters = String::new();
        let mut fields = String::new();
        for index in 0..width {
            write!(parameters, "'r{index},")?;
            write!(fields, "&'r{index} T,")?;
        }
        let source = format!(
            "{prefix}fn f<{parameters}T:Copy>(x:({fields}))->({fields}){{let c=|v|v;c(x)}}"
        );
        let folder = root()?
            .join("obj/parallel/closure-inputs/scaling")
            .join(std::process::id().to_string())
            .join(width.to_string());
        std::fs::create_dir_all(&folder)?;
        let input = folder.join("input.rs");
        let metadata = folder.join("input.rmeta");
        std::fs::write(&input, &source)?;
        let oracle = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(&metadata)
            .arg(&input)
            .env("RUSTC_BOOTSTRAP", "1")
            .output()?;
        std::fs::write(folder.join("rustc.stderr"), &oracle.stderr)?;
        assert_eq!(
            oracle.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&oracle.stderr)
        );
        std::fs::remove_file(metadata)?;
        let report = crate::provider_traits::report(&source)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("source marker inputs width={width} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= 4 * old + 4096, "{width}: {before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and independent Rust execution"]
fn ordinary_inferred_inputs_execute_like_rustc() -> Result {
    let mut inputs = Vec::new();
    for packet in ORDINARY {
        for case in cases(packet)?.into_iter().filter(|case| case.accepted) {
            let name = Path::new(&case.path)
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or("ordinary input name")?
                .to_owned();
            inputs.push((name, crate::resolve::fixture(&case.path)?));
        }
    }
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str(), false))
        .collect();
    assert_eq!(cases.len(), 26);
    crate::native_references::check_literal_cases_capped(
        "parallel/closure-inputs/native-ordinary",
        &cases,
    )
}

fn independent_source(folder: &Path, name: &str, expected: &str) -> Result {
    use crate::provider_traits::native::run;
    let library = folder.join("rust.a");
    let binary = folder.join("rust.native");
    run(
        folder,
        "rustc",
        Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--target=x86_64-unknown-linux-gnu",
                "--crate-type=staticlib",
                "--crate-name=minicore",
                "-Cpanic=abort",
                "--cap-lints=allow",
            ])
            .arg(root()?.join(format!("{SOURCE}/{name}.rs")))
            .arg("-o")
            .arg(&library),
    )?;
    run(
        folder,
        "rust-link",
        Command::new("cc")
            .arg(root()?.join(format!("{SOURCE}/{name}-observer.c")))
            .arg(&library)
            .arg("-o")
            .arg(&binary),
    )?;
    let output = Command::new(binary).output()?;
    std::fs::write(folder.join("rust-execute.stdout"), &output.stdout)?;
    std::fs::write(folder.join("rust-execute.stderr"), &output.stderr)?;
    assert_eq!(output.status.code(), Some(0), "{name}");
    assert_eq!(output.stdout, expected.as_bytes(), "{name}");
    assert!(output.stderr.is_empty(), "{name}");
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend, independent Rust and C linker"]
fn source_pattern_and_generic_consumers_execute_inferred_inputs() -> Result {
    for (name, _, expected) in NATIVES {
        let folder = root()?
            .join("obj/parallel/closure-inputs/native-source")
            .join(std::process::id().to_string())
            .join(name);
        std::fs::create_dir_all(&folder)?;
        independent_source(&folder, name, expected)?;
        let source = crate::resolve::fixture(&format!("{SOURCE}/{name}.rs"))?;
        let observer = crate::resolve::fixture(&format!("{SOURCE}/{name}-observer.c"))?;
        crate::ffi_program::check(
            root()?,
            &folder,
            name,
            &lower(&source)?,
            &observer,
            expected,
        )?;
    }
    Ok(())
}
