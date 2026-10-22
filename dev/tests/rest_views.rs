//! Rest projections preserve storage identity while owned snapshots stay independent.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod qualify;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/resolve.rs"]
mod source;
use std::path::Path;

#[test]
fn constant_rest_storage_matches_predeclared_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa25/rest-views/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for name in [
        "nested-slice",
        "reference-array",
        "deep-reference",
        "empty-array",
        "mutable-array",
        "mutable-slice",
        "nested-mutable",
        "whole-range-write",
        "reborrow",
        "copied-array",
        "static-range",
        "literal-range",
        "literal-copy",
        "generic-copy",
        "too-short",
        "raw-alias",
        "owned-array",
    ] {
        checked::check(
            &root.join(format!(
                "pa{}/tests/spec/rest-views/{name}.rs",
                if name.starts_with("raw-") { 27 } else { 25 }
            )),
            &evidence,
            name,
            (0, 0, ""),
        )?;
    }
    for (name, reason) in [
        (
            "local-escape",
            "required lifetime does not outlive its target",
        ),
        (
            "empty-local-escape",
            "required lifetime does not outlive its target",
        ),
        ("bounds", "constant index is out of bounds"),
        ("shared-write", "cannot assign through a shared reference"),
        (
            "loan-conflict",
            "access conflicts with a live reference loan",
        ),
        (
            "raw-shared-write",
            "constant raw pointer write is derived from a shared reference",
        ),
        ("raw-expired", "constant pointer refers to expired storage"),
    ] {
        checked::check(
            &root.join(format!(
                "pa{}/tests/spec/rest-views/{name}.rs",
                if name.starts_with("raw-") { 27 } else { 25 }
            )),
            &evidence,
            name,
            (1, 1, reason),
        )?;
    }
    Ok(())
}

#[test]
fn pattern_lengths_read_metadata_without_demanding_static_contents()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa25/rest-views/metadata-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for (stage, name) in [
        (25, "static"),
        (25, "self"),
        (27, "mutable"),
        (28, "interior"),
        (30, "foreign"),
    ] {
        let name = format!("metadata-pattern-{name}");
        checked::check(
            &root.join(format!("pa{stage}/tests/spec/rest-views/{name}.rs")),
            &evidence,
            &name,
            (0, 0, ""),
        )?;
    }
    for name in [
        "ordinary-with-interior-type",
        "byte-with-interior-type",
        "interior-empty",
    ] {
        checked::check(
            &root.join(format!("pa28/tests/spec/rest-views/{name}.rs")),
            &evidence,
            name,
            (0, 0, ""),
        )?;
    }
    Ok(())
}

