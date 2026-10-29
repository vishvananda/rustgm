//! Actual callable Copy/Clone markers retain formation but need no signature Copy.
use super::{Declarations, Frame, Predicate, Reuse, Search, Source, tests};
use crate::{
    resolve::expansion::Report,
    sema::traits::{assumptions::Assumptions, implementations::Availability, selection},
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument,
        callable::Convention, parameters::predicates::Bound,
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn declarations(tail: &str) -> TestResult<(Report, Declarations)> {
    let (header, _) = crate::provider_cases::GENERIC_COPY
        .split_once("fn twice")
        .ok_or("source header")?;
    let report = crate::test_source::source_provider(&format!("{header}{tail}"), "fn_copying")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}

fn predicate(facts: &mut Declarations, role: LangItem, ty: TypeId) -> TestResult<Predicate> {
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let declaration = catalog.language(role).ok_or("role")?;
    assert_eq!(catalog.validated_language(role)?, Some(declaration));
    Ok(tests::predicate(
        facts,
        declaration.definition,
        ty,
        GenericArgs::EMPTY,
    )?)
}

fn function(
    facts: &mut Declarations,
    ty: TypeId,
    unsafe_: bool,
    convention: Convention,
) -> TestResult<TypeId> {
    let parameters = facts.types.intern_list(&[ty])?;
    Ok(facts.types.intern(Kind::Function {
        parameters,
        result: ty,
        bound_regions: 0,
        unsafe_,
        convention,
    })?)
}

fn select(
    facts: &mut Declarations,
    goal: Predicate,
    reuse: &mut Reuse,
) -> TestResult<selection::Selection> {
    let assumptions = Assumptions::default();
    let (mut context, goal) = tests::instantiate(facts, goal)?;
    let mut caller = tests::caller(&assumptions, None);
    caller.first_region = 100;
    Ok(selection::select_reusing(
        facts,
        &mut context,
        &goal,
        caller,
        (Availability::Validated, false),
        reuse,
    )?)
}

#[test]
fn actual_callable_markers_do_not_require_copy_signature_inputs_or_results() -> TestResult {
    let (report, mut facts) = declarations("struct NoCopy(u8);fn item(x:NoCopy)->NoCopy{x}")?;
    let nominal =
        facts.definitions[tests::definition(&report, "NoCopy")?.0 as usize].ok_or("nominal")?;
    let item = facts.definitions[tests::definition(&report, "item")?.0 as usize].ok_or("item")?;
    assert!(matches!(
        facts.types.kind(item),
        Some(Kind::FunctionItem { .. })
    ));
    let ordinary = function(&mut facts, nominal, false, Convention::RUST)?;
    let foreign = function(&mut facts, nominal, true, Convention::C)?;
    for role in [LangItem::Copy, LangItem::Clone] {
        let mut reuse = Reuse::default();
        for ty in [ordinary, foreign, item, ordinary, item] {
            let goal = predicate(&mut facts, role, ty)?;
            let selection::Selection::Proven(evidence) = select(&mut facts, goal, &mut reuse)?
            else {
                return Err("missing actual callable marker".into());
            };
            assert!(matches!(evidence.source, Source::Builtin));
            assert!(evidence.proof.is_empty() && evidence.sized.is_empty());
        }
        let goal = predicate(&mut facts, role, nominal)?;
        assert!(matches!(
            select(&mut facts, goal, &mut reuse)?,
            selection::Selection::Absent
        ));
    }
    Ok(())
}

#[test]
fn pending_projection_is_not_a_callable_shape_and_invalid_inputs_still_fail() -> TestResult {
    let (report, mut facts) = declarations(
        "trait Family<X>{type Out;}fn probe<T:Family<u8>>(value:T::Out)->T::Out{value}",
    )?;
    let item = facts.definitions[tests::definition(&report, "probe")?.0 as usize]
        .ok_or("actual generic function item")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(item) else {
        return Err("actual generic function item kind".into());
    };
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("actual declared callable signature".into());
    };
    assert!(matches!(
        facts.types.kind(result),
        Some(Kind::Projection(_))
    ));
    for role in [LangItem::Copy, LangItem::Clone] {
        let requirement = predicate(&mut facts, role, result)?;
        let (mut context, goal) = tests::instantiate(&mut facts, requirement)?;
        let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
        assert_eq!(
            frame
                .inputs
                .context
                .head(&facts.types, frame.goal.subject()),
            Err(crate::types::infer::Error::Normalization),
        );
        assert!(!Search::source_builtin_marker_goal(&facts, &mut frame)?);
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        assert!(
            search
                .source_builtin_marker_result(&mut facts, &mut frame)?
                .is_none()
        );
        frame.inputs.context = crate::types::infer::Context::default();
        assert_eq!(
            Search::source_builtin_marker_goal(&facts, &mut frame),
            Err("invalid inference identity".into()),
        );
        let (mut context, goal) = tests::instantiate(&mut facts, requirement)?;
        let mut exhausted = Frame::new(&facts, &mut context, &goal, 1)?;
        let (role_item, arguments) = exhausted.goal.trait_arguments().ok_or("actual role goal")?;
        assert!(arguments.is_empty());
        let mut context = crate::types::infer::Context::default();
        let subject = context.known(&facts.types, result)?;
        exhausted.goal = crate::types::infer::predicates::Predicate::trait_application(
            subject,
            role_item,
            Vec::new(),
        );
        exhausted.inputs.context = context;
        exhausted.inputs.context.steps = 1_000_000;
        assert_eq!(
            Search::source_builtin_marker_goal(&facts, &mut exhausted),
            Err("type constraint work budget exceeded".into()),
        );
    }
    Ok(())
}

