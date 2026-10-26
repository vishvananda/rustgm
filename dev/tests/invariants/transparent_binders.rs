//! Real alias clauses retain surrounding slots without copying a growing binder path.
use super::*;
use crate::types::{Kind as Type, parameters::clauses::Quantified};

fn quantified(checker: &Checker<'_>, clause: Clause) -> TestResult<Quantified> {
    let Clause::ForAll(id) = clause else { return Err("actual alias clause lost its quantifier".into()) };
    Ok(checker.facts.types.quantified(id).ok_or("actual quantified clause")?)
}
fn reference(checker: &Checker<'_>, ty: TypeId, depth: u16, slot: u16) -> TestResult {
    let Type::Reference { region, referent, mutable: false } = checker.facts.types.kind(ty).ok_or("actual alias reference")? else { return Err("alias reference changed kind".into()) };
    assert_eq!(region, RegionId::bound(depth, slot)?);
    assert_eq!(referent, TypeId::primitive(Primitive::U16));
    Ok(())
}
fn check_leaf(checker: &Checker<'_>, clauses: &[Predicate], count: u32, first: u16) -> TestResult {
    assert_eq!(clauses.len(), 2);
    for clause in clauses {
        let value = quantified(checker, clause.value)?;
        assert_eq!(value.bound_regions, count);
        match value.predicate.bound {
            Bound::Trait(id) => {
                let application = checker.facts.types.trait_application(id).ok_or("actual alias leaf application")?;
                let values = checker.facts.types.arguments(application.arguments).ok_or("real leaf argument list")?.collect::<Vec<_>>();
                let [Argument::Lifetime(x), Argument::Lifetime(y), Argument::Type(function)] = values.as_slice() else { return Err("leaf lost complete arguments".into()) };
                assert_eq!(*x, RegionId::bound(0, first)?);
                assert_eq!(*y, RegionId::bound(0, first + 1)?);
                let Type::Function { parameters, result: TypeId::UNIT, bound_regions: 0, .. } = checker.facts.types.kind(*function).ok_or("real nested callable")? else { return Err("alias callable changed kind".into()) };
                let parameters = checker.facts.types.list(parameters).ok_or("nested callable parameters")?;
                assert_eq!(parameters.len(), 2);
                reference(checker, parameters[0], 1, first)?;
                reference(checker, parameters[1], 1, first + 1)?;
            }
            Bound::ProjectionEqual(result) => reference(checker, result, 0, first)?,
            _ => return Err("alias introduced an unrelated predicate".into()),
        }
    }
    Ok(())
}
#[test]
fn nested_alias_binders_keep_unused_slots_equalities_and_callable_capture_depth() -> TestResult {
    let report = report("#![feature(trait_alias)]trait P<'a,'b,T>{type Out;}trait A<'x,T>=for<'y>P<'x,'y,fn(&'x T,&'y T),Out=&'x T>;trait B<T>=for<'z>A<'z,T>;trait C<T>=for<'unused>B<T>;fn consumer<U:for<'s>C<u16>>(){}fn main(){}")?;
    let mut checker = headers(&report)?;
    let arguments = checker.facts.types.intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U16))])?;
    let request = edge(&checker, "C", TypeId::primitive(Primitive::U8), arguments)?;
    let clauses = checker.expand_alias_edge(request)?;
    check_leaf(&checker, &clauses, 3, 1)?;
    let consumer = definition(&report, "consumer", DefKind::Function)?;
    checker.work.push(super::super::super::Work::Definition(consumer, false));
    checker.drain(&mut 0, 1_000_000)?;
    let clauses = checker.facts.generics.predicates.get(&consumer).ok_or("actual quantified bound consumer")?;
    check_leaf(&checker, clauses, 4, 2)?;
    Ok(())
}
#[test]
fn identical_alias_inputs_in_distinct_surrounding_binders_remain_distinct() -> TestResult {
    let report = report("#![feature(trait_alias)]trait P{}trait A=P;trait Both=A+for<'unused>A;fn main(){}")?;
    let mut checker = headers(&report)?;
    let request = edge(&checker, "Both", TypeId::primitive(Primitive::U8), GenericArgs::EMPTY)?;
    let clauses = checker.expand_alias_edge(request)?;
    assert_eq!(clauses.len(), 2, "surrounding binder is part of a canonical application");
    assert_eq!(clauses.iter().filter(|clause| matches!(clause.value, Clause::Predicate(_))).count(), 1);
    let bound = clauses.iter().find(|clause| matches!(clause.value, Clause::ForAll(_))).ok_or("retained unused scope")?;
    assert_eq!(quantified(&checker, bound.value)?.bound_regions, 1);
    Ok(())
}
#[test]
fn quantified_shared_alias_graphs_compose_each_surrounding_scope_once() -> TestResult {
    let mut observations = Vec::<(usize, usize, usize)>::new();
    for width in [8_usize, 32, 128] {
        let mut source = String::from("#![feature(trait_alias)]trait P<'a>{}trait A0=for<'last>P<'last>;");
        for index in 1..=width { write!(source, "trait A{index}=for<'x>A{}+for<'y>A{};", index - 1, index - 1)?; }
        source.push_str("fn main(){}");
        let report = report(&source)?;
        let mut checker = headers(&report)?;
        let request = edge(&checker, &format!("A{width}"), TypeId::primitive(Primitive::U8), GenericArgs::EMPTY)?;
        let clauses = checker.expand_alias_edge(request)?;
        assert_eq!(clauses.len(), 1);
        assert_eq!(quantified(&checker, clauses[0].value)?.bound_regions, u32::try_from(width + 1)?);
        let table = checker.trait_aliases.as_ref().ok_or("actual shared quantified graph")?;
        let work = table.budget.steps.get();
        let bytes = table.budget.bytes.get();
        if let Some(&(old_width, old_work, old_bytes)) = observations.last() {
            let ratio = width / old_width;
            assert!(work <= old_work * (ratio + 2) + 256, "binder work grew faster than canonical scopes");
            assert!(bytes <= old_bytes * (ratio + 2) + 4096, "binder retention copied ancestor prefixes");
        }
        observations.push((width, work, bytes));
    }
    println!("quantified alias width/work/logical bytes {observations:?}");
    Ok(())
}
