//! Negative memo entries are local proofs; recursive cutoffs and open goals are not.
use super::{Caller, Context, Decision, Declarations, Frame, Goal, Predicate, Search};
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        model::DefId,
    },
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    support::source::SourceFile,
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        infer::{Domain, Instantiation, arguments::Argument as Inferred},
        parameters::{Parameter, ParameterKind, environment::Environment, predicates::Bound},
        traits::Application,
    },
};
use std::{fmt::Write, path::Path, sync::Arc};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn declarations(source: &str) -> Result<(Report, Declarations), Box<dyn std::error::Error>> {
    let report = resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "cache.rs".into(),
            path: "cache.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn definition(report: &Report, name: &str) -> Result<DefId, Box<dyn std::error::Error>> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .ok_or("missing test declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
fn predicate(
    facts: &mut Declarations,
    item: DefId,
    ty: TypeId,
    arguments: GenericArgs,
) -> Result<Predicate, &'static str> {
    Ok(Predicate {
        ty,
        bound: Bound::Trait(facts.types.intern_trait_application(Application {
            item: ItemId(item.0),
            arguments,
        })?),
    })
}
fn instantiate(
    facts: &mut Declarations,
    predicate: Predicate,
) -> Result<(Context, Goal), crate::types::infer::Error> {
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    Ok((context, goal))
}
fn caller<'a>(
    assumptions: &'a Assumptions,
    environment: Option<&'a Arc<Environment>>,
) -> Caller<'a> {
    Caller {
        assumptions,
        environment,
        first_region: 1,

        retain_sized: false,
    }
}

#[test]
fn repeated_failed_conditions_share_one_completed_chain() -> Result<(), Box<dyn std::error::Error>>
{
    let mut source = String::from("trait A<U>{}struct N;trait B0{}impl<T:Copy>B0 for T{}");
    for i in 1..32 {
        write!(source, "trait B{i}{{}}impl<T:B{}>B{i} for T{{}}", i - 1)?;
    }
    for i in 0..128 {
        write!(source, "impl<T:B31>A<[u8;{i}]>for T{{}}")?;
    }
    source.push_str("impl<T>A<()>for T{}");
    let (report, mut facts) = declarations(&source)?;
    let subject =
        facts.definitions[definition(&report, "N")?.0 as usize].ok_or("missing subject type")?;
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = facts.types.intern(Kind::Parameter(id))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(parameter)])?;
    let predicate = predicate(&mut facts, definition(&report, "A")?, subject, arguments)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution =
        Instantiation::new(&mut context, &facts.types, &[(id, Inferred::Type(unknown))])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("failed conditions hid the unique applicable implementation".into());
    };
    context.equate_trait_predicate(&mut facts.types, &goal, search.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert_eq!(search.cycles, 0);
    assert_eq!(
        search
            .completed
            .values()
            .filter(|d| **d == Decision::Absent)
            .count(),
        32
    );
    assert!(search.steps < 30_000, "{}", search.steps);
    Ok(())
}

#[test]
fn inductive_cycle_does_not_publish_negative_answers() -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = declarations(
        "trait A{}trait B{}trait C{}impl<T:B>A for T{}impl<T:C>B for T{}impl<T:B>C for T{}",
    )?;
    let predicate = predicate(
        &mut facts,
        definition(&report, "A")?,
        TypeId::primitive(Primitive::U8),
        GenericArgs::EMPTY,
    )?;
    let (mut context, goal) = instantiate(&mut facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(search.run(&mut facts, first)?, Decision::Absent);
    assert_eq!(search.cycles, 1);
    assert!(search.completed.is_empty());
    assert!(search.active.is_empty());
    Ok(())
}

