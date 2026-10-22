//! Source binders preserve the identities consumed by scoped proof queries.
use super::{TestResult, rustc};
use crate::semantic;
use rustgm::{
    sema::model::Declarations,
    types::{
        Kind, RegionId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::Bound,
        },
    },
};

fn predicates(name: &str, source: &str) -> TestResult<(Declarations, Vec<Clause>)> {
    rustc::qualify(&format!("quantified-lexical-{name}"), source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let owner = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| report.package.names.get(definition.name) == "f")
        .ok_or("missing function f")?
        .0;
    let clauses = facts
        .generics
        .predicates
        .get(&rustgm::resolve::model::DefId(u32::try_from(owner)?))
        .ok_or("missing function predicates")?
        .iter()
        .map(|p| p.value)
        .collect();
    Ok((facts, clauses))
}
fn quantified(facts: &Declarations, clause: Clause) -> TestResult<Quantified> {
    let Clause::ForAll(id) = clause else {
        return Err("lost written quantifier".into());
    };
    Ok(facts
        .types
        .quantified(id)
        .ok_or("missing quantified body")?)
}
fn arguments(facts: &Declarations, clause: Quantified) -> TestResult<Vec<Argument>> {
    let Bound::Trait(id) = clause.predicate.bound else {
        return Err("missing trait application".into());
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("missing application")?;
    Ok(facts
        .types
        .arguments(application.arguments)
        .ok_or("missing arguments")?
        .collect())
}
#[test]
fn alpha_renaming_shares_identity_but_unused_slots_remain() -> TestResult {
    let (facts, clauses) = predicates(
        "alpha",
        "trait A<'a>{}fn f<T>()where for<'a>T:A<'a>,for<'renamed>T:A<'renamed>,for<'unused,'a>T:A<'a>{}fn main(){}",
    )?;
    assert_eq!(clauses.len(), 3);
    assert_eq!(clauses[0], clauses[1]);
    assert_ne!(clauses[0], clauses[2]);
    let unused = quantified(&facts, clauses[2])?;
    assert_eq!(unused.bound_regions, 2);
    assert_eq!(
        arguments(&facts, unused)?,
        vec![Argument::Lifetime(RegionId::bound(0, 1)?)]
    );
    Ok(())
}
#[test]
fn nested_callable_keeps_its_local_and_both_enclosing_captures() -> TestResult {
    let (facts, clauses) = predicates(
        "nested",
        "trait A<X>{}fn f<'x,T>(x:&'x T)where for<'a>T:A<for<'b>fn(&'a u8,&'b u8,&'x u8)>{}fn main(){}",
    )?;
    let value = quantified(&facts, clauses[0])?;
    assert_eq!(value.bound_regions, 1);
    let [Argument::Type(ty)] = arguments(&facts, value)?[..] else {
        return Err("missing callable argument".into());
    };
    let Some(Kind::Function {
        parameters,
        bound_regions,
        ..
    }) = facts.types.kind(ty)
    else {
        return Err("missing nested function".into());
    };
    assert_eq!(bound_regions, 1);
    let mut regions = Vec::new();
    for &ty in facts
        .types
        .list(parameters)
        .ok_or("missing callable parameters")?
    {
        let Some(Kind::Reference { region, .. }) = facts.types.kind(ty) else {
            return Err("missing reference".into());
        };
        regions.push(region.bound_parts());
    }
    assert_eq!(regions, [Some((1, 0)), Some((0, 0)), Some((2, 0))]);
    Ok(())
}
#[test]
fn trait_position_captures_subject_while_where_position_binds_it() -> TestResult {
    let (facts, clauses) = predicates(
        "subject",
        "trait A<'a>{}fn f<'x,T>(x:&'x T)where &'x T:for<'a>A<'a>,for<'b>&'b T:A<'b>{}fn main(){}",
    )?;
    for (clause, depth) in clauses.into_iter().zip([1, 0]) {
        let value = quantified(&facts, clause)?;
        let Some(Kind::Reference { region, .. }) = facts.types.kind(value.predicate.ty) else {
            return Err("missing bound reference subject".into());
        };
        assert_eq!(region.bound_parts(), Some((depth, 0)));
        assert_eq!(
            arguments(&facts, value)?,
            vec![Argument::Lifetime(RegionId::bound(0, 0)?)]
        );
    }
    Ok(())
}
#[test]
fn empty_written_quantifiers_survive_parsing_and_canonicalization() -> TestResult {
    let source = "fn f<'x,T>(x:&'x T)where for<>T:Copy,&'x T:for<>Copy,T:Clone{}fn main(){}";
    let ast = rustgm::syntax::expression::parse_crate(source).map_err(|e| format!("{e:?}"))?;
    let count = ast.nodes.iter().filter(|node| matches!(node.kind,
        rustgm::syntax::ast::Kind::Where { quantified: true, binders, .. } |
        rustgm::syntax::ast::Kind::Bound { quantified: true, binders, .. } if binders.start == binders.end)).count();
    assert_eq!(count, 2);
    assert!(std::mem::size_of::<rustgm::syntax::ast::Node>() <= 36);
    let (facts, clauses) = predicates("empty", source)?;
    assert_eq!(quantified(&facts, clauses[0])?.bound_regions, 0);
    let second = quantified(&facts, clauses[1])?;
    assert_eq!(second.bound_regions, 0);
    let Some(Kind::Reference { region, .. }) = facts.types.kind(second.predicate.ty) else {
        return Err("missing captured reference".into());
    };
    assert_eq!(region.bound_parts(), Some((1, 0)));
    assert!(matches!(clauses[2], Clause::Predicate(_)));
    Ok(())
}

#[test]
fn associated_result_contract_distinguishes_its_local_from_the_owner_capture() -> TestResult {
    let (_, source, _) = super::results::CASES
        .iter()
        .find(|case| case.0 == "result-captured-owner")
        .ok_or("missing independently qualified result source")?;
    let (report, facts) = semantic::analyze(source)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "Item")
        .ok_or("missing associated declaration")?;
    let contract = facts
        .associated
        .type_contract(rustgm::resolve::model::DefId(u32::try_from(index)?))
        .ok_or("missing result contract")?;
    let mut traits = 0;
    let mut equalities = 0;
    for &clause in &contract.bounds {
        if !matches!(clause, Clause::ForAll(_)) {
            continue;
        }
        let value = quantified(&facts, clause)?;
        assert_eq!(value.bound_regions, 1);
        let parent = match value.predicate.bound {
            Bound::Trait(_) => {
                traits += 1;
                assert_eq!(
                    arguments(&facts, value)?,
                    [Argument::Lifetime(RegionId::bound(0, 0)?)]
                );
                value.predicate.ty
            }
            Bound::ProjectionEqual(result) => {
                equalities += 1;
                let Some(Kind::Tuple(fields)) = facts.types.kind(result) else {
                    return Err("missing quantified result tuple".into());
                };
                let mut regions = Vec::new();
                for &field in facts.types.list(fields).ok_or("missing result fields")? {
                    let Some(Kind::Reference { region, .. }) = facts.types.kind(field) else {
                        return Err("missing result reference".into());
                    };
                    regions.push(region.bound_parts());
                }
                assert_eq!(regions, [Some((0, 0)), Some((1, 0))]);
                let Some(Kind::Projection(equality)) = facts.types.kind(value.predicate.ty) else {
                    return Err("missing result equality subject".into());
                };
                equality.subject
            }
            _ => return Err("unexpected quantified result requirement".into()),
        };
        let Some(Kind::Projection(parent)) = facts.types.kind(parent) else {
            return Err("missing captured result parent".into());
        };
        let application = facts
            .types
            .trait_application(parent.trait_application)
            .ok_or("missing captured parent application")?;
        assert_eq!(
            facts
                .types
                .arguments(application.arguments)
                .ok_or("missing parent arguments")?
                .collect::<Vec<_>>(),
            [Argument::Lifetime(RegionId::bound(1, 0)?)]
        );
    }
    assert_eq!((traits, equalities), (1, 1));
    Ok(())
}
