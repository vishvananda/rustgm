//! Actual resolved aliases exercise canonical slots, equality clauses and shared dependencies.
#[path = "transparent_binders.rs"]
mod binders;
#[path = "transparent_naming.rs"]
mod naming;
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
    types::{
        ItemId, Primitive, RegionId,
        arguments::{Argument, Constant},
        parameters::{clauses::Clause, predicates::Bound},
    },
};
use std::{fmt::Write, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> TestResult<Report> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "contracts.rs".into(),
            path: "contracts.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    Ok(resolve::analyze_expanded(report)?)
}
fn headers(report: &Report) -> TestResult<Checker<'_>> {
    let mut checker = Checker::new(report)?;
    checker.prepare_trait_owners(&mut 0, 1_000_000)?;
    Ok(checker)
}
fn definition(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == kind && report.package.names.get(definition.name) == name
        })
        .ok_or("missing actual source declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
fn edge(
    checker: &Checker<'_>,
    name: &str,
    subject: TypeId,
    arguments: GenericArgs,
) -> TestResult<Edge> {
    let target = definition(checker.report, name, DefKind::TraitAlias)?;
    let header = checker
        .trait_aliases
        .as_ref()
        .ok_or("actual aliases")?
        .header(target)?;
    Ok(Edge {
        source: header.owner.source,
        target,
        subject,
        original_subject: subject,
        arguments,
        quantified: None,
    })
}
#[test]
fn alias_headers_retain_full_formals_equality_and_distinct_unsized_self() -> TestResult {
    let source = "#![feature(trait_alias)]trait P<'a,T,const N:usize>{type Out;}trait A<'unused,'a,U,const M:usize>=P<'a,U,M,Out=u8> where Self:P<'a,U,M>,U:Copy;trait B<'b,V,const K:usize>=A<'static,'b,V,K>;fn main(){}";
    let report = report(source)?;
    let checker = headers(&report)?;
    let a = definition(&report, "A", DefKind::TraitAlias)?;
    let pending = checker.trait_aliases.as_ref().ok_or("actual alias table")?;
    assert!(pending.closed);
    let a_header = pending.header(a)?;
    let schema = a_header.schema.as_ref().ok_or("actual alias schema")?;
    assert_eq!(schema.captures, 2);
    let slots = &checker.facts.generics.parameters[&a];
    assert_eq!(slots.len(), 4);
    assert!(matches!(slots[0].argument, Argument::Lifetime(_)));
    assert!(matches!(slots[1].argument, Argument::Lifetime(_)));
    assert!(matches!(slots[2].argument, Argument::Type(_)));
    assert!(matches!(slots[3].argument, Argument::Const(_)));
    let crate::types::Kind::Parameter(id) = checker
        .facts
        .types
        .kind(schema.self_type)
        .ok_or("actual Self type")?
    else {
        return Err("Self is not rigid".into());
    };
    let self_parameter = checker
        .facts
        .types
        .parameter(id)
        .ok_or("actual Self parameter")?;
    assert_eq!(
        self_parameter.owner,
        ItemId(a_header.owner.self_definition.0)
    );
    assert_eq!(
        self_parameter.kind,
        crate::types::parameters::ParameterKind::Type { sized: false }
    );
    assert!(checker.facts.associated.owner(a).is_none());
    assert!(
        checker
            .facts
            .associated
            .owner(a_header.owner.self_definition)
            .is_none()
    );
    assert!(!checker.facts.generics.trait_self_bounds.contains_key(&a));
    let clauses = a_header
        .clauses
        .as_deref()
        .ok_or("complete canonical clauses")?;
    assert!(clauses.iter().any(|clause| matches!(clause.value, Clause::Predicate(p) if matches!(p.bound, Bound::ProjectionEqual(_)))));
    let b = pending.header(definition(&report, "B", DefKind::TraitAlias)?)?;
    assert_eq!(b.edges.len(), 1);
    assert_eq!(b.edges[0].target, a);
    assert!(b.clauses.as_ref().is_some_and(Vec::is_empty));
    assert!(
        checker.reject_alias_execution().is_err(),
        "partial headers cannot publish executable facts"
    );
    Ok(())
}
#[test]
fn alias_applications_substitute_all_slots_and_preserve_real_leaf_equalities() -> TestResult {
    let report = report(
        "#![feature(trait_alias)]trait P<'a,T,const N:usize>{type Out;}trait A<'unused,'a,U,const M:usize>=P<'a,U,M,Out=u8> where Self:P<'a,U,M>,U:Copy;trait B<'b,V,const K:usize>=A<'static,'b,V,K>;fn main(){}",
    )?;
    let mut checker = headers(&report)?;
    let ty = TypeId::primitive(Primitive::U8);
    let n = checker.facts.types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let arguments = checker.facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::STATIC),
        Argument::Type(ty),
        Argument::Const(n),
    ])?;
    let requested = edge(&checker, "B", TypeId::primitive(Primitive::U32), arguments)?;
    let clauses = checker.expand_alias_edge(requested)?;
    assert_eq!(clauses.len(), 3);
    let p = definition(&report, "P", DefKind::Trait)?;
    let actual = clauses
        .iter()
        .find_map(|clause| match clause.value {
            Clause::Predicate(predicate) => predicate
                .bound
                .trait_parts()
                .map(|(id, _)| (predicate.ty, id)),
            Clause::ForAll(_) => None,
        })
        .ok_or("actual leaf application")?;
    assert_eq!(actual.0, requested.subject);
    let application = checker
        .facts
        .types
        .trait_application(actual.1)
        .ok_or("actual leaf trait")?;
    assert_eq!(application.item, ItemId(p.0));
    assert_eq!(
        checker
            .facts
            .types
            .arguments(application.arguments)
            .ok_or("leaf arguments")?
            .collect::<Vec<_>>(),
        vec![
            Argument::Lifetime(RegionId::STATIC),
            Argument::Type(ty),
            Argument::Const(n)
        ]
    );
    assert!(clauses.iter().any(|clause| matches!(clause.value, Clause::Predicate(predicate) if predicate.ty == ty && predicate.bound == Bound::Copy)));
    assert!(
        checker
            .expand_alias_edge(Edge {
                arguments: GenericArgs::EMPTY,
                ..requested
            })
            .is_err()
    );
    distinct_applications_keep_both_argument_environments()?;
    Ok(())
}