#[test]
fn missing_proof_cannot_cross_caller_assumptions_or_query_modes()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = declarations("trait A{}trait B{}impl<T:B>A for T{}")?;
    let ty = TypeId::primitive(Primitive::U8);
    let root = predicate(
        &mut facts,
        definition(&report, "A")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let bound = predicate(
        &mut facts,
        definition(&report, "B")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let empty = Assumptions::default();
    let mut supplied = Assumptions::default();
    supplied.insert(&facts.types, bound)?;
    for (assumptions, accepted) in [(&empty, false), (&supplied, true), (&empty, false)] {
        for availability in [Availability::Declarations, Availability::Validated] {
            let (mut context, goal) = instantiate(&mut facts, root)?;
            let mut search = Search::new(caller(assumptions, None), availability);
            let first = Frame::new(&facts, &mut context, &goal, 1)?;
            assert_eq!(
                matches!(search.run(&mut facts, first)?, Decision::Proven(_)),
                accepted
            );
        }
    }
    Ok(())
}

#[test]
fn builtin_permissions_belong_to_the_same_immutable_scope() -> Result<(), Box<dyn std::error::Error>>
{
    use crate::types::parameters::environment::Abilities;
    let (report, mut facts) = declarations("trait A{}impl<T:Copy>A for T{}")?;
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(id))?;
    let root = predicate(
        &mut facts,
        definition(&report, "A")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let empty = Assumptions::default();
    for copy in [false, true, false] {
        let environment = Arc::new(Environment::new(
            &facts.types,
            &[(id, Abilities { copy, clone_: copy })],
        )?);
        let (mut context, goal) = instantiate(&mut facts, root)?;
        let mut search = Search::new(caller(&empty, Some(&environment)), Availability::Validated);
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        assert_eq!(
            matches!(search.run(&mut facts, first)?, Decision::Proven(_)),
            copy
        );
    }
    Ok(())
}

#[test]
fn provisional_headers_never_authorize_validated_queries() -> Result<(), Box<dyn std::error::Error>>
{
    use crate::sema::traits::implementations::{Implementation, Implementations};
    let (report, mut facts) = declarations("trait A{}impl A for u8{}")?;
    let header = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing impl")?;
    let (owner, source, root) = (header.owner, header.source, header.predicate);
    let item = ItemId(definition(&report, "A")?.0);
    let mut registry = Implementations::default();
    registry.prepare(
        &mut facts.types,
        item,
        Implementation {
            owner,
            source,
            predicate: root,
            methods: Vec::new(),
            types: Vec::new(),
        },
    )?;
    facts.generics.implementations = Some(Box::new(registry));
    let assumptions = Assumptions::default();
    for (availability, accepted) in [
        (Availability::Validated, false),
        (Availability::Declarations, true),
        (Availability::Validated, false),
    ] {
        let (mut context, goal) = instantiate(&mut facts, root)?;
        let mut search = Search::new(caller(&assumptions, None), availability);
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        assert_eq!(
            matches!(search.run(&mut facts, first)?, Decision::Proven(_)),
            accepted
        );
    }
    facts
        .generics
        .implementations
        .as_mut()
        .ok_or("missing registry")?
        .complete(owner, Vec::new(), Vec::new())?;
    let (mut context, goal) = instantiate(&mut facts, root)?;
    let result = super::super::select(&mut facts, &mut context, &goal, caller(&assumptions, None))?;
    assert!(matches!(result, super::super::Selection::Proven(_)));
    Ok(())
}

#[test]
fn exhausted_work_does_not_become_a_negative_result() -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = declarations("trait A{}")?;
    // Ordinary absence with an empty registry still passes through the worklist.
    facts.generics.implementations = Some(Box::default());
    let root = predicate(
        &mut facts,
        definition(&report, "A")?,
        TypeId::primitive(Primitive::U8),
        GenericArgs::EMPTY,
    )?;
    let (mut context, goal) = instantiate(&mut facts, root)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
    search.steps = 1_000_000;
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(
        search.run(&mut facts, first),
        Err("trait selection work budget exceeded".into())
    );
    assert!(search.completed.is_empty());
    Ok(())
}

