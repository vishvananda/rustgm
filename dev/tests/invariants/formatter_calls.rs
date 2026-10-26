//! A const formatter constructor keeps a genuine target for later runtime ABI calls.
use super::*;
use crate::{
    resolve::model::DefId,
    sema::values::{Constant, Reader, WordTarget},
    types::{Kind, Primitive, TypeId},
};
const PROVIDER: &str = "pa28/tests/spec/source-transmute/formatter-formation-and-runtime.rs";
fn definition(report: &crate::resolve::expansion::Report, name: &str) -> Result<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .ok_or_else(|| format!("missing {name}"))?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn const_nonconst_formatter_keeps_original_target_and_emits_real_runtime_owner_calls() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let report = crate::test_source::source_provider(&source, "erased_formatter")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let original = definition(&report, "format_value")?;
    assert!(!facts.is_const_function(original));
    let stored = definition(&report, "FORMATTER")?;
    let value = Reader::new(&facts)
        .definition(stored)
        .ok_or("actual const pointer")?;
    let target = value
        .as_function()
        .ok_or("actual original function target")?;
    assert!(
        matches!(facts.types.kind(target.ty()),Some(Kind::FunctionItem{item,..}) if item.0==original.0)
    );
    let destination = facts
        .definitions
        .get(stored.0 as usize)
        .copied()
        .flatten()
        .ok_or("destination fn type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(target.ty()) else {
        return Err("original signature".into());
    };
    assert_ne!(signature, destination, "this is genuine signature erasure");
    assert!(Reader::new(&facts).function_word(destination, WordTarget::Value(value))? == value);
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in [
        "const_formatter",
        "owned_formatter",
        "replaced_raw",
        "passed_raw",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-erased-formatter");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    eprintln!(
        "erased formatter Body={} Catalog={} Resolution={} Package={} Origins={} Constant={}",
        size_of::<ir::Body>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::body::operand_origins::Origins<'_>>(),
        size_of::<Constant>()
    );
    Ok(())
}
#[test]
fn function_word_validity_never_fabricates_a_target_from_numeric_or_data_storage() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let report = crate::test_source::source_provider(&source, "erased_word_validity")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let stored = definition(&report, "FORMATTER")?;
    let expected = facts.definitions[stored.0 as usize].ok_or("function type")?;
    let function = Reader::new(&facts).definition(stored).ok_or("function")?;
    let scalar = Constant::scalar(crate::sema::model::Scalar::Integer(0));
    let address = {
        let mut evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
        evaluator.allocation(Some(scalar), TypeId::primitive(Primitive::U8))?
    };
    let reader = Reader::new(&facts);
    for invalid in [
        scalar,
        Constant::raw_address(0),
        Constant::raw_address(1),
        address,
    ] {
        assert!(
            reader
                .function_word(expected, WordTarget::Value(invalid))
                .is_err()
        );
    }
    assert!(
        reader
            .function_word(
                TypeId::primitive(Primitive::Usize),
                WordTarget::Value(function)
            )
            .is_err()
    );
    assert!(
        reader
            .function_word(
                expected,
                WordTarget::Vtable {
                    concrete: TypeId::UNIT,
                    target: TypeId::UNIT
                }
            )
            .is_err()
    );
    assert!(reader.function_word(expected, WordTarget::Value(function))? == function);
    Ok(())
}
#[test]
fn erased_formatter_owner_lifetime_unsafe_and_const_invocation_remain_checked() -> Result {
    for (name, root, expected) in [
        (
            "borrowed-owner-escape",
            "escaped",
            "required lifetime does not outlive its target",
        ),
        ("erased-unsafe-call", "unchecked", "unsafe"),
        ("erased-const-call", "execute", "const"),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa28/tests/spec/source-transmute/{name}.rs"))?;
        let report = crate::test_source::source_provider(&source, "erased_opponent")?;
        let result = check_root(&report, root);
        let error = match result {
            Ok(()) => return Err(format!("{name}: incorrectly accepted").into()),
            Err(error) => error.to_string(),
        };
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
fn check_root(report: &crate::resolve::expansion::Report, name: &str) -> Result {
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(report)?;
    let root = definition(report, name)?;
    crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[root],
        },
        prepared,
    )?;
    Ok(())
}