fn distinct_applications_keep_both_argument_environments() -> TestResult {
    let report = report(
        "#![feature(trait_alias)]trait P<T>{}trait A<T>=P<T>;trait Both=A<u8>+A<u16>;fn main(){}",
    )?;
    let mut checker = headers(&report)?;
    let request = edge(
        &checker,
        "Both",
        TypeId::primitive(Primitive::U32),
        GenericArgs::EMPTY,
    )?;
    let clauses = checker.expand_alias_edge(request)?;
    assert_eq!(
        clauses.len(),
        2,
        "different actual arguments cannot share an application key"
    );
    let mut arguments = crate::support::hash::HashSet::default();
    for clause in clauses {
        let predicate = clause.value.ordinary()?;
        let (id, _) = predicate
            .bound
            .trait_parts()
            .ok_or("actual leaf trait application")?;
        let application = checker
            .facts
            .types
            .trait_application(id)
            .ok_or("actual leaf")?;
        let values = checker
            .facts
            .types
            .arguments(application.arguments)
            .ok_or("actual leaf arguments")?
            .collect::<Vec<_>>();
        let [Argument::Type(ty)] = values.as_slice() else {
            return Err("wrong leaf environment".into());
        };
        arguments.insert(*ty);
    }
    assert_eq!(
        arguments,
        [
            TypeId::primitive(Primitive::U8),
            TypeId::primitive(Primitive::U16)
        ]
        .into_iter()
        .collect()
    );
    Ok(())
}
#[test]
fn alias_bound_worklist_publishes_only_real_trait_and_equality_clauses() -> TestResult {
    let report = report(
        "#![feature(trait_alias)]trait P<T>{type Out;}trait A<T>=P<T,Out=u8> where Self:P<T>;fn consumer<U:A<u16>>(){}fn main(){}",
    )?;
    let mut checker = headers(&report)?;
    let consumer = definition(&report, "consumer", DefKind::Function)?;
    checker
        .work
        .push(super::super::Work::Definition(consumer, false));
    checker.drain(&mut 0, 1_000_000)?;
    let clauses = checker
        .facts
        .generics
        .predicates
        .get(&consumer)
        .ok_or("actual consumer bounds")?;
    assert_eq!(clauses.len(), 2);
    let real_trait = definition(&report, "P", DefKind::Trait)?;
    let mut applications = 0;
    let mut equalities = 0;
    for clause in clauses {
        let predicate = clause.value.ordinary()?;
        match predicate.bound {
            Bound::Trait(id) => {
                let application = checker
                    .facts
                    .types
                    .trait_application(id)
                    .ok_or("actual expanded trait")?;
                assert_eq!(application.item, ItemId(real_trait.0));
                assert_eq!(
                    checker
                        .facts
                        .types
                        .arguments(application.arguments)
                        .ok_or("expanded real arguments")?
                        .collect::<Vec<_>>(),
                    vec![Argument::Type(TypeId::primitive(Primitive::U16))]
                );
                applications += 1;
            }
            Bound::ProjectionEqual(result) => {
                assert_eq!(result, TypeId::primitive(Primitive::U8));
                equalities += 1;
            }
            _ => return Err("unexpected invented alias premise".into()),
        }
    }
    assert_eq!((applications, equalities), (1, 1));
    Ok(())
}
#[test]
fn alias_cycles_arity_and_unfinished_binder_consumers_fail_without_empty_success() -> TestResult {
    for source in [
        "trait A=B;trait B=A;",
        "trait P<T>{}trait A=P;",
        "trait P{}trait A=P<u8>;",
    ] {
        let report = report(&format!("#![feature(trait_alias)]{source}fn main(){{}}"))?;
        assert!(
            headers(&report).is_err(),
            "invalid unused alias header succeeded: {source}"
        );
    }
    failed_graph_cannot_restart_as_an_empty_contract()?;
    let report = report(
        "#![feature(trait_alias)]trait P<'a>{}trait A=for<'a>P<'a>;trait B=for<'b>A;trait Empty=;fn main(){}",
    )?;
    let mut checker = headers(&report)?;
    let a = edge(
        &checker,
        "A",
        TypeId::primitive(Primitive::U8),
        GenericArgs::EMPTY,
    )?;
    assert!(matches!(
        checker.expand_alias_edge(a)?.as_slice(),
        [Predicate {
            value: Clause::ForAll(_),
            ..
        }]
    ));
    let b = edge(&checker, "B", a.subject, GenericArgs::EMPTY)?;
    let clauses = checker.expand_alias_edge(b)?;
    let [
        Predicate {
            value: Clause::ForAll(id),
            ..
        },
    ] = clauses.as_slice()
    else {
        return Err("nested alias lost its complete quantifier".into());
    };
    let quantified = checker
        .facts
        .types
        .quantified(*id)
        .ok_or("actual composed alias")?;
    assert_eq!(
        quantified.bound_regions, 2,
        "unused outer lifetime remains a declared slot"
    );
    let empty = edge(&checker, "Empty", a.subject, GenericArgs::EMPTY)?;
    assert!(checker.expand_alias_edge(empty)?.is_empty());
    Ok(())
}

