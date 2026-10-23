//! Retained operand syntax is data; it cannot authorize a runtime value promotion.
use crate::{
    body,
    body::ir::{BinaryOperands, Rvalue},
    lex::token::Punctuation,
    sema, test_source,
};

fn build(
    source: &str,
) -> Result<
    (
        crate::resolve::expansion::Report,
        sema::model::Declarations,
        body::ir::Program,
    ),
    String,
> {
    let report = test_source::resolve(source, true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    Ok((report, facts, program))
}

#[test]
fn division_source_facts_preserve_group_block_call_and_negative_literal_distinctions()
-> Result<(), String> {
    for (source, expected) in [
        ("fn f()->u8{7u8/((2))}", BinaryOperands::Both),
        ("fn f()->u8{7u8/{2}}", BinaryOperands::Left),
        ("fn g()->u8{2}fn f()->u8{7u8/g()}", BinaryOperands::Left),
        ("fn f(n:u8)->u8{7u8/n}", BinaryOperands::Both),
        ("fn f()->i8{7i8/-1}", BinaryOperands::Both),
        ("fn f()->i8{-128i8/-1}", BinaryOperands::Both),
        ("fn f()->i8{7i8/-(1+1)}", BinaryOperands::Left),
        (
            "macro_rules! id{($e:expr)=>{$e}}fn f()->u8{7u8/id!(2)}",
            BinaryOperands::Both,
        ),
    ] {
        let (_, facts, program) = build(source)?;
        let actual = program
            .bodies
            .iter()
            .flat_map(|body| &body.values)
            .find_map(|value| {
                if let Rvalue::Binary {
                    operator: Punctuation::Slash,
                    operands,
                    ..
                } = value.kind
                {
                    Some(operands)
                } else {
                    None
                }
            })
            .ok_or("missing division")?;
        assert_eq!(actual, expected, "{source}");
        assert!(facts.body_work.cfg_operand_visits.get() >= 2);
    }
    assert_eq!(size_of::<Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    Ok(())
}

#[test]
fn immediate_fact_is_not_promotion_permission_and_operation_syntax_is_no_longer_needed()
-> Result<(), String> {
    for (source, promoted) in [
        ("fn f()->&'static u8{&(7u8/2)}", true),
        ("fn f()->&'static u8{&(7u8/{2})}", false),
        ("fn f(n:u8)->&'static u8{&(7u8/n)}", false),
        ("fn f()->&'static i8{&(-128i8/-1)}", false),
    ] {
        let (mut report, mut facts, mut program) = build(source)?;
        let function = program.bodies.first().ok_or("missing function")?;
        let division = function
            .statements
            .iter()
            .find_map(|statement| {
                let body::ir::Action::Evaluate(value) = statement.action else {
                    return None;
                };
                matches!(
                    function.values[value.0 as usize].kind,
                    Rvalue::Binary {
                        operator: Punctuation::Slash,
                        ..
                    }
                )
                .then_some(statement.source)
            })
            .ok_or("missing division statement")?;
        // Only the irrelevant original operation syntax is replaced; the typed
        // operands, values, scopes, obligations and source identity remain exact.
        let node = &mut report.package.units[function.source.file.0 as usize]
            .configured
            .ast
            .nodes[division.get() as usize];
        node.kind = crate::syntax::ast::Kind::Name(node.span, None);
        body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
        let contracts = body::regions::collect(&facts).map_err(str::to_owned)?;
        let result = crate::borrow::regions::check_bodies(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        );
        assert_eq!(result.is_ok(), promoted, "{source}");
    }
    Ok(())
}

#[test]
fn operand_construction_visits_are_linear_and_irrelevant_binaries_do_not_walk_syntax()
-> Result<(), String> {
    for count in [8, 32, 128] {
        let divisions = "let _=7u8/((2));".repeat(count);
        let source = format!("fn f(){{{divisions}}}");
        let (_, facts, _) = build(&source)?;
        assert_eq!(facts.body_work.cfg_operand_visits.get(), count * 4);
        let additions = "let _=7u8+((2));".repeat(count);
        let (_, facts, _) = build(&format!("fn f(){{{additions}}}"))?;
        assert_eq!(facts.body_work.cfg_operand_visits.get(), 0);
    }
    Ok(())
}
