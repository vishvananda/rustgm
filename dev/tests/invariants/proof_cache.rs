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
        parameters::{
            Parameter, ParameterKind,
            environment::{Abilities, Environment},
            predicates::Bound,
        },
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
pub(super) fn declarations(
    source: &str,
) -> Result<(Report, Declarations), Box<dyn std::error::Error>> {
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
pub(super) fn definition(report: &Report, name: &str) -> Result<DefId, Box<dyn std::error::Error>> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .ok_or("missing test declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
pub(super) fn predicate(
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
pub(super) fn instantiate(
    facts: &mut Declarations,
    predicate: Predicate,
) -> Result<(Context, Goal), crate::types::infer::Error> {
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    Ok((context, goal))
}
pub(super) fn caller<'a>(
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
    context.equate_trait_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert_eq!(search.query.cycles, 0);
    assert_eq!(
        search
            .query
            .completed
            .values()
            .filter(|d| **d == Decision::Absent)
            .count(),
        32
    );
    assert!(search.query.steps < 30_000, "{}", search.query.steps);
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
    assert_eq!(search.query.cycles, 1);
    assert!(search.query.completed.is_empty());
    assert!(search.query.active.is_empty());
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
            constants: Vec::new(),
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
        .complete(owner, Vec::new(), Vec::new(), Vec::new())?;
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
    search.query.steps = 1_000_000;
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(
        search.run(&mut facts, first),
        Err("trait selection work budget exceeded".into())
    );
    assert!(search.query.completed.is_empty());
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
    context.equate_proved_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert!(search.query.reused);
    assert_eq!(
        search
            .query
            .completed
            .iter()
            .filter(|(p, result)| matches!(p.bound, Bound::ProjectionEqual(_))
                && **result == Decision::Absent)
            .count(),
        1
    );
    assert_eq!(search.query.cycles, 0);
    assert!(search.query.steps < 20_000, "{}", search.query.steps);
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
    context.equate_proved_predicate(&mut facts.types, &goal, search.query.nodes[root].goal)?;
    assert_eq!(context.resolved(&mut facts.types, unknown)?, TypeId::UNIT);
    assert!(search.query.reused);
    assert_eq!(
        search
            .query
            .completed
            .iter()
            .filter(|(p, result)| matches!(p.bound, Bound::Trait(_))
                && facts.types.contains_projections(p.ty) == Some(true)
                && **result == Decision::Absent)
            .count(),
        1
    );
    assert_eq!(search.query.cycles, 0);
    assert!(search.query.steps < 20_000, "{}", search.query.steps);
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

#[test]
fn recursive_result_inputs_reuse_their_completed_parent_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = declarations("trait A where Self::Out:A{type Out;}")?;
    let projection = facts
        .associated
        .type_contract(definition(&report, "Out")?)
        .ok_or("missing recursive result contract")?
        .projection;
    let Some(Kind::Projection(mut application)) = facts.types.kind(projection) else {
        return Err("missing recursive projection".into());
    };
    let parent = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let mut assumptions = Assumptions::default();
    assumptions.insert(&facts.types, parent)?;
    for depth in [4, 8, 16, 32, 64, 128] {
        application.subject = parent.ty;
        for _ in 0..depth {
            application.subject = facts.types.intern(Kind::Projection(application))?;
        }
        let query = Predicate {
            ty: application.subject,
            bound: parent.bound,
        };
        let (mut context, goal) = instantiate(&mut facts, query)?;
        let mut search = Search::new(caller(&assumptions, None), Availability::Validated);
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        let Decision::Proven(root) = search.run(&mut facts, first)? else {
            return Err("finite recursive result chain was not proved".into());
        };
        assert_eq!(search.query.nodes[root].goal, query);
        assert_eq!(search.query.cycles, 0);
        // Each distinct application contributes bounded selection and proof
        // work; previously completed prefixes remain shared canonical inputs.
        assert!(
            search.query.steps < 512 * (depth + 1),
            "depth {depth}: {} steps",
            search.query.steps
        );
        assert!(search.query.nodes.len() < 5 * (depth + 1));
        let edges: usize = search
            .query
            .nodes
            .iter()
            .map(|node| node.children.len())
            .sum();
        assert!(
            edges < 8 * (depth + 1),
            "depth {depth}: {edges} proof edges"
        );
        let evidence = search.evidence(root)?;
        assert!(evidence.proof.regions.is_empty());
        assert!(evidence.proof.equalities.is_empty());
    }
    Ok(())
}

#[test]
fn cached_normalized_result_keeps_its_inputs_lifetime_equations()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::RegionId;
    let (report, mut facts) = declarations("trait A<'a>{type Out:A<'a>;}")?;
    let projection = facts
        .associated
        .type_contract(definition(&report, "Out")?)
        .ok_or("missing result contract")?
        .projection;
    let Some(Kind::Projection(mut application)) = facts.types.kind(projection) else {
        return Err("missing result projection".into());
    };
    let item = ItemId(definition(&report, "A")?.0);
    let mut assumptions = Assumptions::default();
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId(1))])?;
    let trait_application = facts
        .types
        .intern_trait_application(Application { item, arguments })?;
    assumptions.insert(
        &facts.types,
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(trait_application),
        },
    )?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId(2))])?;
    application.trait_application = facts
        .types
        .intern_trait_application(Application { item, arguments })?;
    for _ in 0..4 {
        application.subject = facts.types.intern(Kind::Projection(application))?;
    }
    let query = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let (mut context, goal) = instantiate(&mut facts, query)?;
    let mut caller = caller(&assumptions, None);
    caller.first_region = 3;
    let mut search = Search::new(caller, Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 3)?;
    assert!(matches!(
        search.run(&mut facts, first)?,
        Decision::Proven(_)
    ));
    let cached = search
        .query
        .completed
        .iter()
        .find_map(|(predicate, decision)| {
            if facts.types.contains_projections(predicate.ty) == Some(true)
                && let Decision::Proven(index) = decision
            {
                Some(*index)
            } else {
                None
            }
        })
        .ok_or("missing reusable derived result")?;
    // This child must remain valid even if its original candidate is discarded.
    let proof = search.evidence(cached)?;
    assert!(proof.proof.regions.is_empty());
    assert!(proof.proof.equalities.iter().any(
        |&(a, b)| (a, b) == (RegionId(1), RegionId(2)) || (a, b) == (RegionId(2), RegionId(1))
    ));
    Ok(())
}

