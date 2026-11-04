//! The CLI preserves actual forced restriction, gates and ordinary trait behavior.
use rustgm::{
    pipeline,
    resolve::model::{DefId, DefKind},
    sema::dyn_compat::{Error, Queries, Rule},
};
use std::fmt::Write as _;
#[path = "helpers/resolve.rs"]
mod test_source;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa31/tests/spec/forced-dyn-policy/{name}.rs")),
    )
}
fn definition(
    report: &rustgm::resolve::expansion::Report,
    name: &str,
) -> Result<DefId, Box<dyn std::error::Error>> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Trait && report.package.names.get(d.name) == name)
        .map(|(i, _)| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| "forced dyn trait".into())
}

#[test]
fn forced_restriction_matches_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases = std::fs::read_to_string(root.join("pa31/tests/spec/forced-dyn-policy/cases.tsv"))?;
    for row in cases.lines() {
        let (path, expected) = row.split_once('\t').ok_or("forced dyn case")?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-constants",
                "--cap-lints=allow",
                "--source-core",
                "probe@forced_dyn_headers",
                "--crate-name",
                "probe",
                "--crate-disambiguator",
                "forced_dyn_headers",
            ])
            .arg(root.join(path))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(cases.lines().count(), 40);
    Ok(())
}

#[test]
fn shared_supertrait_failures_are_typed_cached_and_linear() -> TestResult {
    let prefix = source("declaration")?;
    for width in [16, 64, 128] {
        let mut input = prefix.clone();
        input.push_str("trait Left0:Forced{}trait Right0:Forced{}");
        for i in 1..width {
            let p = i - 1;
            write!(
                input,
                "trait Left{i}:Left{p}+Right{p}{{}}trait Right{i}:Left{p}+Right{p}{{}}"
            )?;
        }
        write!(input, "trait Root:Left{}{{}}", width - 1)?;
        let report = test_source::source_provider(&input, "forced_dyn_graph")?;
        let mut facts = pipeline::analyze_source(&report)?.facts;
        let root = definition(&report, "Root")?;
        let forced = definition(&report, "Forced")?;
        let mut queries = Queries::default();
        let error = queries
            .demand(&report, &mut facts, root)
            .err()
            .ok_or("forced graph accepted")?;
        assert_eq!(
            error,
            Error::Incompatible {
                definition: forced,
                rule: Rule::Forced
            }
        );
        let cold = queries.steps;
        assert!(cold < 40 * width + 100, "{width}: {cold}");
        for _ in 0..16 {
            assert_eq!(queries.demand(&report, &mut facts, root).err(), Some(error));
        }
        assert!(queries.steps - cold <= 32);
    }
    Ok(())
}

#[test]
fn normal_crates_use_the_internal_feature_without_requiring_a_provider() {
    for (input, accepted) in [
        (
            "#![feature(rustc_attrs)]#[rustc_dyn_incompatible_trait]trait Forced{}fn main(){}",
            true,
        ),
        (
            "#![feature(rustc_attrs)]#[rustc_dyn_incompatible_trait]trait Forced{}fn object(_: &dyn Forced){}fn main(){}",
            false,
        ),
        (
            "#[rustc_dyn_incompatible_trait]trait Forced{}fn main(){}",
            false,
        ),
    ] {
        let actual = test_source::resolve(input, true).and_then(|r| {
            pipeline::analyze_source(&r)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(actual.is_ok(), accepted, "{actual:?}");
    }
}

#[test]
fn restriction_preserves_actual_generic_static_dispatch_and_native_inputs() -> TestResult {
    let input = source("native-operation")?;
    let report = test_source::source_provider(&input, "forced_dyn_native")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["selected", "ordinary"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/forced-dyn-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), input)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