#[test]
fn callable_marker_rule_requires_actual_empty_ordinary_roles_and_keeps_precedence() -> TestResult {
    let (report, mut facts) = declarations("trait PretendCopy{}fn item(x:u8)->u8{x}")?;
    let item = facts.definitions[tests::definition(&report, "item")?.0 as usize].ok_or("item")?;
    for role in [LangItem::Copy, LangItem::Clone] {
        let ordinary = predicate(&mut facts, role, item)?;
        let (mut context, goal) = tests::instantiate(&mut facts, ordinary)?;
        let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
        assert!(Search::source_builtin_marker_goal(&facts, &mut frame)?);
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        frame.deferred = true;
        assert!(
            search
                .source_builtin_marker_result(&mut facts, &mut frame)?
                .is_none()
        );
        frame.deferred = false;
        frame.selected = Some(super::frames::Chosen::Positive(0));
        assert!(
            search
                .source_builtin_marker_result(&mut facts, &mut frame)?
                .is_none()
        );
        let Bound::Trait(app) = ordinary.bound else {
            return Err("ordinary role".into());
        };
        for mode in [Constness::Const, Constness::Conditional] {
            let changed = Predicate {
                bound: Bound::trait_application(app, mode),
                ..ordinary
            };
            let (mut context, goal) = tests::instantiate(&mut facts, changed)?;
            let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
            // This rule is unrelated; nonordinary Clone is not a language-negative claim.
            assert!(!Search::source_builtin_marker_goal(&facts, &mut frame)?);
        }
        let application = facts.types.trait_application(app).ok_or("application")?;
        let args = facts
            .types
            .intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
        let changed = tests::predicate(
            &mut facts,
            crate::resolve::model::DefId(application.item.0),
            item,
            args,
        )?;
        let (mut context, goal) = tests::instantiate(&mut facts, changed)?;
        let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
        assert_eq!(
            Search::source_builtin_marker_goal(&facts, &mut frame),
            Err("source callable copying role has unexpected arguments".into())
        );
    }
    let changed = tests::predicate(
        &mut facts,
        tests::definition(&report, "PretendCopy")?,
        item,
        GenericArgs::EMPTY,
    )?;
    let (mut context, goal) = tests::instantiate(&mut facts, changed)?;
    let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(!Search::source_builtin_marker_goal(&facts, &mut frame)?);
    Ok(())
}