#[test]
fn repeated_unsatisfied_equalities_share_a_scoped_negative_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("trait P{type Out;}impl P for u8{type Out=u16;}trait A<U>{}");
    for i in 0..64 {
        write!(source, "impl<T:P<Out=u32>>A<[u8;{i}]>for T{{}}")?;
    }
    source.push_str("impl<T>A<()>for T{}");
    let (report, mut facts) = declarations(&source)?;
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = facts.types.intern(Kind::Parameter(id))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(parameter)])?;
    let query = predicate(
        &mut facts,
        definition(&report, "A")?,
        TypeId::primitive(Primitive::U8),
        arguments,
    )?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution =
        Instantiation::new(&mut context, &facts.types, &[(id, Inferred::Type(unknown))])?;
    let goal = context.instantiate_predicate(&mut facts.types, query, &mut substitution)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("unsatisfied equalities hid the applicable implementation".into());
    };
    context.equate_proved_predicate(&mut facts.types, &goal, search.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert!(search.reused);
    assert_eq!(
        search
            .completed
            .iter()
            .filter(|(p, result)| matches!(p.bound, Bound::ProjectionEqual(_))
                && **result == Decision::Absent)
            .count(),
        1
    );
    assert_eq!(search.cycles, 0);
    assert!(search.steps < 20_000, "{}", search.steps);
    Ok(())
}

#[test]
fn repeated_projected_trait_conditions_share_a_negative_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source =
        String::from("trait P{type Out;}impl P for u8{type Out=u16;}trait C{}trait A<U>{}");
    for i in 0..64 {
        write!(
            source,
            "impl<T:P>A<[u8;{i}]>for T where <T as P>::Out:C{{}}"
        )?;
    }
    source.push_str("impl<T>A<()>for T{}");
    let (report, mut facts) = declarations(&source)?;
    let id = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = facts.types.intern(Kind::Parameter(id))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(parameter)])?;
    let query = predicate(
        &mut facts,
        definition(&report, "A")?,
        TypeId::primitive(Primitive::U8),
        arguments,
    )?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution =
        Instantiation::new(&mut context, &facts.types, &[(id, Inferred::Type(unknown))])?;
    let goal = context.instantiate_predicate(&mut facts.types, query, &mut substitution)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("unsatisfied projected conditions hid the applicable implementation".into());
    };
    context.equate_proved_predicate(&mut facts.types, &goal, search.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert!(search.reused);
    assert_eq!(
        search
            .completed
            .iter()
            .filter(|(p, result)| matches!(p.bound, Bound::Trait(_))
                && facts.types.contains_projections(p.ty) == Some(true)
                && **result == Decision::Absent)
            .count(),
        1
    );
    assert_eq!(search.cycles, 0);
    assert!(search.steps < 20_000, "{}", search.steps);
    Ok(())
}

#[test]
fn cached_result_rules_never_replace_the_callers_parent_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = declarations("trait C{}trait A{type Out:C;}")?;
    let projection = facts
        .associated
        .type_contract(definition(&report, "Out")?)
        .ok_or("missing result contract")?
        .projection;
    let Some(Kind::Projection(application)) = facts.types.kind(projection) else {
        return Err("missing projected test subject".into());
    };
    let parent = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let query = predicate(
        &mut facts,
        definition(&report, "C")?,
        projection,
        GenericArgs::EMPTY,
    )?;
    // Inspecting a declaration template without any caller is intentionally possible.
    // Reusing that immutable mapping can never stand in for the parent obligation.
    let item = ItemId(definition(&report, "C")?.0);
    let (first, _) = crate::sema::traits::results::traits(&mut facts, projection, item)?
        .ok_or("missing result rules")?;
    let (second, steps) = crate::sema::traits::results::traits(&mut facts, projection, item)?
        .ok_or("missing cached rules")?;
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(steps, 1);
    for permitted in [true, false, true] {
        let mut assumptions = Assumptions::default();
        if permitted {
            assumptions.insert(&facts.types, parent)?;
        }
        let (mut context, goal) = instantiate(&mut facts, query)?;
        let result =
            super::super::select(&mut facts, &mut context, &goal, caller(&assumptions, None))?;
        assert_eq!(
            matches!(result, super::super::Selection::Proven(_)),
            permitted
        );
    }
    Ok(())
}