#[test]
fn range_recipes_normalize_and_preserve_each_pointer_permission()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        sema::values::{Constant, Evaluator, Origin, Reader, View, WritePermission},
        types::{Kind, Primitive, TypeId},
    };
    let (report, mut facts) = semantic::analyze("")?;
    let element = TypeId::primitive(Primitive::U8);
    let array = facts.types.intern(Kind::Array {
        element,
        length: 1_000_000,
    })?;
    let slice = facts.types.intern(Kind::Slice(element))?;
    let small = facts.types.intern(Kind::Array { element, length: 2 })?;
    let text = facts.texts.intern(b"abcdef")?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let value = evaluator.repeat(
        Constant::scalar(rustgm::sema::model::Scalar::Integer(7)),
        1_000_000,
    )?;
    let root = evaluator.allocation(Some(value), array)?;
    let first = evaluator.subsequence(root, 2, 999_998, slice)?;
    let second = evaluator.subsequence(first, 3, 2, small)?;
    let shared = evaluator.subsequence(first.restrict_shared(), 3, 2, small)?;
    let field = evaluator.projection(second, 1, element)?;
    let mut deep = root;
    for depth in 1..=128 {
        deep = evaluator.subsequence(deep, 1, 1_000_000 - depth, slice)?;
        assert!(
            matches!(Reader::new(evaluator.facts()).location(deep.location().ok_or("deep range")?)?.origin, Origin::Subsequence { base, from, .. } if Some(base) == root.location() && from == depth)
        );
    }

    assert_eq!(second.write_permission(), Some(WritePermission::Writable));
    assert_eq!(
        shared.write_permission(),
        Some(WritePermission::SharedDerived)
    );
    assert_eq!(root.write_permission(), Some(WritePermission::Writable));
    let snapshot = evaluator.sequence_range(value, 999_998, 2)?;
    let bytes = evaluator.sequence_range(Constant::bytes(text), 1, 4)?;
    let bytes = evaluator.sequence_range(bytes, 1, 2)?;
    let reader = Reader::new(evaluator.facts());
    assert!(
        matches!(reader.location(second.location().ok_or("range")?)?.origin, Origin::Subsequence { base, from: 5, length: 2 } if Some(base) == root.location())
    );
    assert!(
        matches!(reader.location(field.location().ok_or("element")?)?.origin, Origin::Projection { base, index: 6 } if Some(base) == root.location())
    );
    assert!(matches!(
        reader.view(snapshot)?,
        View::Repeat { length: 2, .. }
    ));
    assert!(matches!(
        reader.view(bytes)?,
        View::ByteRange {
            from: 2,
            length: 2,
            ..
        }
    ));
    assert!(matches!(
        reader.view(reader.field(bytes, 0)?)?,
        View::Scalar(rustgm::sema::model::Scalar::Integer(99))
    ));
    assert!(evaluator.subsequence(first, 999_998, 1, slice).is_err());
    Ok(())
}

#[test]
fn compressed_ranges_touch_no_unselected_elements() -> Result<(), Box<dyn std::error::Error>> {
    for repeated in [false, true] {
        let mut records = Vec::new();
        for width in [8, 1024, 1_000_000] {
            for count in [8, 32, 128] {
                let operation = if repeated {
                    "match a { [_,r @ ..]=>match r { [_,s @ ..]=>s[0],_=>0 },_=>0 }"
                } else {
                    "a[2]"
                };
                let source = format!(
                    "const fn f(n:u32)->u8{{let a=[7u8;{width}];let mut i=0u32;let mut v=0u8;while i<n{{v={operation};i=i+1;}}v}}const N:u8=f({count});fn main(){{let _=N;}}"
                );
                qualify::qualify(
                    &format!("parallel/pa25/rest-views/work-{repeated}-{width}-{count}"),
                    &source,
                    true,
                )?;
                let mut report = source::resolve(&source, true)?;
                let rustgm::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = rustgm::pipeline::analyze_source(&report)?;
                let entry = rustgm::sema::entry::select(&mut report, &facts)?;
                rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
                let measured = (facts.constant_work.steps, facts.constant_work.peak_slots);
                println!(
                    "rest={repeated} width={width} count={count} steps={} slots={}",
                    measured.0, measured.1
                );
                assert!(measured.0 < 512 * (count + 1));
                assert!(measured.1 < 64 * (count + 1));
                if width == 8 {
                    records.push(measured);
                } else {
                    let index = [8, 32, 128]
                        .iter()
                        .position(|&n| n == count)
                        .ok_or("count")?;
                    assert_eq!(measured, records[index]);
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn rest_views_execute_aliasing_and_independent_snapshots() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let cases = [
        "nested-slice",
        "empty",
        "reference-array",
        "mutable-alias",
        "owned-drops",
        "whole-write",
        "nested-mutable",
        "copy-isolation",
        "literal",
        "static",
        "generic",
        "reborrow",
    ]
    .into_iter()
    .map(|name| {
        std::fs::read_to_string(root.join(format!(
            "pa{}/tests/rest-view-native/{name}.rs",
            if matches!(name, "mutable-alias" | "owned-drops") {
                27
            } else {
                25
            }
        )))
        .map(|body| (name, body))
    })
    .collect::<Result<Vec<_>, _>>()?;
    let borrowed = cases
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa25/rest-views/native", &borrowed)
}
