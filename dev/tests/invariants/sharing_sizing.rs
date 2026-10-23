//! Only complete actual source-sizing proofs share across an immutable caller.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability, selection},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, RegionId, TypeId,
        infer::{Context, Instantiation},
        parameters::predicates::Bound,
        traits::Application,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn facts() -> Result<(Declarations, TypeId)> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    let report = crate::test_source::source_provider(&source, "sharing_sizing")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "N")
        .ok_or("N")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let ty = facts.definitions[index].ok_or("N type")?;
    Ok((facts, ty))
}
fn predicate(facts: &mut Declarations, kind: LangItem, ty: TypeId) -> Result<Predicate> {
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let role = catalog.language(kind).ok_or("role")?;
    if !matches!(kind, LangItem::MetaSized | LangItem::PointeeSized) {
        assert_eq!(catalog.validated_language(kind)?, Some(role));
    }
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    let goal = Predicate {
        ty,
        bound: Bound::Trait(application),
    };
    if matches!(kind, LangItem::MetaSized | LangItem::PointeeSized) {
        assert!(crate::sema::compiler_items::sizing_predicate(facts, goal)?);
    }
    Ok(goal)
}
fn select(
    facts: &mut Declarations,
    predicate: Predicate,
    reuse: &mut Reuse,
) -> Result<selection::Selection> {
    let mut context = Context::default();
    let mut fold = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut fold)?;
    let assumptions = Assumptions::default();
    Ok(selection::select_reusing(
        facts,
        &mut context,
        &goal,
        selection::Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 1,
            retain_sized: false,
        },
        (Availability::Validated, false),
        reuse,
    )?)
}
#[test]
fn source_sizing_reuse_requires_the_exact_current_validated_catalog() -> Result {
    let (mut facts, ty) = facts()?;
    let goal = predicate(&mut facts, LangItem::MetaSized, ty)?;
    let mut reuse = Reuse::default();
    let selection::Selection::Proven(evidence) = select(&mut facts, goal, &mut reuse)? else {
        return Err("source sizing".into());
    };
    assert!(evidence.proof.is_empty() && evidence.sized.is_empty());
    assert!(reuse.selection(&facts, goal, 1)?.is_some());
    assert!(!reuse.completed.is_empty());
    let mut changed = goal;
    changed.ty = TypeId::UNIT;
    assert!(reuse.selection(&facts, changed, 1)?.is_none());
    let Bound::Trait(id) = goal.bound else {
        return Err("trait".into());
    };
    changed = Predicate {
        bound: Bound::ConstTrait(id),
        ..goal
    };
    assert!(reuse.selection(&facts, changed, 1)?.is_none());
    let catalog = facts.source_bindings.take().ok_or("catalog")?;
    assert!(reuse.selection(&facts, goal, 1).is_err());
    assert!(select(&mut facts, goal, &mut reuse).is_err());
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    let report = crate::test_source::source_provider(&source, "sharing_sizing")?;
    facts.source_bindings = crate::sema::compiler_items::Catalog::reserve(&report)?;
    assert!(facts.source_bindings.is_some());
    assert!(reuse.selection(&facts, goal, 1).is_err());
    assert!(select(&mut facts, goal, &mut reuse).is_err());
    facts.source_bindings = Some(shifted_catalog()?);
    assert!(reuse.selection(&facts, goal, 1).is_err());
    assert!(select(&mut facts, goal, &mut reuse).is_err());
    facts.source_bindings = Some(catalog);
    assert!(reuse.selection(&facts, goal, 1)?.is_some());
    assert!(matches!(
        select(&mut facts, goal, &mut reuse)?,
        selection::Selection::Proven(_)
    ));
    Ok(())
}
fn shifted_catalog() -> Result<Box<crate::sema::compiler_items::Catalog>> {
    let mut source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    let before = source.find("#[lang=").ok_or("first role")?;
    source.insert_str(before, "struct Before0;struct Before1;struct Before2;\n");
    let report = crate::test_source::source_provider(&source, "shifted_sizing")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    facts
        .source_bindings
        .take()
        .ok_or_else(|| "shifted catalog".into())
}
fn node(goal: Predicate, children: Vec<usize>) -> Node {
    Node {
        debt_free: false,
        cycle_dependent: false,
        provisional_size: false,
        goal,
        source: Some(Source::Builtin),
        children,
        first: 1,
        end: 1,
        equalities: Vec::new(),
        outlives: Vec::new(),
        type_outlives: Vec::new(),
        sized: Vec::new(),
    }
}
#[test]
fn sizing_sharing_preserves_each_debt_and_scope() -> Result {
    let (mut facts, ty) = facts()?;
    let goal = predicate(&mut facts, LangItem::MetaSized, ty)?;
    for fault in 0..8 {
        let assumptions = Assumptions::default();
        let mut search = super::super::Search::new(
            selection::Caller {
                assumptions: &assumptions,
                environment: None,
                first_region: 1,
                retain_sized: false,
            },
            Availability::Validated,
        );
        search.query.reuse = Some(Active::new(Reuse::default()));
        let mut value = node(goal, Vec::new());
        let mut scope = ScopeId::ROOT;
        match fault {
            1 => value.outlives.push((RegionId(1), RegionId(2))),
            2 => value.equalities.push((RegionId(1), RegionId(2))),
            3 => value.sized.push(goal.ty),
            4 => value.provisional_size = true,
            5 => value.cycle_dependent = true,
            6 => value.end = 2,
            7 => scope = ScopeId(1),
            _ => {}
        }
        let selected = search
            .query
            .push_source_sizing(&facts, value, scope, 1..1)?;
        search
            .query
            .export_completed(&facts, Decision::Proven(selected))?;
        assert_eq!(
            search
                .query
                .reuse
                .as_ref()
                .ok_or("reuse")?
                .reuse
                .completed
                .contains_key(&goal),
            fault == 0
        );
    }
    Ok(())
}
#[test]
fn sizing_sharing_requires_every_completed_formation_sibling() -> Result {
    let (mut facts, ty) = facts()?;
    let goal = predicate(&mut facts, LangItem::MetaSized, ty)?;
    for unpaid in [false, true] {
        let assumptions = Assumptions::default();
        let mut search = super::super::Search::new(
            selection::Caller {
                assumptions: &assumptions,
                environment: None,
                first_region: 1,
                retain_sized: false,
            },
            Availability::Validated,
        );
        search.query.reuse = Some(Active::new(Reuse::default()));
        let leaf =
            search
                .query
                .push_source_sizing(&facts, node(goal, Vec::new()), ScopeId::ROOT, 1..1)?;
        let mut prefix = node(goal, Vec::new());
        prefix.source = None;
        if unpaid {
            prefix.outlives.push((RegionId(1), RegionId(2)));
        }
        let prefix = search.query.push(prefix, ScopeId::ROOT, 1..1);
        let wrapper = search
            .query
            .push(node(goal, vec![prefix, leaf]), ScopeId::ROOT, 1..1);
        search
            .query
            .export_completed(&facts, Decision::Proven(wrapper))?;
        assert_eq!(
            search
                .query
                .reuse
                .as_ref()
                .ok_or("reuse")?
                .reuse
                .completed
                .contains_key(&goal),
            !unpaid
        );
    }
    Ok(())
}
#[test]
fn unrelated_builtin_does_not_acquire_sizing_sharing_and_hot_layouts_stay_fixed() -> Result {
    let (mut facts, ty) = facts()?;
    let freeze = predicate(&mut facts, LangItem::Freeze, ty)?;
    let assumptions = Assumptions::default();
    let mut search = super::super::Search::new(
        selection::Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 1,
            retain_sized: false,
        },
        Availability::Validated,
    );
    search.query.reuse = Some(Active::new(Reuse::default()));
    assert!(
        search
            .query
            .push_source_sizing(&facts, node(freeze, Vec::new()), ScopeId::ROOT, 1..1)
            .is_err()
    );
    let selected = search
        .query
        .push(node(freeze, Vec::new()), ScopeId::ROOT, 1..1);
    search
        .query
        .export_completed(&facts, Decision::Proven(selected))?;
    assert!(
        !search
            .query
            .reuse
            .as_ref()
            .ok_or("reuse")?
            .reuse
            .completed
            .contains_key(&freeze)
    );
    assert_eq!(size_of::<Query>(), 656);
    assert_eq!(size_of::<Node>(), 160);
    assert_eq!(size_of::<super::super::Frame>(), 1312);
    Ok(())
}
