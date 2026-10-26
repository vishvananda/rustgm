//! Real Not headers and ordinary caller proof precede primitive emission.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    resolve::unit::NodeKey,
    sema::model::Operation,
    syntax::ast::{Kind as Syntax, NodeId, Unary},
    types::Kind,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const NOT: &str = "#[lang=\"not\"]pub trait Inverter{type Output;fn not(self)->Self::Output;}";
const BOOL: &str = "impl Inverter for bool{type Output=bool;fn not(self)->bool{!self}}";
fn source(tail: &str) -> Result<String> {
    let actual = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let (foundation, _) = actual
        .split_once("pub fn identity<T>")
        .ok_or("actual prerequisite foundation")?;
    Ok(format!(
        "{foundation}\nimpl Clone for bool{{fn clone(&self)->Self{{*self}}}}impl Copy for bool{{}}\n{tail}"
    ))
}
fn analyze(tail: &str) -> Result<(Report, Analysis)> {
    let report = crate::test_source::source_provider(&source(tail)?, "source_unary")?;
    let checked = pipeline::analyze_source(&report)?;
    Ok((report, checked))
}
fn exported(report: &Report, facts: &Declarations, name: &str) -> Result<DefId> {
    Ok(facts
        .functions
        .iter()
        .find(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            report.package.names.get(definition.name) == name
        })
        .ok_or("actual selected function")?
        .definition)
}

#[test]
fn actual_source_not_uses_ordinary_impl_proof_and_erases_only_primitive_dispatch() -> Result {
    let tail = format!(
        "{NOT}{BOOL}pub struct Flag(bool);impl Inverter for Flag{{type Output=bool;fn not(self)->bool{{!self.0}}}}pub fn overloaded()->bool{{!Flag(false)}}pub fn scalar(x:bool)->bool{{!x}}"
    );
    let (
        report,
        Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(&tail)?;
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
            op: Unary::Not,
            operand,
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: NodeId::new(u32::try_from(index)?),
        };
        let operand = facts
            .node(NodeKey {
                node: operand,
                ..key
            })
            .and_then(|f| f.ty)
            .ok_or("actual operand type")?;
        if matches!(facts.types.kind(operand), Some(Kind::Primitive(_))) {
            primitive += 1;
            assert!(!facts.methods.calls.contains_key(&key));
            assert!(matches!(
                facts.node(key).and_then(|n| n.operation),
                Some(Operation::Unary {
                    operator: Unary::Not,
                    ..
                })
            ));
        } else {
            overloaded += 1;
            assert!(facts.methods.calls.contains_key(&key));
        }
    }
    assert_eq!((primitive, overloaded), (3, 1));
    let actual = source_not(
        &report,
        &mut facts,
        NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: report.package.units[0].configured.ast.root,
        },
    )?;
    assert!(facts.associated.operation(actual.definition).is_none());
    assert!(
        report
            .resolution
            .builtin_traits
            .language(crate::support::language_hooks::Hook::Not)
            .is_none()
    );
    let roots = [
        exported(&report, &facts, "overloaded")?,
        exported(&report, &facts, "scalar")?,
    ];
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 3
    );
    assert_eq!(size_of::<crate::sema::compiler_items::Catalog>(), 704);
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2056);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<SourceNot>(), 12);
    Ok(())
}

