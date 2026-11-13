//! A certified premise relates only recorded rigid goal normal forms with its
//! own projection subterms; every other differing pair keeps the exact result.
use super::super::{Canonical, Context, Error, Predicate, Store};
use super::Premise;
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, TypeId,
    arguments::Argument,
    copy::Properties,
    infer::Instantiation,
    parameters::{Parameter, ParameterKind, predicates::Bound},
    projection, traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const PATTERN: ItemId = ItemId(1);
const SEARCHER: ItemId = ItemId(2);
const OTHER: ItemId = ItemId(3);
const REVERSE: ItemId = ItemId(4);
const GOAL: RegionId = RegionId(11);
const PREMISE: RegionId = RegionId(12);

struct Fixture {
    store: Store,
    pattern: traits::ApplicationId,
}

impl Fixture {
    fn new() -> TestResult<Self> {
        let mut store = Store::default();
        for item in [SEARCHER, OTHER] {
            store.declare_projection(
                item,
                projection::Schema {
                    trait_item: PATTERN,
                    sized: true,
                    copies: Properties::default(),
                },
            )?;
        }
        let pattern = store.intern_trait_application(traits::Application {
            item: PATTERN,
            arguments: GenericArgs::EMPTY,
        })?;
        Ok(Self { store, pattern })
    }

