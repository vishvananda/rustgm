//! Genuine source binary declarations, inference, effects and scalar execution stay joined.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    sema::{
        declarations,
        model::{Declarations, Operation},
    },
    support::language_hooks::Hook,
    syntax::ast::{Kind as Syntax, NodeId},
    types::{Kind, Primitive, TypeId},
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-shift-or/{name}.rs"
    ))?)
}
fn analyze(text: &str) -> TestResult<(Report, Analysis)> {
    let report = crate::test_source::source_provider(text, "source_binary")?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn named(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == kind && report.package.names.get(d.name) == name)
        .map(|(i, _)| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("actual {name}").into())
}
fn lower(
    report: &Report,
    analysis: Analysis,
    names: &[&str],
) -> TestResult<crate::lowir::program::Program> {
    let Analysis {
        mut facts,
        prepared,
    } = analysis;
    let roots = names
        .iter()
        .map(|n| named(report, n, DefKind::Function))
        .collect::<TestResult<Vec<_>>>()?;
    Ok(crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
#[test]
fn real_source_operands_rhs_inference_and_output_precede_primitive_erasure() -> TestResult {
    let (report, Analysis { mut facts, .. }) = analyze(&input("runtime-overloads")?)?;
    let site = report.resolution.definitions
        [named(&report, "encoded", DefKind::Function)?.0 as usize]
        .node
        .ok_or("actual source")?;
    let header = source_binary(&report, &mut facts, site, Hook::Shl)?;
    assert!(matches!(
        facts.types.kind(header.rhs),
        Some(Kind::Parameter(_))
    ));
    let mut scalar = 0;
    let mut overloaded = 0;
    let mut inferred_i32 = 0;
    for (index, node) in report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        let Syntax::Binary {
            op: crate::lex::token::Punctuation::ShiftLeft | crate::lex::token::Punctuation::Or,
            left,
            right,
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: site.file,
            node: NodeId::new(u32::try_from(index)?),
        };
        let Some(left) = facts.node(NodeKey { node: left, ..key }).and_then(|f| f.ty) else {
            continue;
        };
        if matches!(facts.types.kind(left), Some(Kind::Primitive(_))) {
            scalar += 1;
            assert!(!facts.methods.calls.contains_key(&key));
            assert!(matches!(
                facts.node(key).and_then(|f| f.operation),
                Some(Operation::Binary { .. })
            ));
            if facts
                .node(NodeKey { node: right, ..key })
                .and_then(|f| f.ty)
                == Some(TypeId::primitive(Primitive::I32))
            {
                inferred_i32 += 1;
            }
        } else {
            overloaded += 1;
            assert!(facts.methods.calls.contains_key(&key));
        }
    }
    assert!(scalar >= 5 && overloaded >= 2 && inferred_i32 >= 1);
    Ok(())
}
#[test]
fn actual_overloads_and_explicit_custom_bodies_reach_original_instance_lowering() -> TestResult {
    for (name, exports) in [
        ("runtime-overloads", ["encoded", "shifted", "ored"]),
        (
            "const-followup/custom-runtime-only",
            ["syntax", "explicit_shift", "explicit_or"],
        ),
    ] {
        let (report, analysis) = analyze(&input(name)?)?;
        let program = lower(&report, analysis, &exports)?;
        for name in exports {
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == name && s.body.is_some())
                    .count(),
                1
            );
        }
        assert!(program.symbols.iter().filter(|s| s.body.is_some()).count() >= 4);
    }
    Ok(())
}
#[test]
fn actual_const_effects_and_negative_overflow_faults_remain_required() -> TestResult {
    let source =
        input("const-followup/primitive-const-impl")? + "\npub fn const_value()->usize{ENCODED}";
    let (report, analysis) = analyze(&source)?;
    let program = lower(&report, analysis, &["const_value"])?;
    use crate::lowir::program::{Literal, Operand, Terminator};
    let body = program
        .symbols
        .iter()
        .find_map(|s| s.body.as_ref())
        .ok_or("actual const body")?;
    assert!(body.blocks.iter().any(|b|matches!(b.terminator,Terminator::Return(Some(v)) if matches!(v.operand,Operand::Literal(id) if matches!(body.literals.get(id.0 as usize),Some(Literal::Integer(7)))))));
    for name in [
        "primitive-const",
        "explicit-nonconst",
        "nonconst-overload",
        "const-followup/const-negative-impl",
        "const-followup/const-overflow-impl",
    ] {
        let report = crate::test_source::source_provider(&input(name)?, "source_const_op")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} invalid const accepted"))?;
        eprintln!("source-const opponent {name}: {error}");
        assert!(
            !error.message.contains("not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn source_binary_headers_reject_wrong_formals_members_and_replayed_images() -> TestResult {
    let original = input("runtime-overloads")?;
    for text in [
        original.replace("fn shl(self, rhs: Rhs)", "fn shl(&self, rhs: Rhs)"),
        original.replace("Shl<Rhs = Self>", "Shl<Rhs = u8>"),
        original.replace(
            "fn shl(self, rhs: Rhs) -> Self::Output",
            "fn shl(self, rhs: Rhs) -> bool",
        ),
    ] {
        assert!(analyze(&text).is_err());
    }
    let (mut report, Analysis { mut facts, .. }) = analyze(&original)?;
    let site = report.resolution.definitions
        [named(&report, "encoded", DefKind::Function)?.0 as usize]
        .node
        .ok_or("source site")?;
    let selected = source_binary(&report, &mut facts, site, Hook::Shl)?;
    let old = facts.body_work.canonical_steps;
    source_binary(&report, &mut facts, site, Hook::Shl)?;
    assert!(facts.body_work.canonical_steps > old);
    let old_signature = facts.definitions[selected.method.0 as usize].replace(TypeId::UNIT);
    assert!(source_binary(&report, &mut facts, site, Hook::Shl).is_err());
    facts.definitions[selected.method.0 as usize] = old_signature;
    report.package.units[0].source =
        crate::support::source::SourceFile::new(original.into_bytes())?;
    assert!(source_binary(&report, &mut facts, site, Hook::Shl).is_err());
    Ok(())
}
#[test]
fn source_operator_work_scales_and_ordinary_path_retains_hot_layouts() -> TestResult {
    let original = input("const-followup/custom-runtime-only")?;
    for count in [1, 8, 32, 128] {
        let mut source = original.clone();
        write!(source, "pub fn repeated(mut x:usize)->usize{{")?;
        for _ in 0..count {
            source.push_str("x=x<<1i32|1usize;");
        }
        source.push_str("x}");
        let (report, Analysis { mut facts, .. }) = analyze(&source)?;
        let site = report.resolution.definitions
            [named(&report, "repeated", DefKind::Function)?.0 as usize]
            .node
            .ok_or("actual repeated source")?;
        let before = facts.body_work.canonical_steps;
        for _ in 0..count {
            source_binary(&report, &mut facts, site, Hook::Shl)?;
        }
        let work = facts.body_work.canonical_steps - before;
        assert!(work <= count * 1000);
        println!(
            "binary-work operators={} header_work={work} body_work={}",
            2 * count,
            facts.body_work.canonical_steps
        );
    }
    let report = crate::test_source::resolve("pub fn ordinary(x:usize)->usize{x<<1|1}", true)?;
    let facts = declarations::analyze(&report)?;
    assert!(facts.source_bindings.is_none());
    println!(
        "SourceBinary={} SourceUnary={} CallContexts={} Operation={} NodeFacts={} Body={} Source={}",
        size_of::<SourceBinary>(),
        size_of::<crate::sema::compiler_items::unary_contract::SourceUnary>(),
        size_of::<crate::sema::bodies::CallContexts>(),
        size_of::<Operation>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::sema::traits::selection::Source>()
    );
    assert_eq!(size_of::<Operation>(), 16);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    Ok(())
}