#[test]
fn source_not_rejects_missing_fake_malformed_and_stale_role_authority() -> Result {
    for tail in [
        "pub fn probe(x:bool)->bool{!x}".to_owned(),
        format!(
            "{}{BOOL}pub fn probe(x:bool)->bool{{!x}}",
            NOT.replace("#[lang=\"not\"]", "")
        ),
        format!("{NOT}pub fn probe(x:bool)->bool{{!x}}"),
        format!(
            "{}{BOOL}pub fn probe(x:bool)->bool{{!x}}",
            NOT.replace("fn not(self)", "fn not(&self)")
        ),
        format!(
            "{}impl Inverter for bool{{type Output=bool;fn not(self)->bool{{false}}}}pub fn probe(x:bool)->bool{{!x}}",
            NOT.replace("->Self::Output", "->bool")
        ),
        format!(
            "{NOT}pub struct Flag;impl Inverter for Flag{{type Output=u8;fn not(self)->u8{{7}}}}pub fn probe(x:Flag)->u8{{if !x{{1}}else{{0}}}}"
        ),
    ] {
        assert!(analyze(&tail).is_err(), "tail={tail}");
    }
    let tail = format!("{NOT}{BOOL}");
    let (mut report, Analysis { mut facts, .. }) = analyze(&tail)?;
    let site = NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: report.package.units[0].configured.ast.root,
    };
    let actual = source_not(&report, &mut facts, site)?;
    let baseline = facts.body_work.canonical_steps;
    source_not(&report, &mut facts, site)?;
    assert!(facts.body_work.canonical_steps > baseline);
    let clone = crate::test_source::source_provider(&source(&tail)?, "source_unary")?;
    assert!(source_not(&clone, &mut facts, site).is_err());
    let index = *facts
        .function_indices
        .get(&actual.method)
        .ok_or("actual method")?;
    facts.functions[index].signature = TypeId::UNIT;
    assert!(source_not(&report, &mut facts, site).is_err());
    report.package.units[0].source =
        crate::support::source::SourceFile::new(source(&tail)?.into_bytes())?;
    assert!(source_not(&report, &mut facts, site).is_err());
    let (report, Analysis { mut facts, .. }) = analyze(&tail)?;
    facts.body_work.canonical_steps = crate::support::crates::MAX_COMPONENTS;
    assert!(source_not(&report, &mut facts, site).is_err());
    assert!(facts.body_work.canonical_steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}

#[test]
fn primitive_source_not_preserves_const_effects_and_real_constant_evaluation() -> Result {
    let ordinary = format!(
        "{NOT}impl Inverter for bool{{type Output=bool;fn not(self)->bool{{false}}}}pub const VALUE:bool=!false;pub fn value()->bool{{VALUE}}"
    );
    let (report, checked) = analyze(&ordinary)?;
    true_constant(&report, checked)?;
    let constant = NOT.replace("pub trait", "pub const trait");
    let prelude = "#![feature(const_trait_impl)]";
    let destruct = "#[lang=\"destruct\"]pub const trait Destruct:PointeeSized{}";
    let rejected = format!(
        "{prelude}{}{destruct}{constant}{BOOL}pub const VALUE:bool=!false;",
        source("")?
    );
    let report = crate::test_source::source_provider(&rejected, "source_unary")?;
    assert!(pipeline::analyze_source(&report).is_err());
    let accepted = (rejected + "pub fn value()->bool{VALUE}")
        .replace("impl Inverter for bool", "const impl Inverter for bool");
    let report = crate::test_source::source_provider(&accepted, "source_unary")?;
    let checked = pipeline::analyze_source(&report)?;
    true_constant(&report, checked)?;
    assert!(!crate::expand::builtin::Builtin::Assert.executable());
    Ok(())
}

fn true_constant(
    report: &Report,
    Analysis {
        mut facts,
        prepared,
    }: Analysis,
) -> Result {
    use crate::lowir::program::{Literal, Operand, Terminator};
    let roots = [exported(report, &facts, "value")?];
    let program = crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    let mut bodies = program
        .symbols
        .iter()
        .filter_map(|symbol| symbol.body.as_ref());
    let body = bodies.next().ok_or("actual constant consumer body")?;
    assert!(bodies.next().is_none());
    assert!(body.blocks.iter().any(|block| match block.terminator {
        Terminator::Return(Some(value)) => match value.operand {
            Operand::Literal(id) =>
                matches!(body.literals.get(id.0 as usize), Some(Literal::Integer(1))),
            _ => false,
        },
        _ => false,
    }));
    Ok(())
}
