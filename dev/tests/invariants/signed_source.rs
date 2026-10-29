//! Actual source signed offsets are distinct from ordinary same-named function bodies.
use super::*;
use crate::{
    pipeline,
    sema::model::Scalar,
    sema::values::{Constant, Evaluator, Reader, View},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-wrapping-offset/{name}.rs"
    ))?)
}
fn report(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "source_wrapping_offset",
    )?)
}
#[test]
fn actual_source_signed_wrapping_and_distinct_ordinary_body_reach_native_inputs() -> TestResult {
    for name in ["source-caller", "unmarked-lookalike", "generic-forward"] {
        let report = report(name)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        for symbol in [
            "offset_word",
            "offset_byte",
            "offset_zero",
            "read_back",
            "const_round",
            "const_max_back",
            "const_min_stay",
            "const_zero",
            "const_raw",
        ] {
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == symbol && s.body.is_some())
                    .count(),
                1,
                "{name}: {symbol}"
            );
        }
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/wrapping-offset-v1/candidate")
            .join(name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("source.rs"), source(name)?)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
#[test]
fn actual_source_const_offsets_keep_provenance_raw_words_and_zero_size() -> TestResult {
    let report = report("source-caller")?;
    let analysis = pipeline::analyze_source(&report)?;
    for (name, expected) in [
        ("ROUND", 0x1122_3344),
        ("MAX_BACK", 0x5566_7788),
        ("MIN_STAY", 0xaabb_ccdd),
        ("ZERO", 7),
    ] {
        let id = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(i, d)| {
                (report.package.names.get(d.name) == name).then_some(DefId(u32::try_from(i).ok()?))
            })
            .ok_or("actual source constant")?;
        let reader = Reader::new(&analysis.facts);
        let value = reader
            .definition(id)
            .ok_or("source constant was not evaluated")?;
        assert!(
            matches!(reader.view(value)?,View::Scalar(Scalar::Integer(n)) if n==expected),
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn source_signature_unsafe_const_and_reached_access_opponents_remain_independent() -> TestResult {
    for name in [
        "wrong-signature",
        "wrong-generics",
        "unsafe-call",
        "nonconst-source-target",
        "reached-invalid-deref",
    ] {
        let result = report(name).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        assert!(result.is_err(), "{name}: actual invalid source accepted");
        eprintln!("{name}: {}", result.err().ok_or("missing error")?);
    }
    Ok(())
}
#[test]
fn wrapping_word_kernel_preserves_root_write_restrictions_and_failed_work() -> TestResult {
    let report = crate::test_source::resolve("const VALUE:u8=7;", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let byte = TypeId::primitive(Primitive::U8);
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let root = evaluator.allocation(None, byte)?;
    for incoming in [root, root.restrict_shared()] {
        let mut work = 0;
        let far = evaluator.wrapping_byte_projection(incoming, u64::MAX, byte, &mut work, 1024)?;
        let back = evaluator.wrapping_byte_projection(far, 1, byte, &mut work, 1024)?;
        let (address, _) = evaluator.word_address(back, 1024)?;
        assert_eq!(address.root(), root.location().ok_or("root")?);
        assert_eq!(address.offset(), 0);
        assert_eq!(back.write_permission(), incoming.write_permission());
        assert!(
            Reader::new(evaluator.facts())
                .location(address.root())?
                .value
                .is_none()
        );
        let zero = evaluator.wrapping_byte_projection(incoming, 0, byte, &mut work, 1024)?;
        assert!(zero == incoming);
    }
    let mut work = 0;
    assert!(
        evaluator
            .wrapping_byte_projection(root, 1, byte, &mut work, 0)
            .is_err()
    );
    assert_eq!(work, 1);
    let mut work = 0;
    let raw = evaluator.wrapping_byte_projection(
        Constant::raw_address(0),
        u64::MAX,
        byte,
        &mut work,
        1024,
    )?;
    assert!(matches!(
        Reader::new(evaluator.facts()).view(raw)?,
        View::Address { bits: u64::MAX }
    ));
    Ok(())
}
#[test]
fn repeated_wrapping_flattens_provenance_with_bounded_work_and_unchanged_hot_records() -> TestResult
{
    let report = crate::test_source::resolve("const VALUE:u8=7;", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let byte = TypeId::primitive(Primitive::U8);
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let root = evaluator.allocation(None, byte)?;
    for width in [8, 32, 128] {
        let mut value = root;
        let mut work = 0;
        for _ in 0..width {
            value = evaluator.wrapping_byte_projection(value, u64::MAX, byte, &mut work, 4096)?;
        }
        let (address, _) = evaluator.word_address(value, 16)?;
        assert_eq!(address.root(), root.location().ok_or("root")?);
        assert_eq!(address.offset(), 0u64.wrapping_sub(width));
        assert!(work <= 3 * usize::try_from(width)?);
        eprintln!("wrapping width={width} work={work}");
    }
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    // Range member registration grows only the cold source catalog here.
    assert_eq!(size_of::<crate::sema::compiler_items::Catalog>(), 1616);
    Ok(())
}
