//! Source Unsize upcasts retain intrinsic bounds, equality, markers and lifetimes.
use crate::provider_traits::{TestResult, lower};
const FOLDER: &str = "pa31/tests/spec/unsizing-upcasts";
#[test]
fn actual_source_upcasts_use_intrinsic_object_bounds_and_all_caller_debts() -> TestResult {
    let cases = crate::resolve::fixture(&format!("{FOLDER}/cases.tsv"))?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in cases.lines() {
        let (name, status) = line.split_once('\t').ok_or("upcast columns")?;
        let source = crate::resolve::fixture(&format!("{FOLDER}/{name}"))?;
        let result = lower(&source);
        match (status, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error)) => {
                let message = error.to_string();
                if [
                    "not implemented",
                    "budget",
                    "exhausted",
                    "undefined",
                    "unsupported",
                ]
                .iter()
                .any(|fence| message.contains(fence))
                {
                    failures.push(format!("{name}: {message}"));
                }
                println!("{name}: {message}");
            }
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 10);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn real_source_reference_upcast_and_dispatch_execute() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/source-upcast/native");
    std::fs::create_dir_all(&folder)?;
    let client = crate::resolve::fixture("pa31/tests/spec/unsizing-roles/array-slice.c")?;
    for name in ["supertrait", "associated", "actual-value"] {
        let source = crate::resolve::fixture(&format!("{FOLDER}/{name}.rs"))?;
        let program = lower(&source)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        crate::ffi_program::check(root, &folder, name, &program, &client, "7\n")?;
    }
    Ok(())
}

#[test]
fn object_upcast_work_follows_its_supertrait_diamond_without_scanning_unrelated_types() -> TestResult
{
    use std::fmt::Write as _;
    let foundation = crate::resolve::fixture(&format!("{FOLDER}/supertrait.rs"))?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = foundation.clone();
        for index in 0..width {
            writeln!(source, "trait Side{index}:Base{{}}")?;
        }
        source.push_str("trait Wide:");
        for index in 0..width {
            if index != 0 {
                source.push('+');
            }
            write!(source, "Side{index}")?;
        }
        source.push_str("{}fn wide(){");
        for _ in 0..8 {
            source.push_str("require::<dyn Wide,dyn Base>();");
        }
        source.push('}');
        let mut expected = None;
        for noise in [0, 8192] {
            let report = crate::resolve::source_provider(&source, "object_upcast_work")?;
            let mut facts = rustgm::sema::declarations::analyze(&report)?;
            for length in 0..noise {
                facts.types.intern(rustgm::types::Kind::Array {
                    element: rustgm::types::TypeId::primitive(rustgm::types::Primitive::U8),
                    length,
                })?;
            }
            let counters = |facts: &rustgm::sema::model::Declarations| {
                [
                    facts.body_work.canonical_steps,
                    facts.body_work.trait_selections,
                    facts.body_work.projection_steps,
                    facts.body_work.projection_selections,
                ]
            };
            let before = counters(&facts);
            let facts = rustgm::sema::bodies::analyze(&report, facts)?;
            let after = counters(&facts);
            let observed = std::array::from_fn::<_, 4, _>(|index| after[index] - before[index]);
            assert!(observed[1] > 0);
            if let Some(expected) = expected {
                assert_eq!(observed, expected, "width{width} noise{noise}");
            }
            expected = Some(observed);
            println!("width={width} noise={noise} work={observed:?}");
        }
        let observed = expected.ok_or("work observation")?;
        if let Some(previous) = previous {
            assert!(
                observed[0] <= previous * 6,
                "supertrait diamond work must remain linear"
            );
        }
        previous = Some(observed[0]);
    }
    Ok(())
}
