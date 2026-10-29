//! Source Div borrows the existing trait, effect, method and primitive execution proofs.
use super::*;
use crate::support::bindings::LangItem;
const TYPES: [&str; 14] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64",
];
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-div/{name}.rs"
    ))?)
}
fn site(report: &Report) -> TestResult<NodeKey> {
    Ok(
        report.resolution.definitions[named(report, "Div", DefKind::Trait)?.0 as usize]
            .node
            .ok_or("actual Div owner")?,
    )
}
fn write_program(name: &str, source: &str, program: &crate::lowir::program::Program) -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-div-v1/candidate")
        .join(name);
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    crate::lowir::writer::write(
        program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn actual_div_header_primitive_evidence_and_cold_layout_are_separate() -> TestResult {
    let source = fixture("source-runtime")?;
    let (report, Analysis { mut facts, .. }) = analyze(&source)?;
    let actual = source_binary(&report, &mut facts, site(&report)?, Hook::Div)?;
    assert_eq!(actual.definition, named(&report, "Div", DefKind::Trait)?);
    assert_eq!(
        facts.associated.trait_owner(actual.method),
        Some(actual.definition)
    );
    assert_eq!(
        facts.associated.constant_trait(actual.definition),
        Some(true)
    );
    assert!(matches!(
        facts.types.kind(actual.rhs),
        Some(Kind::Parameter(_))
    ));
    assert!(facts.primitive_methods.is_none());
    assert!(
        report
            .resolution
            .builtin_traits
            .language(Hook::Div)
            .is_none()
    );
    assert_eq!(LangItem::Div.index(), 70);
    assert_eq!(LangItem::RangeSub.index(), 69);
    assert_eq!(size_of::<SourceBinary>(), 16);
    assert_eq!(size_of::<Operation>(), 16);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    let (bindings, imported, role) = crate::sema::compiler_items::Catalog::binding_layouts();
    println!(
        "source-div-layout Catalog={} Reservations={} Bindings={bindings} Imported={imported} Role={role} Provider={} SourceBinary={}",
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<Option<crate::resolve::bindings::Declaration>>() * LangItem::ALL.len(),
        size_of::<SourceBinary>()
    );
    let mut operators = 0;
    for (index, node) in report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        let Syntax::Binary {
            op: crate::lex::token::Punctuation::Slash,
            ..
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: NodeId::new(u32::try_from(index)?),
        };
        if let Some(fact) = facts.node(key).filter(|f| f.operation.is_some()) {
            operators += 1;
            assert!(matches!(fact.operation, Some(Operation::Binary { .. })));
            assert!(!facts.methods.calls.contains_key(&key));
        }
    }
    assert!(operators >= TYPES.len() * 2, "actual operators={operators}");
    Ok(())
}
#[test]
fn original_integer_float_methods_and_const_values_reach_native_inputs() -> TestResult {
    let source = fixture("source-runtime")?;
    let (report, analysis) = analyze(&source)?;
    let names = TYPES
        .iter()
        .flat_map(|ty| {
            [
                format!("op_{ty}"),
                format!("ufcs_{ty}"),
                format!("constant_{ty}"),
            ]
        })
        .collect::<Vec<_>>();
    let refs = names.iter().map(String::as_str).collect::<Vec<_>>();
    let program = lower(&report, analysis, &refs)?;
    for name in &names {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == *name && s.body.is_some())
                .count(),
            1,
            "actual {name}"
        );
    }
    write_program("source-runtime", &source, &program)
}
#[test]
fn explicit_override_executes_its_real_source_body_while_scalar_syntax_is_primitive() -> TestResult
{
    let source = fixture("override")?;
    let (report, analysis) = analyze(&source)?;
    let program = lower(&report, analysis, &["syntax", "explicit", "constant"])?;
    write_program("override", &source, &program)
}
#[test]
fn genuine_size_wrapper_and_provided_size_initializer_form_actual_field_bounds() -> TestResult {
    for (name, bound) in [("size-only", 8), ("field", 2)] {
        let source = fixture(name)?;
        let (report, analysis) = analyze(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(
            analysis.facts.array_lengths.values().any(|&n| n == bound),
            "{name}: real field length"
        );
        let size = named(&report, "size_of", DefKind::Function)?;
        assert!(
            report.resolution.definitions[size.0 as usize]
                .node
                .is_some()
        );
        assert!(analysis.facts.primitive_constants.is_none());
        let program = lower(&report, analysis, &["pointer_words_size"])?;
        write_program(name, &source, &program)?;
    }
    Ok(())
}
#[test]
fn missing_changed_nonconst_and_checked_faults_cannot_bypass_source_div() -> TestResult {
    for (name, expected) in [
        ("zero", "zero"),
        ("overflow", "overflow"),
        ("ordinary-ufcs", "cannot divide"),
        ("missing-role", "language"),
        ("wrong-default", ""),
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(error.contains(expected), "{name}: {error}");
        assert!(
            !error.contains("unsupported source language item"),
            "{name}: {error}"
        );
        println!("source-div-negative {name}: {error}");
    }
    let source = fixture("foundation")?;
    let (mut report, Analysis { mut facts, .. }) = analyze(&source)?;
    let key = site(&report)?;
    let actual = source_binary(&report, &mut facts, key, Hook::Div)?;
    let before = facts.body_work.canonical_steps;
    let original = facts.definitions[actual.method.0 as usize].replace(TypeId::UNIT);
    assert!(source_binary(&report, &mut facts, key, Hook::Div).is_err());
    assert!(facts.body_work.canonical_steps > before);
    facts.definitions[actual.method.0 as usize] = original;
    report.package.units[0].source = crate::support::source::SourceFile::new(source.into_bytes())?;
    assert!(source_binary(&report, &mut facts, key, Hook::Div).is_err());
    Ok(())
}
#[test]
fn div_work_is_paid_on_warm_failure_and_repetition_with_an_ordinary_control() -> TestResult {
    let (report, Analysis { mut facts, .. }) = analyze(&fixture("foundation")?)?;
    let key = site(&report)?;
    for width in [8, 32, 128] {
        let before = facts.body_work.canonical_steps;
        for _ in 0..width {
            source_binary(&report, &mut facts, key, Hook::Div)?;
        }
        let work = facts.body_work.canonical_steps - before;
        assert!(work >= width && work <= 100 * width);
        println!("source-div-work width={width} work={work}");
    }
    facts.body_work.canonical_steps = crate::support::crates::MAX_COMPONENTS;
    assert!(source_binary(&report, &mut facts, key, Hook::Div).is_err());
    assert!(facts.body_work.canonical_steps > crate::support::crates::MAX_COMPONENTS);
    let plain = "fn main(){let _=16usize/8usize;}";
    let report = crate::test_source::resolve(plain, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(analysis.facts.source_bindings.is_none());
    lower(&report, analysis, &["main"])?;
    Ok(())
}

#[test]
fn source_div_roles_cannot_be_published_through_legacy_contract_profiles() -> TestResult {
    let (report, Analysis { mut facts, .. }) = analyze(&fixture("export")?)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let error = crate::sema::crates::authored_export::Stage::begin_module_feature_predicates(
        &report, &mut facts, &identity,
    )
    .err()
    .ok_or("source Div contract exported")?;
    assert_eq!(error, "source Div contract transport is not implemented");
    Ok(())
}