#[test]
fn callable_copying_cannot_cache_an_unpaid_nominal_signature() -> TestResult {
    let (report, mut facts) = declarations("trait Need{}impl Need for u8{}struct N<T:Need>(T);")?;
    let owner = ItemId(tests::definition(&report, "N")?.0);
    let mut signatures = Vec::new();
    for primitive in [Primitive::U8, Primitive::U16] {
        let args = facts
            .types
            .intern_arguments(&[Argument::Type(TypeId::primitive(primitive))])?;
        let nominal = facts.types.intern(Kind::Adt(owner, args))?;
        signatures.push(function(&mut facts, nominal, false, Convention::RUST)?);
    }
    for role in [LangItem::Copy, LangItem::Clone] {
        let mut reuse = Reuse::default();
        for (index, paid) in [(0, true), (1, false), (1, false), (0, true)] {
            let goal = predicate(&mut facts, role, signatures[index])?;
            match select(&mut facts, goal, &mut reuse) {
                Ok(selection::Selection::Proven(evidence)) if paid => {
                    assert!(matches!(evidence.source, Source::Builtin));
                }
                Err(error) if !paid => {
                    assert!(
                        error
                            .to_string()
                            .contains("source nominal formation prerequisite is unsatisfied"),
                        "{error}"
                    );
                    assert!(reuse.selection(&facts, goal, 100)?.is_none());
                }
                _ => return Err("callable marker bypassed its nominal contract".into()),
            }
        }
        let goal = predicate(&mut facts, role, signatures[0])?;
        let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
        let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        assert_eq!(
            search
                .source_builtin_marker_result(&mut facts, &mut frame)
                .err(),
            Some("source structural rule requires completed nominal formation".into())
        );
    }
    Ok(())
}

#[test]
fn callable_marker_keeps_nominal_signature_lifetime_debts_on_repeated_queries() -> TestResult {
    let (report, mut facts) =
        declarations("trait Need{}impl<'a:'static>Need for &'a u8{}struct N<T:Need>(T);")?;
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(10),
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let args = facts.types.intern_arguments(&[Argument::Type(reference)])?;
    let nominal = facts
        .types
        .intern(Kind::Adt(ItemId(tests::definition(&report, "N")?.0), args))?;
    let pointer = function(&mut facts, nominal, false, Convention::RUST)?;
    for role in [LangItem::Copy, LangItem::Clone] {
        let goal = predicate(&mut facts, role, pointer)?;
        let mut reuse = Reuse::default();
        for _ in 0..2 {
            let selection::Selection::Proven(evidence) = select(&mut facts, goal, &mut reuse)?
            else {
                return Err("missing conditional callable proof".into());
            };
            assert!(matches!(evidence.source, Source::Builtin));
            assert!(!evidence.proof.is_empty());
            assert!(!super::size_evidence_tests::valid(&facts, &evidence.proof)?);
            assert!(reuse.selection(&facts, goal, 100)?.is_none());
        }
    }
    Ok(())
}

#[test]
fn direct_nominal_outlives_is_paid_by_lifetime_formation_after_callable_marker() -> TestResult {
    // Retain the original v4 source and raw observation. Ordinary nominal
    // Outlives clauses intentionally belong to lifetime formation, whereas
    // the test above retains the lifetime conditions of selected trait impls.
    let (report, mut facts) = declarations("struct N<T:'static>(T);")?;
    let input = RegionId(10);
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: input,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let args = facts.types.intern_arguments(&[Argument::Type(reference)])?;
    let nominal = facts
        .types
        .intern(Kind::Adt(ItemId(tests::definition(&report, "N")?.0), args))?;
    let pointer = function(&mut facts, nominal, false, Convention::RUST)?;
    for role in [LangItem::Copy, LangItem::Clone] {
        let goal = predicate(&mut facts, role, pointer)?;
        let selection::Selection::Proven(evidence) =
            select(&mut facts, goal, &mut Reuse::default())?
        else {
            return Err("missing direct nominal callable marker".into());
        };
        assert!(matches!(evidence.source, Source::Builtin));
        assert!(evidence.proof.is_empty());
    }
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    assert!(
        queries
            .requirements(pointer)?
            .contains(&(input, RegionId::STATIC))
    );
    let steps = queries.steps();
    for paid in [false, true, false] {
        let mut constraints = crate::types::relations::Constraints::new(
            &facts.types,
            &facts.lifetimes.variances,
            1,
            100_000,
        )?;
        let universal = RegionId::bound(0, 0)?;
        constraints.variable(input)?;
        constraints.outlives(input, universal, 0)?;
        constraints.outlives(universal, input, 0)?;
        if paid {
            constraints.assume(universal, RegionId::STATIC)?;
        }
        // This is the actual Obligation::WellFormed consumer in borrow::regions.
        constraints.require_requirements(
            queries.symbolic_requirements(pointer)?,
            crate::types::relations::ProofScope::ROOT,
            0,
        )?;
        let result = constraints.solve();
        match result {
            Ok(_) => assert!(paid),
            Err(
                crate::types::region_graph::Error::Unsatisfied { .. }
                | crate::types::region_graph::Error::Verification { .. },
            ) => assert!(!paid),
            Err(error) => return Err(error.into()),
        }
        assert_eq!(
            queries.steps(),
            steps,
            "warm formation must keep its cached debt"
        );
    }
    Ok(())
}

