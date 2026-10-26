//! Real pointer refinements retain logical identity and use ordinary source consumers.
use super::*;
use crate::{
    test_source,
    types::{
        Constraint, ItemId, Kind, Primitive, TypeId,
        parameters::{Parameter, ParameterKind},
    },
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const PROVIDER: &str = "pa31/tests/spec/source-nonnull/source-methods.rs";
#[test]
fn structural_storage_and_budget_preserve_logical_constraints_and_unknown_metadata() -> Result {
    let mut types = Store::default();
    let mut defs = Definitions::default();
    let mut layouts = Layouts::default();
    let u8_ = TypeId::primitive(Primitive::U8);
    let raw = types.intern(Kind::Pointer {
        mutable: false,
        pointee: u8_,
    })?;
    let pattern = types.intern(Kind::Pattern {
        base: raw,
        constraint: Constraint::NonNull,
    })?;
    assert_ne!(pattern, raw);
    let mut work = 0;
    assert_eq!(
        compare(
            &mut types,
            &mut defs,
            &mut layouts,
            [pattern, raw],
            &mut work,
            1000
        )?,
        StorageRelation::Equal
    );
    assert!(
        compare(
            &mut types,
            &mut defs,
            &mut layouts,
            [pattern, raw],
            &mut work,
            0
        )
        .is_err()
    );
    let a = types.intern_parameter(Parameter {
        owner: ItemId(77),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let b = types.intern_parameter(Parameter {
        owner: ItemId(77),
        index: 1,
        kind: ParameterKind::Type { sized: false },
    })?;
    let a = types.intern(Kind::Parameter(a))?;
    let b = types.intern(Kind::Parameter(b))?;
    let a = types.intern(Kind::Pointer {
        mutable: false,
        pointee: a,
    })?;
    let b = types.intern(Kind::Pointer {
        mutable: true,
        pointee: b,
    })?;
    let mut work = 0;
    assert_eq!(
        compare(&mut types, &mut defs, &mut layouts, [a, b], &mut work, 1000)?,
        StorageRelation::NeedConcreteLayout
    );
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    eprintln!(
        "Pattern layouts Kind={} AST={} Node={} Store={} Constructor={} Context={} Unit={} Package={} Report={} Body={} Reservations={}",
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<Store>(),
        size_of::<crate::types::infer::Constructor>(),
        size_of::<crate::types::infer::Context>(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::resolve::bindings::Reservations>()
    );
    Ok(())
}
#[test]
fn actual_source_nonnull_methods_preserve_const_metadata_and_emit_runtime_bodies() -> Result {
    let source = test_source::fixture(PROVIDER)?;
    let report = test_source::source_provider(&source, "nonnull_provider")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let wide = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "WIDE")
        .ok_or("wide declaration")?;
    let value = crate::sema::values::Reader::new(&facts)
        .definition(crate::resolve::model::DefId(u32::try_from(wide)?))
        .ok_or("wide const")?;
    let field = crate::sema::values::Reader::new(&facts).field(value, 0)?;
    assert!(matches!(
        field.metadata(),
        Some(crate::sema::values::Metadata::Length(4))
    ));
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
        "pointer_read",
        "pointer_cast",
        "wide_first",
        "numeric_address",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-nonnull");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn actual_feature_origins_null_values_and_documentary_markers_grant_no_permission() -> Result {
    let source = test_source::fixture(PROVIDER)?;
    for (name, changed, needle) in [
        (
            "null",
            source.replace("transmute(1usize)", "transmute(0usize)"),
            "null",
        ),
        (
            "syntax_gate",
            source.replace(
                "intrinsics,pattern_type_macro,pattern_types",
                "intrinsics,pattern_type_macro",
            ),
            "pattern_types",
        ),
        (
            "macro_gate",
            source.replace(
                "intrinsics,pattern_type_macro,pattern_types",
                "intrinsics,pattern_types",
            ),
            "pattern_type_macro",
        ),
        (
            "unsafe",
            source.replace("unsafe { transmute(1usize) }", "transmute(1usize)"),
            "unsafe",
        ),
    ] {
        let error = match test_source::source_provider(&changed, name) {
            Ok(report) => crate::pipeline::analyze_source(&report)
                .err()
                .map(|error| error.to_string()),
            Err(error) => Some(error),
        }
        .ok_or_else(|| format!("{name} unexpectedly accepted"))?;
        assert!(error.contains(needle), "{name}: {error}");
    }
    let prefix = source
        .split("// Rust88d9 ptr/non_null.rs70")
        .next()
        .ok_or("provider prefix")?
        .replace(
            "intrinsics,pattern_type_macro,pattern_types",
            "intrinsics,pattern_type_macro,allow_internal_unstable",
        );
    for (name, body, accepted) in [
        (
            "literal_permission",
            "#[allow_internal_unstable(pattern_types)] macro_rules! make {()=>{pattern_type!(*const u8 is !null)}} type Selected=make!();",
            true,
        ),
        (
            "captured_permission",
            "#[allow_internal_unstable(pattern_types)] macro_rules! pass {($t:ty)=>{$t}} type Selected=pass!(pattern_type!(*const u8 is !null));",
            false,
        ),
        (
            "disabled_permission",
            "#[cfg(any())] type Selected=pattern_type!(*const u8 is !null);",
            true,
        ),
        (
            "documentary_only",
            "#[rustc_nonnull_optimization_guaranteed] struct Ordinary { pointer:*const u8 }",
            true,
        ),
        (
            "wrong_documentary_owner",
            "#[rustc_nonnull_optimization_guaranteed] fn ordinary(){}",
            false,
        ),
    ] {
        let result =
            test_source::source_provider(&format!("{prefix}{body}"), name).and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    let ordinary = test_source::resolve(
        "fn main(){let value=1u8;let pointer=&value as *const u8;}",
        true,
    )?;
    crate::pipeline::analyze_source(&ordinary)?;
    Ok(())
}
