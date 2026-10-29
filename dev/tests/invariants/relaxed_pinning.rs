//! `UnsafeUnpin` keeps actual source identity, caller debt and structural work.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    support::bindings::LangItem,
    types::{
        ItemId, Kind, RegionId, arguments::Argument, auto_traits::AutoTrait,
        parameters::predicates::Bound,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-unsafe-unpin/{name}.rs"
    ))?)
}
fn declarations(source: &str) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let report = crate::test_source::source_provider(source, "unsafe_unpin")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
#[test]
fn actual_and_renamed_roles_preserve_header_identity_and_complete_caller_bodies() -> Result {
    for name in ["positive", "renamed"] {
        let report = crate::test_source::source_provider(&fixture(name)?, "actual_pinning")?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        let declaration = facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .validated_language(LangItem::UnsafeUnpin)?
            .ok_or("actual role")?;
        let spelling = report
            .package
            .names
            .get(report.resolution.definitions[declaration.definition.0 as usize].name);
        assert_eq!(
            spelling,
            if name == "renamed" {
                "ActualPinningProperty"
            } else {
                "UnsafeUnpin"
            }
        );
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
        );
    }
    Ok(())
}
#[test]
fn source_negative_impls_exceptions_headers_and_formation_never_gain_name_authority() -> Result {
    for name in [
        "negative-payload",
        "negative-zero-array",
        "generic-unpaid",
        "missing-pointer-impl",
        "unsafe-impl-required",
        "header-safe",
        "header-no-auto",
        "header-generic",
        "header-member",
        "lookalike",
        "formation-unpaid",
    ] {
        let error = crate::test_source::source_provider(&fixture(name)?, "pinning_opponent")
            .and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            })
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("budget"), "{name}: {error}");
        eprintln!("UnsafeUnpin opponent {name}: {error}");
    }
    Ok(())
}
#[test]
fn actual_formation_work_is_linear_and_internal_marker_cannot_replace_the_trait() -> Result {
    let source = fixture("nominal-query")?;
    let mut previous = None;
    for depth in [8, 32, 128] {
        let (report, mut facts) = declarations(&source)?;
        let mut ty = facts.definitions[tests::definition(&report, "N")?.0 as usize].ok_or("N")?;
        let owner = ItemId(tests::definition(&report, "W")?.0);
        for _ in 0..depth {
            let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
            ty = facts.types.intern(Kind::Adt(owner, arguments))?;
        }
        let bound = freezing::bound(&mut facts, AutoTrait::UnsafeUnpin)?;
        let (mut context, goal) = tests::instantiate(&mut facts, Predicate { ty, bound })?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let Decision::Proven(root) = search.run(&mut facts, frame)? else {
            return Err("actual structural proof".into());
        };
        let work = search.query.steps;
        let edges: usize = search
            .query
            .nodes
            .iter()
            .map(|node| node.children.len())
            .sum();
        assert!(search.query.nodes[root].children.len() >= 2);
        if let Some((old_work, old_edges)) = previous {
            assert!(work <= old_work * 6);
            assert!(edges <= old_edges * 6);
        }
        previous = Some((work, edges));
        eprintln!("UnsafeUnpin depth={depth} work={work} edges={edges}");
        search.evidence(&facts, root)?;
        let (mut context, goal) = tests::instantiate(&mut facts, Predicate { ty, bound })?;
        let mut rejected = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        rejected.query.steps = 1_000_000;
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        assert!(rejected.run(&mut facts, frame).is_err());
        assert!(rejected.query.steps > 1_000_000);
    }
    let (_, mut facts) = declarations(&source)?;
    let (mut context, goal) = tests::instantiate(
        &mut facts,
        Predicate {
            ty: crate::types::TypeId::UNIT,
            bound: Bound::Auto(AutoTrait::UnsafeUnpin),
        },
    )?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(search.run(&mut facts, frame).is_err());
    assert_eq!(LangItem::UnsafeUnpin.index(), 66);
    for (index, role) in LangItem::ALL[..66].iter().enumerate() {
        assert_eq!(role.index(), index);
    }
    assert_eq!(AutoTrait::ALL.len(), 5);
    assert!(!AutoTrait::ALL.contains(&AutoTrait::UnsafeUnpin));
    eprintln!(
        "UnsafeUnpin Catalog={} Reservations={} Node={} Body={}",
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::body::ir::Body>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn negative_capture_and_coinductive_siblings_keep_their_real_proof_debts() -> Result {
    let source = fixture("nominal-query")?
        + "struct Captured<'a>(&'a u8);impl<'a> !UnsafeUnpin for Captured<'a>{}";
    let (report, mut facts) = declarations(&source)?;
    let args = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "Captured")?.0),
        args,
    ))?;
    let predicate = Predicate {
        ty,
        bound: freezing::bound(&mut facts, AutoTrait::UnsafeUnpin)?,
    };
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Refuted(root) = search.run(&mut facts, frame)? else {
        return Err("negative capture".into());
    };
    assert!(!search.query.nodes[root].debt_free);
    assert!(search.negative_child(Decision::Refuted(root)).is_err());
    let super::super::Refutation::Conditional(proof) =
        search.refutation(&facts, root, predicate)?
    else {
        return Err("lost captured debt".into());
    };
    assert!(!proof.proof.equalities.is_empty());
    assert!(
        super::super::Refutation::Conditional(proof)
            .reject()
            .is_err()
    );
    cycles()
}
fn cycles() -> Result {
    let source=fixture("nominal-query")?.replace("#[unstable(feature=\"unsafe_unpin\",issue=\"125735\")]\nunsafe impl<T:PointeeSized> UnsafeUnpin for *const T{}\n","")+"struct Ring{next:*const Ring}struct BadRing{next:*const BadRing,bad:UnsafePinned<u8>}";
    for (name, expected) in [("Ring", true), ("BadRing", false)] {
        let (report, mut facts) = declarations(&source)?;
        let ty =
            facts.definitions[tests::definition(&report, name)?.0 as usize].ok_or("ring type")?;
        let bound = freezing::bound(&mut facts, AutoTrait::UnsafeUnpin)?;
        let (mut context, goal) = tests::instantiate(&mut facts, Predicate { ty, bound })?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let decision = search.run(&mut facts, frame)?;
        assert_eq!(matches!(decision, Decision::Proven(_)), expected, "{name}");
        assert!(search.query.cycles > 0);
        assert!(search.query.active.is_empty());
    }
    Ok(())
}