#[test]
fn callable_marker_requires_completed_source_roles_before_formation() -> TestResult {
    let (report, mut facts) = declarations("fn item(x:u8)->u8{x}")?;
    let item = facts.definitions[tests::definition(&report, "item")?.0 as usize].ok_or("item")?;
    let goal = predicate(&mut facts, LangItem::Copy, item)?;
    let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
    let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
    let completed = facts.source_bindings.take().ok_or("completed catalog")?;
    facts.source_bindings = crate::sema::compiler_items::Catalog::reserve(&report)?;
    assert_eq!(
        Search::source_builtin_marker_goal(&facts, &mut frame),
        Err("source trait role requires completed declaration validation".into())
    );
    facts.source_bindings = Some(completed);
    assert!(Search::source_builtin_marker_goal(&facts, &mut frame)?);
    Ok(())
}

#[test]
fn other_copy_subjects_do_not_allocate_callable_formation() -> TestResult {
    let (report, mut facts) = declarations("struct N;")?;
    let byte = TypeId::primitive(Primitive::U8);
    let nominal =
        facts.definitions[tests::definition(&report, "N")?.0 as usize].ok_or("nominal")?;
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: true,
        pointee: byte,
    })?;
    let reference = facts.types.intern(Kind::Reference {
        mutable: true,
        region: RegionId::STATIC,
        referent: byte,
    })?;
    for role in [LangItem::Copy, LangItem::Clone] {
        for ty in [byte, nominal, pointer, reference] {
            let goal = predicate(&mut facts, role, ty)?;
            let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
            let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
            let assumptions = Assumptions::default();
            let mut search =
                Search::new(tests::caller(&assumptions, None), Availability::Validated);
            assert!(!search.prepare_nominal_formation(&mut facts, &mut frame)?);
            assert!(frame.formation.is_none());
            assert!(search.query.families.is_empty());
        }
    }
    Ok(())
}

#[test]
fn unit_copy_and_clone_use_actual_roles_without_granting_nonempty_tuple_evidence() -> TestResult {
    let (report, mut facts) = declarations("trait PretendCopy{}struct NoCopy(u8);")?;
    let nominal =
        facts.definitions[tests::definition(&report, "NoCopy")?.0 as usize].ok_or("nominal")?;
    let list = facts.types.intern_list(&[nominal])?;
    let tuple = facts.types.intern(Kind::Tuple(list))?;
    for role in [LangItem::Copy, LangItem::Clone] {
        let mut reuse = Reuse::default();
        let goal = predicate(&mut facts, role, TypeId::UNIT)?;
        for _ in 0..2 {
            let selection::Selection::Proven(evidence) = select(&mut facts, goal, &mut reuse)?
            else {
                return Err("unit needs its real builtin marker proof".into());
            };
            assert!(matches!(evidence.source, Source::Builtin));
            assert!(evidence.proof.is_empty() && evidence.sized.is_empty());
        }
        let goal = predicate(&mut facts, role, tuple)?;
        assert!(matches!(
            select(&mut facts, goal, &mut reuse)?,
            selection::Selection::Absent
        ));
    }
    let pretend = tests::predicate(
        &mut facts,
        tests::definition(&report, "PretendCopy")?,
        TypeId::UNIT,
        GenericArgs::EMPTY,
    )?;
    assert!(matches!(
        select(&mut facts, pretend, &mut Reuse::default())?,
        selection::Selection::Absent
    ));
    Ok(())
}