    fn parameter(&mut self, index: u32) -> TestResult<TypeId> {
        let id = self.store.intern_parameter(Parameter {
            owner: ItemId(100),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        Ok(self.store.intern(Kind::Parameter(id))?)
    }

    fn projection(
        &mut self,
        item: ItemId,
        subject: TypeId,
        arguments: &[Argument],
    ) -> TestResult<TypeId> {
        let arguments = self.store.intern_arguments(arguments)?;
        Ok(self
            .store
            .intern(Kind::Projection(projection::Application {
                item,
                subject,
                trait_application: self.pattern,
                arguments,
            }))?)
    }

    fn searcher(&mut self, subject: TypeId, region: RegionId) -> TestResult<TypeId> {
        self.projection(SEARCHER, subject, &[Argument::Lifetime(region)])
    }

    fn tuple(&mut self, fields: &[TypeId]) -> TestResult<TypeId> {
        let fields = self.store.intern_list(fields)?;
        Ok(self.store.intern(Kind::Tuple(fields))?)
    }

    fn reverse(&mut self, ty: TypeId, region: RegionId) -> TestResult<Canonical> {
        let arguments = self.store.intern_arguments(&[Argument::Lifetime(region)])?;
        Ok(Canonical {
            ty,
            bound: Bound::Trait(self.store.intern_trait_application(traits::Application {
                item: REVERSE,
                arguments,
            })?),
        })
    }

    /// A goal as the complete input rewrite leaves it: its subject is a known
    /// normal form whose projection roots are recorded rigid when `marked`.
    fn goal(
        &mut self,
        context: &mut Context,
        canonical: Canonical,
        marked: bool,
    ) -> TestResult<Predicate> {
        let mut substitution = Instantiation::new(context, &self.store, &[])?;
        let goal = context.instantiate_predicate(&mut self.store, canonical, &mut substitution)?;
        let subject = context.known(&self.store, canonical.ty)?;
        if marked {
            context.retain_rigid_results(&self.store, canonical.ty)?;
        }
        Ok(Predicate { subject, ..goal })
    }
}

/// The exact compare's result for the same goal and premise.
fn exact(
    fixture: &mut Fixture,
    goal: Canonical,
    premise: Canonical,
    marked: bool,
) -> TestResult<Result<(), Error>> {
    let mut context = Context::default();
    let goal = fixture.goal(&mut context, goal, marked)?;
    Ok(context.equate_trait_predicate(&mut fixture.store, &goal, premise))
}

fn normal(
    fixture: &mut Fixture,
    goal: Canonical,
    premise: Canonical,
    marked: bool,
) -> TestResult<(Result<(), Error>, Context)> {
    let mut context = Context::default();
    let goal = fixture.goal(&mut context, goal, marked)?;
    let result = context.equate_normal_premise(&mut fixture.store, &goal, premise);
    Ok((result, context))
}

#[test]
fn certified_premise_relates_rigid_goal_inputs_as_region_equalities() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let goal_ty = fixture.searcher(subject, GOAL)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    assert_eq!(
        exact(&mut fixture, goal, premise, true)?,
        Err(Error::Normalization)
    );
    let (result, context) = normal(&mut fixture, goal, premise, true)?;
    assert_eq!(result, Ok(()));
    // The projection's own lifetime and the trait's lifetime each leave one
    // equality for the caller's region proof; nothing is decided here.
    assert_eq!(
        context.region_equalities,
        vec![(GOAL, PREMISE), (GOAL, PREMISE)]
    );
    // Identical regions need no relation and record nothing.
    let same = fixture.reverse(premise_ty, PREMISE)?;
    let (result, context) = normal(&mut fixture, same, premise, true)?;
    assert_eq!(result, Ok(()));
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn expanded_goal_keeps_its_mark_through_a_trial_import() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let goal_ty = fixture.searcher(subject, GOAL)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    let mut context = Context::default();
    let goal = fixture.goal(&mut context, goal, true)?;
    // A selection query expands the subject before any trial imports it. The
    // rigid mark is the certified relation's alone, so the head still defers.
    assert_eq!(
        context.head(&fixture.store, goal.subject()).err(),
        Some(Error::Normalization)
    );
    let mut probe = Context::default();
    let imported = probe.import_predicate(&fixture.store, &mut context, &goal)?;
    assert_eq!(
        probe.equate_normal_premise(&mut fixture.store, &imported, premise),
        Ok(())
    );
    assert_eq!(
        probe.region_equalities,
        vec![(GOAL, PREMISE), (GOAL, PREMISE)]
    );
    // The unmarked original imports unmarked and keeps the exact result.
    let mut context = Context::default();
    let original = fixture.reverse(goal_ty, GOAL)?;
    let original = fixture.goal(&mut context, original, false)?;
    assert_eq!(
        context.head(&fixture.store, original.subject()).err(),
        Some(Error::Normalization)
    );
    let mut probe = Context::default();
    let imported = probe.import_predicate(&fixture.store, &mut context, &original)?;
    assert_eq!(
        probe.equate_normal_premise(&mut fixture.store, &imported, premise),
        Err(Error::Normalization)
    );
    Ok(())
}

#[test]
fn unmarked_goal_keeps_the_exact_normalization_result() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let goal_ty = fixture.searcher(subject, GOAL)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    let expected = exact(&mut fixture, goal, premise, false)?;
    assert_eq!(expected, Err(Error::Normalization));
    let (result, context) = normal(&mut fixture, goal, premise, false)?;
    assert_eq!(result, expected);
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn different_item_keeps_the_exact_normalization_result() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let goal_ty = fixture.projection(OTHER, subject, &[Argument::Lifetime(GOAL)])?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    let expected = exact(&mut fixture, goal, premise, true)?;
    assert_eq!(expected, Err(Error::Normalization));
    let (result, context) = normal(&mut fixture, goal, premise, true)?;
    assert_eq!(result, expected);
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn different_layout_keeps_the_exact_normalization_result() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    for arguments in [
        vec![Argument::Lifetime(GOAL), Argument::Lifetime(GOAL)],
        vec![Argument::Type(TypeId::UNIT)],
        Vec::new(),
    ] {
        let goal_ty = fixture.projection(SEARCHER, subject, &arguments)?;
        let goal = fixture.reverse(goal_ty, GOAL)?;
        let expected = exact(&mut fixture, goal, premise, true)?;
        assert_eq!(expected, Err(Error::Normalization));
        let (result, context) = normal(&mut fixture, goal, premise, true)?;
        assert_eq!(result, expected);
        assert!(context.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn projection_outside_the_premise_keeps_the_exact_normalization_result() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let other = fixture.parameter(1)?;
    let goal_ty = fixture.searcher(subject, GOAL)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let outside = fixture.searcher(other, PREMISE)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    let mut context = Context::default();
    let goal = context.known(&fixture.store, goal_ty)?;
    context.retain_rigid_results(&fixture.store, goal_ty)?;
    let outside = context.known(&fixture.store, outside)?;
    let inside = context.known(&fixture.store, premise_ty)?;
    // Nested inputs and other blocked pairs are related only against the
    // premise's own outermost projection subterms.
    let mut premise = Premise::new(premise);
    let result =
        context.relate_normal_premise(&mut fixture.store, vec![(goal, outside)], &mut premise);
    assert_eq!(result, Err(Error::Normalization));
    assert!(context.region_equalities.is_empty());
    let result =
        context.relate_normal_premise(&mut fixture.store, vec![(goal, inside)], &mut premise);
    assert_eq!(result, Ok(()));
    assert_eq!(context.region_equalities, vec![(GOAL, PREMISE)]);
    Ok(())
}

#[test]
fn equal_pairs_allocate_no_premise_subterms_or_related_set() -> TestResult {
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let premise_ty = fixture.searcher(subject, PREMISE)?;
    let canonical = fixture.reverse(premise_ty, PREMISE)?;
    let mut context = Context::default();
    let left = context.known(&fixture.store, premise_ty)?;
    let right = context.known(&fixture.store, premise_ty)?;
    let mut premise = Premise::new(canonical);
    context.relate_normal_premise(&mut fixture.store, vec![(left, right); 8], &mut premise)?;
    assert!(premise.subterms.is_none());
    assert!(context.region_equalities.is_empty());
    // A projection-free rewrite result records no mark, allocates no
    // dependency storage or traversal graph and costs no work, so the
    // projection-free rewrites of the scaling controls are unchanged.
    let mut context = Context::default();
    let tuple = fixture.tuple(&[TypeId::UNIT, TypeId::UNIT])?;
    context.retain_rigid_results(&fixture.store, tuple)?;
    assert!(context.dependencies.is_none());
    assert!(
        context
            .projection_subterms(&fixture.store, tuple)?
            .is_empty()
    );
    assert_eq!(context.steps, 0);
    Ok(())
}

/// N distinct rigid pairs against a premise with N projection subterms.
fn tuple_steps(width: u32) -> TestResult<(usize, usize)> {
    let mut fixture = Fixture::new()?;
    let mut goals = Vec::new();
    let mut premises = Vec::new();
    for index in 0..width {
        let subject = fixture.parameter(index)?;
        goals.push(fixture.searcher(subject, GOAL)?);
        premises.push(fixture.searcher(subject, PREMISE)?);
    }
    let goal_ty = fixture.tuple(&goals)?;
    let premise_ty = fixture.tuple(&premises)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    assert_eq!(
        exact(&mut fixture, goal, premise, true)?,
        Err(Error::Normalization)
    );
    let mut context = Context::default();
    let goal = fixture.goal(&mut context, goal, true)?;
    let before = context.steps;
    context.equate_normal_premise(&mut fixture.store, &goal, premise)?;
    Ok((context.steps - before, context.region_equalities.len()))
}

#[test]
fn many_subterms_and_blocked_pairs_cost_linear_work() -> TestResult {
    let mut previous = None;
    let mut observed = Vec::new();
    for width in [16, 64, 256, 1024] {
        let (steps, equalities) = tuple_steps(width)?;
        // One equality per related projection plus the trait's own lifetime.
        assert_eq!(equalities, width as usize + 1);
        if let Some(previous) = previous {
            // The subterm set is collected once per equation, not per pair.
            assert!(steps < 5 * previous, "{steps} after {previous}");
        }
        previous = Some(steps);
        observed.push((width, steps));
    }
    println!("normal-premise steps={observed:?}");
    Ok(())
}

#[test]
fn exhausted_work_fails_closed_at_every_cut() -> TestResult {
    let mut fixture = Fixture::new()?;
    let mut goals = Vec::new();
    let mut premises = Vec::new();
    for index in 0..4 {
        let subject = fixture.parameter(index)?;
        goals.push(fixture.searcher(subject, GOAL)?);
        premises.push(fixture.searcher(subject, PREMISE)?);
    }
    let goal_ty = fixture.tuple(&goals)?;
    let premise_ty = fixture.tuple(&premises)?;
    let goal = fixture.reverse(goal_ty, GOAL)?;
    let premise = fixture.reverse(premise_ty, PREMISE)?;
    let mut context = Context::default();
    let predicate = fixture.goal(&mut context, goal, true)?;
    let before = context.steps;
    context.equate_normal_premise(&mut fixture.store, &predicate, premise)?;
    let needed = context.steps - before;
    for slack in 0..needed {
        let mut context = Context::default();
        let predicate = fixture.goal(&mut context, goal, true)?;
        context.steps = 1_000_000 - slack;
        let result = context.equate_normal_premise(&mut fixture.store, &predicate, premise);
        assert_eq!(result, Err(Error::Budget), "slack {slack} of {needed}");
    }
    let mut context = Context::default();
    let predicate = fixture.goal(&mut context, goal, true)?;
    context.steps = 1_000_000 - needed;
    context.equate_normal_premise(&mut fixture.store, &predicate, premise)?;
    Ok(())
}

/// A body opens its declaration's binder clauses at its universal aliases.
/// Each certified source's image inherits its certificate and an uncertified
/// one's does not. A source the declaration does not hold, as a const trait
/// default's Self premise, carries none, and an extension without sources
/// allocates nothing.
#[test]
fn opened_clauses_carry_only_held_certified_sources() -> TestResult {
    use crate::types::parameters::{
        clauses::Clause,
        environment::{Abilities, Environment},
    };
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let Some(Kind::Parameter(parameter)) = fixture.store.kind(subject) else {
        return Err("missing type parameter".into());
    };
    let binder = RegionId::bound(0, 0)?;
    let bound = fixture.searcher(subject, binder)?;
    let alias = fixture.searcher(subject, GOAL)?;
    let certified = fixture.reverse(bound, binder)?;
    let held = fixture.reverse(bound, RegionId::STATIC)?;
    let foreign = fixture.reverse(subject, binder)?;
    let image = Clause::Predicate(fixture.reverse(alias, GOAL)?);
    let sources = [Clause::Predicate(certified), Clause::Predicate(held)];
    let images = [image, Clause::Predicate(held)];
    let mut declaration = Environment::new(&fixture.store, &[(parameter, Abilities::default())])?
        .with_predicates(&fixture.store, &[certified, held])?;
    let uncertified = declaration.opened_certificates(&sources, &images)?;
    assert!(uncertified.is_empty() && uncertified.capacity() == 0);
    declaration.certify_normal_premises(&fixture.store, &[Clause::Predicate(certified)])?;
    assert!(declaration.has_normal_premises());
    let carried = declaration.opened_certificates(&sources, &images)?;
    assert_eq!(carried, [image]);
    let unheld = [Clause::Predicate(certified), Clause::Predicate(foreign)];
    assert!(
        declaration
            .opened_certificates(&unheld, &images)?
            .is_empty()
    );
    let none = declaration.opened_certificates(&[], &images)?;
    assert!(none.is_empty() && none.capacity() == 0);
    assert!(declaration.opened_certificates(&sources, &[image]).is_err());
    let mut body = declaration.with_clauses(&fixture.store, &images)?;
    assert!(!body.has_normal_premises());
    body.certify_normal_premises(&fixture.store, &carried)?;
    assert!(body.has_normal_premises());
    Ok(())
}

/// A body that opens a `Clone` premise also opens its elaborated `Sized`
/// source, which the declaration assumes as a shape rather than a trait
/// premise. That source carries no certificate and keeps the certified
/// premise's own. A `Sized` subject the declaration does not assume still
/// withholds every certificate.
#[test]
fn opened_clauses_admit_assumed_sized_sources() -> TestResult {
    use crate::types::parameters::{
        clauses::Clause,
        environment::{Abilities, Environment},
    };
    let mut fixture = Fixture::new()?;
    let subject = fixture.parameter(0)?;
    let other = fixture.parameter(1)?;
    let mut parameters = Vec::new();
    for ty in [subject, other] {
        let Some(Kind::Parameter(parameter)) = fixture.store.kind(ty) else {
            return Err("missing type parameter".into());
        };
        parameters.push((parameter, Abilities::default()));
    }
    let declared = fixture.searcher(subject, PREMISE)?;
    let opened = fixture.searcher(subject, GOAL)?;
    let premise = fixture.reverse(declared, PREMISE)?;
    let sized = |ty| Canonical {
        ty,
        bound: Bound::Sized,
    };
    let mut declaration = Environment::new(&fixture.store, &parameters)?
        .with_predicates(&fixture.store, &[premise, sized(declared)])?;
    declaration.certify_normal_premises(&fixture.store, &[Clause::Predicate(premise)])?;
    let image = Clause::Predicate(fixture.reverse(opened, GOAL)?);
    let images = [image, Clause::Predicate(sized(opened))];
    let sources = [
        Clause::Predicate(premise),
        Clause::Predicate(sized(declared)),
    ];
    assert_eq!(declaration.opened_certificates(&sources, &images)?, [image]);
    let unassumed = [Clause::Predicate(premise), Clause::Predicate(sized(other))];
    assert!(
        declaration
            .opened_certificates(&unassumed, &images)?
            .is_empty()
    );
    Ok(())
}
