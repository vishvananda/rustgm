//! Genuine unchecked arithmetic reuses typed call proof and checked constant execution.
use crate::resolve as source;
use rustgm::{
    pipeline,
    sema::values::{Reader, View},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/unchecked-arithmetic";
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    Ok(source::source_provider(
        &source::fixture(&format!("{PACKET}/{name}.rs"))?,
        "unchecked_arithmetic",
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
        .ok_or("actual arithmetic constant")?;
    let reader = Reader::new(facts);
    let value = reader
        .definition(definition)
        .ok_or("missing arithmetic result")?;
    let View::Scalar(rustgm::sema::model::Scalar::Integer(bits)) = reader.view(value)? else {
        return Err("source arithmetic constant is not integer data".into());
    };
    Ok(bits)
}
#[test]
fn every_integer_width_retains_exact_unchecked_arithmetic_bits() -> Result {
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
    assert_eq!(count, 240);
    let ordinary = report("ordinary")?;
    let analysis = pipeline::analyze_source(&ordinary)?;
    assert_eq!(constant(&ordinary, &analysis.facts, "ORD")?, 99);
    program("actual-mul")?;
    program("ordinary")?;
    program("runtime")?;
    program("legacy-sub")?;
    Ok(())
}
#[test]
fn overflow_zero_minimum_types_and_call_contracts_reject() -> Result {
    for (name, reason) in [
        (
            "add-unsigned-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "add-positive-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "add-negative-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "mul-unsigned-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "mul-positive-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "mul-negative-overflow",
            "constant integer arithmetic overflows",
        ),
        (
            "mul-min-negative-one",
            "constant integer arithmetic overflows",
        ),
        (
            "div-zero",
            "constant integer arithmetic overflows or divides by zero",
        ),
        (
            "rem-zero",
            "constant integer arithmetic overflows or divides by zero",
        ),
        ("div-min-negative-one", "overflows"),
        ("rem-min-negative-one", "overflows"),
        (
            "unsafe-call",
            "call to unsafe function requires an unsafe context",
        ),
        ("missing-copy", "trait obligation"),
        ("nonconst-call", "non-const calls"),
        (
            "noninteger",
            "source arithmetic execution requires its supported closed scalar type",
        ),
        ("wrong-safety", "source intrinsic safety differs"),
        ("wrong-result", "source intrinsic signature differs"),
        (
            "wrong-parameters",
            "source intrinsic has the wrong owned generic counts",
        ),
    ] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("{name} accepted"))?
            .to_string();
        assert!(error.contains(reason), "{name}: {error}");
        println!("{name}: {error}");
    }
    Ok(())
}
#[test]
fn successive_multiply_constants_have_bounded_linear_preparation() -> Result {
    let mut previous = None;
    for count in [8, 32, 128] {
        let report = report(&format!("scaling-{count}"))?;
        let analysis = pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.steps,
        ];
        if let Some(old) = previous {
            let old: [usize; 3] = old;
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        println!("unchecked arithmetic count={count} work={work:?}");
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM and host C linker"]
fn source_unchecked_constants_and_runtime_results_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root
        .join("obj/parallel/intrinsic-next/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        (
            "matrix",
            "observe-matrix.c",
            source::fixture(&format!("{PACKET}/expected-matrix.stdout"))?,
        ),
        (
            "runtime",
            "observe-runtime.c",
            source::fixture(&format!("{PACKET}/expected-matrix.stdout"))?,
        ),
        ("actual-mul", "observe-actual.c", "221\n".to_string()),
        ("ordinary", "observe-prior.c", "99\n".to_string()),
        ("legacy-sub", "observe-prior.c", "7\n".to_string()),
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