fn failed_graph_cannot_restart_as_an_empty_contract() -> TestResult {
    let report = report("#![feature(trait_alias)]trait A=B;trait B=A;fn main(){}")?;
    let mut checker = Checker::new(&report)?;
    let error = checker
        .prepare_trait_owners(&mut 0, 1_000_000)
        .err()
        .ok_or("cycle was accepted")?;
    assert!(error.message.contains("cycle in trait alias"));
    assert!(
        !checker
            .trait_aliases
            .as_ref()
            .ok_or("retained failed owner")?
            .closed
    );
    let request = edge(
        &checker,
        "A",
        TypeId::primitive(Primitive::U8),
        GenericArgs::EMPTY,
    )?;
    assert!(checker.expand_alias_edge(request).is_err());
    let error = checker
        .complete_alias_headers(&mut 0, 1_000_000)
        .err()
        .ok_or("failed canonical graph restarted")?;
    assert!(error.message.contains("cannot restart"));
    Ok(())
}
#[test]
fn alias_diamonds_keep_linear_canonical_work_and_ordinary_storage_stays_empty() -> TestResult {
    let ordinary = report("trait P{}fn main(){}")?;
    assert!(headers(&ordinary)?.trait_aliases.is_none());
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(trait_alias)]trait P{}trait A0=P;");
        for index in 1..=width {
            write!(source, "trait A{index}=A{}+A{};", index - 1, index - 1)?;
        }
        source.push_str("fn main(){}");
        let report = report(&source)?;
        let mut checker = headers(&report)?;
        let request = edge(
            &checker,
            &format!("A{width}"),
            TypeId::primitive(Primitive::U8),
            GenericArgs::EMPTY,
        )?;
        assert_eq!(checker.expand_alias_edge(request)?.len(), 1);
        let table = checker.trait_aliases.as_ref().ok_or("actual diamond")?;
        assert_eq!(table.headers.len(), width + 1);
        assert_eq!(
            table
                .headers
                .iter()
                .map(|header| header.edges.len())
                .sum::<usize>(),
            width * 2
        );
        let work = table.budget.steps.get();
        assert!(
            work < (width + 1) * 250,
            "shared applications were flattened repeatedly"
        );
        observations.push((width, work, table.budget.bytes.get()));
    }
    println!(
        "alias graph widths/work/logical cumulative bytes: {observations:?}; Checker {} Pending {} Header {} Edge {}",
        size_of::<Checker<'_>>(),
        size_of::<Pending>(),
        size_of::<Header>(),
        size_of::<Edge>()
    );
    Ok(())
}
