//! An always-const premise proves a `[const]` requirement, as rustc's host
//! effect `satisfies` does: stdarch's `Simd` compares `[T; N]` under
//! `T: SimdElement`, whose supertrait is `const PartialEq`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/always-const-premises";
const UNSTABLE: &str = "#[rustc_const_unstable(feature = \"simd_eq_fixture\", issue = \"none\")]\n";
const SCALAR: &str = "pub const fn f<T: SimdElement>(a: &T, b: &T) -> bool {\n    *a == *b\n}\n";
const UNMATCHED: &str = "trait obligation requires an unambiguous matching assumption";
/// rustc reports E0277 for the first four and E0284 for the ambiguous pair.
const REJECTED: [(&str, &str); 5] = [
    ("plain-supertrait.rs", UNMATCHED),
    ("maybe-plain-premise.rs", UNMATCHED),
    ("const-goal-maybe-premise.rs", "can't compare `T` with `T`"),
    (
        "const-impl-maybe-premise.rs",
        "can't compare `W<T>` with `W<T>`",
    ),
    (
        "both-premises.rs",
        "ambiguous trait assumptions in declaration",
    ),
];

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "const_premises")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            matches!(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name),
                "entry" | "probe" | "range_entry" | "range_probe"
            )
        })
        .map(|function| function.definition)
        .collect::<Vec<_>>();
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}