#[test]
fn normalized_inputs_keep_fresh_impl_debts_out_of_the_reusable_cache()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::RegionId;
    let (report, mut facts) = declarations(
        "trait P<'a>{type Value:A<'a>;}trait A<'a>{type Next:A<'a>;}impl<'a,'b:'a,T:A<'a>>P<'a>for &'b T{type Value=T;}",
    )?;
    let next = facts
        .associated
        .type_contract(definition(&report, "Next")?)
        .ok_or("missing recursive result")?
        .projection;
    let Some(Kind::Projection(mut application)) = facts.types.kind(next) else {
        return Err("missing recursive projection".into());
    };
    let parameter = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    application.subject = facts.types.intern(Kind::Parameter(parameter))?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId(2))])?;
    let item = ItemId(definition(&report, "A")?.0);
    application.trait_application = facts
        .types
        .intern_trait_application(Application { item, arguments })?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(
        &facts.types,
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        },
    )?;
    let subject = facts.types.intern(Kind::Reference {
        region: RegionId(1),
        mutable: false,
        referent: application.subject,
    })?;
    let item = ItemId(definition(&report, "P")?.0);
    let trait_application = facts
        .types
        .intern_trait_application(Application { item, arguments })?;
    application.subject =
        facts
            .types
            .intern(Kind::Projection(crate::types::projection::Application {
                item: ItemId(definition(&report, "Value")?.0),
                subject,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))?;
    for _ in 0..4 {
        application.subject = facts.types.intern(Kind::Projection(application))?;
    }
    let query = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let (mut context, goal) = instantiate(&mut facts, query)?;
    let mut caller = caller(&assumptions, None);
    caller.first_region = 3;
    let mut search = Search::new(caller, Availability::Validated);
    let first = Frame::new(&facts, &mut context, &goal, 3)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("missing normalized result proof".into());
    };
    assert!(
        search
            .query
            .completed
            .iter()
            .all(|(predicate, _)| facts.types.contains_projections(predicate.ty) == Some(false))
    );
    let proof = search.evidence(root)?;
    assert!(proof.proof.regions.end > 3);
    assert!(!proof.proof.outlives.is_empty());
    assert!(!proof.proof.equalities.is_empty());
    Ok(())
}

fn checked_outer_request(
    facts: &mut Declarations,
    application: crate::types::projection::Application,
) -> Result<crate::types::substitute::Request, &'static str> {
    let outer = facts.types.intern(Kind::Projection(application))?;
    let mut fold = crate::types::substitute::Normalizer::default();
    let crate::types::substitute::Step::Resolve(request) =
        fold.start_with_input(&mut facts.types, outer, application.subject)?
    else {
        return Err("missing checked outer projection request");
    };
    Ok(request)
}

