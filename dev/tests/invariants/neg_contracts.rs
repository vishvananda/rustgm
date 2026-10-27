//! Actual Neg declarations/effects precede both discriminants and ordinary calls.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    syntax::ast::{Kind as Syntax, Unary},
    types::{ItemId, Primitive},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-location/neg-{name}.rs"
    ))?)
}
fn definition(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| row.kind == kind && report.package.names.get(row.name) == name)
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("missing actual {name}").into())
}
#[test]
fn actual_discriminants_require_current_neg_and_const_impl_before_scalar_evaluation() -> TestResult
{
    for (name, expected) in [
        ("missing-const", "Neg language declaration"),
        ("missing-discriminant", "Neg language declaration"),
        (
            "ordinary-impl-const-discriminant",
            "trait obligation requires",
        ),
    ] {
        let report = crate::test_source::source_provider(&fixture(name)?, "negative_discriminant")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name}: unproved Neg accepted"))?;
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
    let source = fixture("const-real-discriminant")?;
    let report = crate::test_source::source_provider(&source, "real_discriminant")?;
    let Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
    let owner = ItemId(definition(&report, "Order", DefKind::Enum)?.0);
    let shape = facts.aggregates.get(owner).ok_or("actual enum")?;
    let variants = facts.aggregates.variants(shape);
    assert_eq!(
        variants
            .iter()
            .map(|variant| variant.discriminant)
            .collect::<Vec<_>>(),
        [u128::from(u64::MAX), 0, 1]
    );
    let site = report.resolution.definitions[owner.0 as usize]
        .node
        .ok_or("actual enum source")?;
    let selected = source_neg(&report, &mut facts, site)?;
    assert!(facts.associated.operation(selected.definition).is_none());
    assert!(
        report
            .resolution
            .builtin_traits
            .language(crate::support::language_hooks::Hook::Neg)
            .is_none()
    );
    facts.body_work.canonical_steps = crate::support::crates::MAX_COMPONENTS;
    assert!(source_neg(&report, &mut facts, site).is_err());
    assert!(facts.body_work.canonical_steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
#[test]
fn actual_overloads_keep_calls_and_primitive_impl_bodies_do_not_recurse() -> TestResult {
    let source = fixture("const-real-discriminant")?
        + r"
        pub struct Wrapped(isize);
        impl Neg for Wrapped{type Output=isize;fn neg(self)->isize{-self.0}}
        pub fn scalar(x:isize)->isize{-x}
        pub fn overloaded()->isize{-Wrapped(7)}
        pub const MINIMUM:i8=-128i8;
    ";
    // The minimum literal has its own genuine source implementation and retains
    // the existing literal sign bookkeeping before ordinary scalar erasure.
    let source = source
        + "impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}const impl Neg for i8{type Output=Self;fn neg(self)->Self{-self}}";
    let report = crate::test_source::source_provider(&source, "neg_overloads")?;
    let Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let mut primitive = 0;
    let mut overloaded = 0;
    for (index, node) in report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        let Syntax::Unary {
            op: Unary::Negate,
            operand,
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: crate::syntax::ast::NodeId::new(u32::try_from(index)?),
        };
        let ty = facts
            .node(NodeKey {
                node: operand,
                ..key
            })
            .and_then(|row| row.ty)
            .ok_or("actual operand type")?;
        if matches!(
            facts.types.kind(ty),
            Some(Kind::Primitive(Primitive::Isize | Primitive::I8))
        ) {
            primitive += 1;
            assert!(!facts.methods.calls.contains_key(&key));
        } else {
            overloaded += 1;
            assert!(facts.methods.calls.contains_key(&key));
        }
    }
    assert!(primitive >= 5);
    assert_eq!(overloaded, 1);
    let roots = [
        definition(&report, "scalar", DefKind::Function)?,
        definition(&report, "overloaded", DefKind::Function)?,
    ];
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    for bad in [
        source.replace("fn neg(self)", "fn wrong(self)"),
        source.replace("fn neg(self)->Self::Output;", "fn neg(self)->Self;"),
    ] {
        let report = crate::test_source::source_provider(&bad, "wrong_neg_member")?;
        assert!(pipeline::analyze_source(&report).is_err());
    }
    assert_eq!(size_of::<SourceUnary>(), 12);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    assert_eq!(size_of::<crate::body::ir::Rvalue>(), 20);
    Ok(())
}

#[test]
fn actual_enum_and_const_neg_prepare_native_exports_after_source_proof() -> TestResult {
    let input = fixture("native")?;
    let report = crate::test_source::source_provider(&input, "source_neg_native")?;
    let Analysis {
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
    for name in ["neg_enum", "neg_constant"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.name == name && symbol.body.is_some())
                .count(),
            1
        );
    }
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-neg");
    std::fs::create_dir_all(&output)?;
    std::fs::write(output.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(output.join("provider.lowir"))?,
    )?;
    Ok(())
}
