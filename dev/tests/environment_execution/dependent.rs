//! Projection query views share owner slots without transporting proof authority.
use super::{checked_sources, native};
use std::path::PathBuf;
// Rust object outcome, type view outcome, executable outcome, expected diagnostic.
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("borrowed-result", 0, 0, 0, ""),
    ("nested-child", 0, 0, 0, ""),
    ("named-binder", 0, 0, 0, ""),
    ("repeated-clause", 0, 0, 0, ""),
    ("parent-public-binder", 0, 0, 0, ""),
    (
        "borrowed-escape",
        1,
        0,
        1,
        "required lifetime does not outlive its target",
    ),
    (
        "distinct-output",
        1,
        0,
        1,
        "required lifetime does not outlive its target",
    ),
    (
        "distinct-clauses",
        1,
        1,
        1,
        "associated type requires an unambiguous implementation",
    ),
];
const FRONTIERS: &[(&str, i32, &str)] = &[
    (
        "written-input",
        0,
        "required lifetime does not outlive its target",
    ),
    (
        "nested-binder",
        0,
        "conditional nested output shape is not implemented",
    ),
    (
        "nested-input-escape",
        1,
        "conditional nested output shape is not implemented",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/projected-captures/{name}.rs"))
}
#[test]
fn captured_projection_views_match_independent_object_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-projected-repair/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, execution, reason) in CASES {
        checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (rust, [types, execution, execution], reason),
        )?;
    }
    Ok(())
}
#[test]
fn retained_projection_frontiers_are_not_semantic_negative_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-projected-repair/frontiers/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, reason) in FRONTIERS {
        checked_sources::check_modes(&fixture(name), &evidence, name, (rust, [0, 1, 1], reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn captured_projection_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, _, _, execution, _) in CASES {
        if execution == 0 {
            let source = std::fs::read_to_string(fixture(name))?;
            let source = source
                .strip_prefix(native::CHECK)
                .ok_or("missing check prefix")?;
            native::check_cases(
                "parallel/pa24-projected-repair/native",
                &[(name, source, false)],
            )?;
        }
    }
    Ok(())
}

#[test]
fn projected_captures_reuse_owner_environment_and_shared_input_views()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source="trait Pick<'a>{type Out:Copy;}impl<'a> Pick<'a> for u8{type Out=&'a u8;}fn take<'a,T:Pick<'a>>(x:T::Out)->T::Out{".to_owned();
        for index in 0..width {
            write!(source, "let f{index}=move||x;let _=f{index}();")?;
        }
        source.push_str("x}fn main(){let n=7;let _=take::<u8>(&n);}");
        let (_, facts) = super::semantic::analyze(&source)?;
        assert_eq!(facts.closures.all().len(), width);
        let work = facts.body_work.inference_steps
            + facts.body_work.canonical_steps
            + facts.body_work.projection_steps;
        eprintln!("projected captures={width} inference_canonical_projection_steps={work}");
        observations.push(work);
    }
    assert!(
        observations[2] - observations[1] <= 5 * (observations[1] - observations[0]),
        "{observations:?}"
    );
    Ok(())
}