#[test]
fn exact_normalized_lifetime_premises_preserve_one_shared_canonical_graph()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::sema::nominals::traits::normal_form::{self, Progress};
    let (report, mut facts) = declarations("trait A<'a>{type Out:A<'a>;}")?;
    let inner = facts
        .associated
        .type_contract(definition(&report, "Out")?)
        .ok_or("missing recursive contract")?
        .projection;
    let Some(Kind::Projection(mut application)) = facts.types.kind(inner) else {
        return Err("missing result projection".into());
    };
    let before = facts.types.len();
    application.subject = inner;
    for _ in 0..512 {
        let request = checked_outer_request(&mut facts, application)?;
        let Progress::Ready(projected) = normal_form::begin(
            &mut facts,
            None,
            request,
            1,
            false,
            Availability::Declarations,
        )?
        else {
            return Err("an exact opaque premise waited for an implementation".into());
        };
        assert!(matches!(
            projected.source,
            Some(super::Source::ResultBound(_))
        ));
        assert_eq!(
            projected.resolution,
            Some(crate::types::substitute::Resolution::Opaque)
        );
        assert!(projected.proof.is_empty());
        application.subject = facts.types.intern(Kind::Projection(application))?;
    }
    assert!(
        facts.types.len() - before < 8 * 512,
        "{} new canonical types",
        facts.types.len() - before
    );
    Ok(())
}

#[test]
fn canonical_premises_preserve_caller_alternatives_and_mapped_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{RegionId, substitute::Bound as Substitution};
    let (report, mut facts) = declarations("trait A<'a>{type Out:A<'a>;}")?;
    let inner = facts
        .associated
        .type_contract(definition(&report, "Out")?)
        .ok_or("missing recursive contract")?
        .projection;
    let Some(Kind::Projection(mut application)) = facts.types.kind(inner) else {
        return Err("missing result projection".into());
    };
    let Some(Kind::Parameter(parameter)) = facts.types.kind(application.subject) else {
        return Err("missing trait Self parameter".into());
    };
    application.subject = inner;
    let request = checked_outer_request(&mut facts, application)?;
    let goal = Predicate {
        ty: inner,
        bound: Bound::Trait(application.trait_application),
    };
    let environment = Environment::new(&facts.types, &[(parameter, Abilities::default())])?
        .with_predicates(&facts.types, &[goal])?;
    assert!(
        matches!(super::super::opaque_normalized(&mut facts, Some(&environment), request)?, Some(super::Source::Assumption(value)) if value == goal)
    );
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 1)?)])?;
    let different = predicate(&mut facts, definition(&report, "A")?, inner, arguments)?;
    let competing = environment
        .clone()
        .with_predicates(&facts.types, &[different])?;
    assert!(super::super::opaque_normalized(&mut facts, Some(&competing), request)?.is_none());
    let mut different_application = application;
    let Bound::Trait(trait_application) = different.bound else {
        return Err("missing trait".into());
    };
    different_application.trait_application = trait_application;
    let different_request = checked_outer_request(&mut facts, different_application)?;
    assert!(super::super::opaque_normalized(&mut facts, None, different_request)?.is_none());
    let mut fold = Substitution::new(vec![RegionId(7), RegionId(8)]);
    let mapped = environment.substitute_regions(&mut facts.types, &mut fold)?;
    let mapped_goal = fold.predicate(&mut facts.types, goal)?;
    let outer = facts.types.intern(Kind::Projection(application))?;
    let outer = fold.apply(&mut facts.types, outer)?;
    let Some(Kind::Projection(mapped_application)) = facts.types.kind(outer) else {
        return Err("missing mapped projection".into());
    };
    let mapped_request = checked_outer_request(&mut facts, mapped_application)?;
    assert!(
        matches!(super::super::opaque_normalized(&mut facts, Some(&mapped), mapped_request)?, Some(super::Source::Assumption(value)) if value == mapped_goal)
    );
    let competing = competing.substitute_regions(&mut facts.types, &mut fold)?;
    assert!(
        super::super::opaque_normalized(&mut facts, Some(&competing), mapped_request)?.is_none()
    );
    Ok(())
}

#[test]
fn exact_premises_cannot_hide_caller_or_result_equality_rules()
-> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "trait A<'a>{type Out:A<'a>;}",
        "trait A<'a>{type Out:A<'a,Out=u8>;}impl<'a>A<'a>for u8{type Out=u8;}",
    ] {
        let (report, mut facts) = declarations(source)?;
        let inner = facts
            .associated
            .type_contract(definition(&report, "Out")?)
            .ok_or("missing recursive contract")?
            .projection;
        let Some(Kind::Projection(mut application)) = facts.types.kind(inner) else {
            return Err("missing result projection".into());
        };
        let Some(Kind::Parameter(parameter)) = facts.types.kind(application.subject) else {
            return Err("missing trait Self parameter".into());
        };
        application.subject = inner;
        let request = checked_outer_request(&mut facts, application)?;
        if source.contains("Out=u8") {
            assert!(super::super::opaque_normalized(&mut facts, None, request)?.is_none());
        }
        let outer = facts.types.intern(Kind::Projection(application))?;
        let equality = Predicate {
            ty: outer,
            bound: Bound::ProjectionEqual(TypeId::primitive(Primitive::U8)),
        };
        let environment = Environment::new(&facts.types, &[(parameter, Abilities::default())])?
            .with_predicates(&facts.types, &[equality])?;
        assert!(
            super::super::opaque_normalized(&mut facts, Some(&environment), request)?.is_none()
        );
    }
    Ok(())
}
