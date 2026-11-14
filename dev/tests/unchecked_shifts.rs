//! Actual integer shift declarations retain both Copy inputs and unsafe caller proof.
use crate::resolve as source;
use rustgm::{
    pipeline,
    sema::values::{Reader, View},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/unchecked-shifts";
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    Ok(source::source_provider(
        &source::fixture(&format!("{PACKET}/{name}.rs"))?,
        "unchecked_shifts",
    )?)
}
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    Ok(crate::provider_traits::lower_report(&report(name)?)
        .map_err(|error| format!("{name}: {error}"))?)
}
fn constant(
    report: &rustgm::resolve::expansion::Report,
    facts: &rustgm::sema::model::Declarations,
    name: &str,
) -> Result<u128> {
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, d)| {
            (report.package.names.get(d.name) == name)
                .then(|| u32::try_from(index).ok().map(rustgm::resolve::model::DefId))
                .flatten()
        })
        .ok_or("actual shift constant")?;
    let reader = Reader::new(facts);
    let value = reader
        .definition(definition)
        .ok_or("missing shift result")?;
    let View::Scalar(rustgm::sema::model::Scalar::Integer(bits)) = reader.view(value)? else {
        return Err("source shift constant is not integer data".into());
    };
    Ok(bits)
}
#[test]
fn every_integer_operand_pair_retains_exact_shift_bits() -> Result {
    let matrix = report("matrix")?;
    let analysis = pipeline::analyze_source(&matrix)?;
    let expected = source::fixture(&format!("{PACKET}/expected-values.tsv"))?;
    let mut count = 0;
    for row in expected.lines() {
        let (name, bits) = row.split_once('\t').ok_or("shift expectation row")?;
        assert_eq!(
            constant(&matrix, &analysis.facts, name)?,
            bits.parse::<u128>()?,
            "{name}"
        );
        count += 1;
    }
    assert_eq!(count, 576);
    let ordinary = report("ordinary")?;
    let analysis = pipeline::analyze_source(&ordinary)?;
    assert_eq!(constant(&ordinary, &analysis.facts, "ORD")?, 99);
    program("actual-wrapper")?;
    program("ordinary")?;
    program("runtime")?;
    Ok(())
}
#[test]
fn invalid_counts_types_safety_constness_copy_and_prototypes_reject() -> Result {
    for name in [
        "negative-count",
        "equal-width",
        "wide-count",
        "noninteger",
        "unsafe-call",
        "missing-copy",
        "nonconst-call",
        "wrong-safety",
        "wrong-result",
    ] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("{name} accepted"))?
            .to_string();
        assert!(!error.contains("not implemented"), "{name}: {error}");
        assert!(!error.contains("budget"), "{name}: {error}");
        if matches!(name, "negative-count" | "equal-width" | "wide-count") {
            assert!(
                error.contains("shift amount is out of range"),
                "{name}: {error}"
            );
        }
        println!("{name}: {error}");
    }
    Ok(())
}
#[test]
fn successive_shift_constants_have_bounded_linear_preparation() -> Result {
    let mut previous = None;
    for count in [8, 32, 128] {
        let report = report(&format!("scaling-{count}"))?;
        let analysis = pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
        ];
        if let Some(old) = previous {
            let old: [usize; 2] = old;
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        println!("shifts count={count} work={work:?}");
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM and host C linker"]
fn source_shift_constants_and_runtime_results_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root
        .join("obj/parallel/unchecked-shifts/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        (
            "actual-wrapper",
            "observe.c",
            "9223372036854775808\n".to_string(),
        ),
        (
            "matrix",
            "observe-matrix.c",
            source::fixture(&format!("{PACKET}/expected-matrix.stdout"))?,
        ),
        (
            "runtime",
            "observe-runtime.c",
            source::fixture(&format!("{PACKET}/expected-runtime.stdout"))?,
        ),
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program(name)?,
            &source::fixture(&format!("{PACKET}/{observer}"))?,
            &expected,
        )?;
    }
    Ok(())
}