#[test]
fn always_const_premises_prove_conditional_requirements_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("const premise columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid const premise outcome".into()),
        };
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@const_premises",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "const_premises",
            ])
            .arg(root.join(path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        if output.status.code() != Some(i32::from(!accepted))
            || accepted != reason.is_none()
            || reason.is_some_and(|reason| !stderr.contains(reason))
        {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
        if accepted && let Err(error) = program(&crate::resolve::fixture(path)?) {
            failures.push(format!("{path}: prepared lowering: {error}"));
        }
        count += 1;
    }
    assert_eq!(count, 18);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` const functions, each comparing through the supertrait.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "{UNSTABLE}pub const fn f{index}<T: SimdElement>(a: &T, b: &T) -> bool {{ *a == *b }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear, and every width completes prepared lowering.
#[test]
fn const_premise_work_tracks_functions() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/scalar-op.rs"))?;
    let prefix = source
        .strip_suffix(&format!("{UNSTABLE}{SCALAR}"))
        .ok_or("const premise suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "const_premises")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("const premises functions={functions} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}

const DEFAULTS: &str = "pa31/tests/spec/default-supertraits";

#[test]
fn defining_defaults_consume_exact_completed_self_supertrait_premises() -> Result {
    for name in [
        "conditional",
        "always",
        "generic",
        "method-where",
        "chain",
        "ordinary-control",
        "constant",
        "original-max",
        "builtin-destruct",
        "method-const-bound",
    ] {
        program(&crate::resolve::fixture(&format!("{DEFAULTS}/{name}.rs"))?)
            .map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("plain", "trait obligation requires"),
        ("unrelated", "trait obligation requires"),
        ("wrong-argument", "incompatible types"),
        ("method-plain-bound", "trait obligation requires"),
        ("runtime-destructor", "trait obligation requires"),
    ] {
        let error = program(&crate::resolve::fixture(&format!("{DEFAULTS}/{name}.rs"))?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn ordinary_const_default_minima_preserve_supertrait_effects() -> Result {
    for (name, accepted) in [
        ("conditional", true),
        ("always", true),
        ("generic", true),
        ("method-where", true),
        ("chain", true),
        ("ordinary-control", true),
        ("plain", false),
        ("unrelated", false),
        ("wrong-argument", false),
        ("method-const-bound", true),
        ("method-plain-bound", false),
    ] {
        let text =
            crate::resolve::fixture(&format!("pa25/tests/spec/default-supertraits/{name}.rs"))?;
        let result = (|| -> Result {
            let report = crate::resolve::resolve(&text, true)?;
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
            Ok(())
        })();
        if accepted {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result.err().ok_or_else(|| format!("accepted {name}"))?;
            let cause = if name == "wrong-argument" {
                "incompatible types"
            } else {
                "trait obligation requires"
            };
            assert!(error.to_string().contains(cause), "{name}: {error}");
        }
    }
    Ok(())
}

#[test]
fn completed_default_premise_lookup_scales_with_actual_calls() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let text = crate::resolve::fixture(&format!("{DEFAULTS}/scaling-{count}.rs"))?;
        let report = crate::resolve::source_provider(&text, "default_supertrait_scaling")?;
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
        ];
        println!("default supertraits={count} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= old * 4 + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn inherited_const_default_executes_its_selected_implementation() -> Result {
    use std::process::Command;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/default-supertraits/native");
    std::fs::create_dir_all(&out)?;
    for (packet, name, observer, expected) in [
        (DEFAULTS, "constant", "observe.c", "41\n"),
        (DEFAULTS, "builtin-destruct", "observe.c", "41\n"),
        (DEFAULTS, "original-max", "max-observe.c", "41 41\n"),
        (GENERIC_SUBJECTS, "native", "observe.c", "41 59\n"),
        (
            GENERIC_SUBJECTS,
            "runtime-contains",
            "range-observe.c",
            "1 0 1 0\n",
        ),
    ] {
        let ir = out.join(format!("{name}.lowir"));
        let object = out.join(format!("{name}.o"));
        let executable = out.join(format!("{name}.native"));
        rustgm::lowir::writer::write(
            &program(&crate::resolve::fixture(&format!("{packet}/{name}.rs"))?)?,
            &mut std::fs::File::create(&ir)?,
        )?;
        for (stage, command) in [
            (
                "backend",
                Command::new("python3")
                    .arg(root.join("scripts/backend.py"))
                    .arg("--emit-object")
                    .arg(&ir)
                    .arg("-o")
                    .arg(&object),
            ),
            (
                "link",
                Command::new("cc")
                    .arg(root.join(packet).join(observer))
                    .arg(&object)
                    .arg("-o")
                    .arg(&executable),
            ),
            ("native", &mut Command::new(&executable)),
        ] {
            let output = command.output()?;
            std::fs::write(out.join(format!("{name}-{stage}.stdout")), &output.stdout)?;
            std::fs::write(out.join(format!("{name}-{stage}.stderr")), &output.stderr)?;
            assert!(
                output.status.success() && output.stderr.is_empty(),
                "{name}/{stage}: {output:?}"
            );
            if stage == "native" {
                assert_eq!(output.stdout, expected.as_bytes(), "{name}");
            }
        }
    }
    Ok(())
}

const GENERIC_SUBJECTS: &str = "pa31/tests/spec/default-generic-subjects";
#[test]
fn const_defaults_consume_exact_premises_for_other_generic_subjects() -> Result {
    let mut failures = Vec::new();
    for name in [
        "direct",
        "inherited",
        "same-application",
        "native",
        "reference",
        "original-contains",
        "runtime-contains",
    ] {
        if let Err(error) = program(&crate::resolve::fixture(&format!(
            "{GENERIC_SUBJECTS}/{name}.rs"
        ))?) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, cause) in [
        ("plain", "trait obligation requires"),
        ("missing", "trait obligation requires"),
        ("wrong-argument", "incompatible types"),
        ("reference-plain", "trait obligation requires"),
        ("reference-runtime-impl", "trait obligation requires"),
        ("concrete-runtime-impl", "trait obligation requires"),
    ] {
        let error = program(&crate::resolve::fixture(&format!(
            "{GENERIC_SUBJECTS}/{name}.rs"
        ))?)
        .err()
        .ok_or_else(|| format!("accepted {name}"))?;
        if !error.to_string().contains(cause) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn other_subject_premises_and_requests_scale_with_actual_default_calls() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let text = crate::resolve::fixture(&format!("{GENERIC_SUBJECTS}/scaling-{count}.rs"))?;
        let report = crate::resolve::source_provider(&text, "generic_subject_scaling")?;
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
        ];
        println!("default subjects={count} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= old * 4 + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}
