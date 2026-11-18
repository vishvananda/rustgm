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

fn saturating_source() -> Result<String> {
    let original = source::fixture(&format!("{PACKET}/ordinary.rs"))?;
    let (roles, _) = original
        .split_once("const fn unchecked_mul")
        .ok_or("source roles")?;
    Ok(format!(
        r#"{roles}
        #[rustc_intrinsic_const_stable_indirect]
        #[rustc_nounwind]
        #[rustc_intrinsic]
        pub const fn saturating_add<T: Copy>(a: T, b: T) -> T;
        #[rustc_intrinsic_const_stable_indirect]
        #[rustc_nounwind]
        #[rustc_intrinsic]
        pub const fn saturating_sub<T: Copy>(a: T, b: T) -> T;
        const U8_FLOOR: u8 = saturating_sub(5, 6);
        const U8_CEILING: u8 = saturating_add(250, 10);
        const I8_FLOOR: i8 = saturating_sub(129u8 as i8, 2);
        const I8_CEILING: i8 = saturating_sub(126, 251u8 as i8);
        const I128_FLOOR: i128 = saturating_sub(170141183460469231731687303715884105728u128 as i128, 1);
        const U128_CEILING: u128 = saturating_add(340282366920938463463374607431768211455, 1);
        const NO_OVERFLOW: u8 = saturating_sub(7, 2);
        #[unsafe(no_mangle)]
        pub extern "C" fn add_i8(a: i8, b: i8) -> i8 {{ saturating_add(a, b) }}
        #[unsafe(no_mangle)]
        pub extern "C" fn sub_u8(a: u8, b: u8) -> u8 {{ saturating_sub(a, b) }}
        "#
    ))
}

#[test]
fn source_saturation_preserves_signed_and_unsigned_target_widths() -> Result {
    let input = saturating_source()?;
    let report = source::source_provider(&input, "saturating_arithmetic")?;
    let analysis = pipeline::analyze_source(&report)?;
    for (name, bits) in [
        ("U8_FLOOR", 0),
        ("U8_CEILING", 255),
        ("I8_FLOOR", 128),
        ("I8_CEILING", 127),
        ("I128_FLOOR", 1_u128 << 127),
        ("U128_CEILING", u128::MAX),
        ("NO_OVERFLOW", 5),
    ] {
        assert_eq!(constant(&report, &analysis.facts, name)?, bits, "{name}");
    }
    crate::provider_traits::lower_report(&report)?;
    let bad = format!("{input} const BAD: bool = saturating_sub(false, true);");
    let report = source::source_provider(&bad, "noninteger_saturation")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("accepted noninteger saturation")?;
    assert!(
        error.to_string().contains("supported closed scalar type"),
        "{error}"
    );
    Ok(())
}

#[test]
#[ignore = "requires pinned CPPGM and the host C linker"]
fn source_saturation_executes_through_the_authenticated_runtime_path() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/intrinsic-next/saturation-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let program = crate::provider_traits::lower(&saturating_source()?)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "saturation",
        &program,
        r#"#include <stdint.h>
        #include <stdio.h>
        extern int8_t add_i8(int8_t, int8_t);
        extern uint8_t sub_u8(uint8_t, uint8_t);
        int main(void) {
            printf("%d %d %d %d %d\n", add_i8(-127, -2), add_i8(126, 5), add_i8(3, 4),
                   sub_u8(5, 6), sub_u8(7, 2));
        }"#,
        "-128 127 7 0 5\n",
    )?;
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
