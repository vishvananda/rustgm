//! An impl's own completed associated-type bound is a normal form in its
//! environment. A supertrait or body goal whose rewritten subject is a rigid
//! projection matches that premise when they differ only in lifetime inputs,
//! which become region equalities, as rustc 1.98.0 relates a rigid alias's
//! arguments. An elaborated supertrait premise inherits its declared bound's
//! certificate when it keeps that bound's subject and binder and its trait
//! application adds no projection. Other elaborated, raw-environment and
//! written-equation premises keep the exact compare.
use std::fmt::Write;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa21/tests/spec/assoc-bounds/certified-premises";
/// A related lifetime input that the premise does not prove is a region error.
const REGION: &str = "trait argument lifetime requirement is not satisfied: required lifetime does not outlive its target";

/// Outcomes declared before rustc 1.98.0 ran: `None` accepts, otherwise the
/// first error this compiler reports for a control that is rejected.
const CASES: [(&str, Option<&str>); 19] = [
    ("bound-positive", None),
    ("where-positive", None),
    ("mixed-positive", None),
    ("hrtb-positive", None),
    ("hrtb-premise", None),
    ("outlives-both-ways", None),
    ("lifetime-trait-positive", None),
    ("body-impl-positive", None),
    ("plain-positive", None),
    ("static-positive", None),
    ("normalizable-region", None),
    ("call-positive", None),
    ("substituted-normalizable", None),
    (
        "bound-missing",
        Some("trait implementation does not satisfy the trait's requirements"),
    ),
    ("hrtb-required", Some(REGION)),
    ("other-lifetime", Some(REGION)),
    ("outlives-one-way", Some(REGION)),
    (
        "gate-different-item",
        Some("trait implementation does not satisfy the trait's requirements"),
    ),
    // rustc accepts; the raw-environment premise is never certified. Stricter.
    (
        "raw-environment-premise",
        Some("associated type requires an unambiguous implementation"),
    ),
];

fn analyze(source: &str) -> TestResult<Result<(), String>> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

/// Each control whose first error differs from its declaration.
fn declared_differences(folder: &str, cases: &[(&str, Option<&str>)]) -> TestResult<Vec<String>> {
    let mut differences = Vec::new();
    for &(name, expected) in cases {
        let source = crate::test_source::fixture(&format!("{folder}/{name}.rs"))?;
        let actual = analyze(&source)?;
        if actual.as_ref().err().map(String::as_str) != expected {
            differences.push(format!("{name}: {actual:?}, declared {expected:?}"));
        }
    }
    Ok(differences)
}

#[test]
fn certified_bound_premises_match_declared_outcomes() -> TestResult {
    let differences = declared_differences(CONTROLS, &CASES)?;
    assert!(differences.is_empty(), "{differences:#?}");
    Ok(())
}

/// `width` lifetime inputs on the member and `premises` distinct bounds.
fn scaled(width: usize, premises: usize) -> TestResult<String> {
    let mut source = String::new();
    let lifetimes = (0..width)
        .map(|i| format!("'x{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let applied = vec!["'a"; width].join(", ");
    writeln!(
        source,
        "trait Pattern: Sized {{ type Searcher<{lifetimes}>; }}"
    )?;
    for index in 0..premises {
        writeln!(source, "trait R{index}<'a> {{}}")?;
    }
    let bounds = (0..premises)
        .map(|i| format!("P::Searcher<{applied}>: R{i}<'a>"))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(
        source,
        "struct Split<'a, P: Pattern>(P::Searcher<{applied}>);"
    )?;
    writeln!(source, "trait Walk {{}} trait FusedWalk: Walk {{}}")?;
    writeln!(
        source,
        "impl<'a, P> Walk for Split<'a, P> where P: Pattern, {bounds} {{}}"
    )?;
    writeln!(
        source,
        "impl<'a, P> FusedWalk for Split<'a, P> where P: Pattern, {bounds} {{}}"
    )?;
    source.push_str("fn main() {}\n");
    Ok(source)
}

/// Selection rows for `width` × `premises`: each supertrait condition first
/// meets its premise unrewritten, then matches once after one rewrite.
fn requests(width: usize, premises: usize) -> TestResult<(usize, usize)> {
    let source = scaled(width, premises)?;
    let capture = crate::sema::copy_trace::Capture::matching(&[
        "assumption-match ",
        "assumption-needs-normalization ",
    ])?;
    let result = analyze(&source)?;
    let rows = capture.rows()?;
    drop(capture);
    assert_eq!(result, Ok(()), "{width}x{premises}");
    let matched = rows
        .iter()
        .filter(|r| r.starts_with("assumption-match "))
        .count();
    Ok((matched, rows.len() - matched))
}

#[test]
fn certified_premise_requests_do_not_grow_with_member_lifetimes() -> TestResult {
    let mut observed = Vec::new();
    for premises in [1, 4, 16] {
        let base = requests(1, premises)?;
        for width in [4, 16] {
            let wide = requests(width, premises)?;
            assert_eq!(wide, base, "{width}x{premises}");
        }
        observed.push((premises, base));
    }
    let (_, (first_matched, _)) = observed[0];
    for &(premises, (matched, retried)) in &observed {
        // Matches grow by at most four for each added premise, whatever the
        // member's width, and each supertrait condition is retried exactly
        // once, after its one complete rewrite.
        assert!(
            matched - first_matched <= 4 * (premises - 1),
            "{observed:?}"
        );
        assert_eq!(retried, premises, "{observed:?}");
    }
    println!("certified-premise requests={observed:?}");
    Ok(())
}

/// A body copies its environment's trait premises, aliasing each escaping
/// one at its universals. A copy inherits only its own source's
/// certificate, and certifying a premise the copy does not hold fails.
#[test]
fn copied_premises_keep_only_their_own_sources_certificates() -> TestResult {
    use crate::sema::traits::assumptions::Assumptions;
    use crate::types::{
        GenericArgs, ItemId, Kind, RegionId, Store,
        arguments::Argument,
        copy::Properties,
        parameters::{
            Parameter, ParameterKind,
            clauses::Clause,
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
        projection, traits,
    };
    let mut store = Store::default();
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let pattern = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(100),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(parameter))?;
    // `<P as Pattern>::Searcher<'r>: ReverseSearcher<'r>`.
    let mut premise = |region: RegionId| -> TestResult<Clause> {
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let ty = store.intern(Kind::Projection(projection::Application {
            item: ItemId(2),
            subject,
            trait_application: pattern,
            arguments,
        }))?;
        let bound = Bound::Trait(store.intern_trait_application(traits::Application {
            item: ItemId(4),
            arguments,
        })?);
        Ok(Clause::Predicate(Predicate { ty, bound }))
    };
    let certified = premise(RegionId::bound(0, 0)?)?;
    let uncertified = premise(RegionId::STATIC)?;
    let alias = premise(RegionId(11))?;
    let ordinary = [certified, uncertified]
        .into_iter()
        .map(Clause::ordinary)
        .collect::<Result<Vec<_>, _>>()?;
    let mut environment = Environment::new(&store, &[(parameter, Abilities::default())])?
        .with_predicates(&store, &ordinary)?;
    environment.certify_normal_premises(&store, &[certified])?;
    let premises = [(certified, alias), (uncertified, uncertified)];
    let mut copied = Assumptions::copied(&store, Some(&environment), &premises)?;
    let index = copied.index.as_ref().ok_or("missing copied index")?;
    assert!(index.normal_premise(alias));
    assert!(!index.normal_premise(uncertified));
    let plain = Assumptions::copied(&store, None, &premises)?;
    let index = plain.index.as_ref().ok_or("missing plain index")?;
    assert!(!index.normal_premise(alias) && !index.normal_premise(uncertified));
    assert!(copied.certify_normal(&[certified]).is_err());
    assert!(Assumptions::default().certify_normal(&[alias]).is_err());
    copied.certify_normal(&[uncertified])?;
    Ok(())
}

const ELABORATED: &str = "pa21/tests/spec/assoc-bounds/elaborated-premises";
const REQUIREMENTS: &str = "trait implementation does not satisfy the trait's requirements";
/// Batch 18's outcomes, declared before rustc 1.98.0 ran: an elaborated
/// supertrait premise inherits its declared bound's certificate when it keeps
/// that bound's subject and binder and adds no projection.
const ELABORATED_CASES: [(&str, Option<&str>); 9] = [
    ("elaborated-bound", None),
    ("elaborated-where", None),
    ("elaborated-two-steps", None),
    ("elaborated-hrtb", None),
    ("elaborated-body", None),
    ("elaborated-explicit", None),
    ("elaborated-missing", Some(REQUIREMENTS)),
    ("elaborated-other-lifetime", Some(REGION)),
    // rustc accepts; a projection-bearing supertrait argument stays uncertified. Stricter.
    ("elaborated-projection-argument", Some(REQUIREMENTS)),
];

#[test]
fn elaborated_premises_match_declared_outcomes() -> TestResult {
    let differences = declared_differences(ELABORATED, &ELABORATED_CASES)?;
    assert!(differences.is_empty(), "{differences:#?}");
    Ok(())
}

/// `<P as Pattern>::Searcher<'a>` and `<P as Pattern>::Key` premises over
/// trait items numbered from 10, with `'a` universal.
struct Elaborated {
    store: crate::types::Store,
    parameter: crate::types::parameters::ParameterId,
    searcher: crate::types::TypeId,
    key: crate::types::TypeId,
    region: crate::types::RegionId,
}
impl Elaborated {
    fn new() -> TestResult<Self> {
        use crate::types::{
            GenericArgs, ItemId, Kind, RegionId, Store,
            arguments::Argument,
            copy::Properties,
            parameters::{Parameter, ParameterKind},
            projection, traits,
        };
        let mut store = Store::default();
        for item in [2, 3] {
            store.declare_projection(
                ItemId(item),
                projection::Schema {
                    trait_item: ItemId(1),
                    sized: true,
                    copies: Properties::default(),
                },
            )?;
        }
        let pattern = store.intern_trait_application(traits::Application {
            item: ItemId(1),
            arguments: GenericArgs::EMPTY,
        })?;
        let parameter = store.intern_parameter(Parameter {
            owner: ItemId(100),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let subject = store.intern(Kind::Parameter(parameter))?;
        let region = RegionId(11);
        let lifetime = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let searcher = store.intern(Kind::Projection(projection::Application {
            item: ItemId(2),
            subject,
            trait_application: pattern,
            arguments: lifetime,
        }))?;
        let key = store.intern(Kind::Projection(projection::Application {
            item: ItemId(3),
            subject,
            trait_application: pattern,
            arguments: GenericArgs::EMPTY,
        }))?;
        Ok(Self {
            store,
            parameter,
            searcher,
            key,
            region,
        })
    }
    /// `ty: Trait<'a, extra...>` for trait item `item`.
    fn premise(
        &mut self,
        ty: crate::types::TypeId,
        item: u32,
        extra: &[crate::types::arguments::Argument],
    ) -> TestResult<crate::types::parameters::clauses::Clause> {
        use crate::types::{
            ItemId,
            arguments::Argument,
            parameters::{
                clauses::Clause,
                predicates::{Bound, Predicate},
            },
            traits,
        };
        let mut arguments = vec![Argument::Lifetime(self.region)];
        arguments.extend_from_slice(extra);
        let arguments = self.store.intern_arguments(&arguments)?;
        let bound = Bound::Trait(self.store.intern_trait_application(traits::Application {
            item: ItemId(item),
            arguments,
        })?);
        Ok(Clause::Predicate(Predicate { ty, bound }))
    }
    /// `for<'x> <P as Pattern>::Searcher<'x>: Trait<'x>` for trait item `item`.
    fn quantified(&mut self, item: u32) -> TestResult<crate::types::parameters::clauses::Clause> {
        use crate::types::{
            ItemId, Kind, RegionId,
            arguments::Argument,
            parameters::{
                clauses::{Clause, Quantified},
                predicates::{Bound, Predicate},
            },
            projection, traits,
        };
        let bound_region = RegionId::bound(0, 0)?;
        let arguments = self
            .store
            .intern_arguments(&[Argument::Lifetime(bound_region)])?;
        let Some(Kind::Projection(application)) = self.store.kind(self.searcher) else {
            return Err("searcher projection".into());
        };
        let ty = self
            .store
            .intern(Kind::Projection(projection::Application {
                arguments,
                ..application
            }))?;
        let bound = Bound::Trait(self.store.intern_trait_application(traits::Application {
            item: ItemId(item),
            arguments,
        })?);
        let id = self.store.intern_quantified(Quantified {
            bound_regions: 1,
            predicate: Predicate { ty, bound },
        })?;
        Ok(Clause::ForAll(id))
    }
}
fn sourced(
    values: &[crate::types::parameters::clauses::Clause],
) -> Vec<crate::sema::generics::Predicate> {
    let source = crate::resolve::unit::NodeKey {
        file: crate::support::source::FileId(0),
        node: crate::syntax::ast::NodeId::new(1),
    };
    values
        .iter()
        .map(|&value| crate::sema::generics::Predicate { source, value })
        .collect()
}

/// Only a premise with a certified bound's subject and binder, and no
/// projection in its trait application, inherits the certificate.
#[test]
fn elaborated_premises_inherit_only_a_certified_subjects_certificate() -> TestResult {
    use crate::sema::generics::elaborated_certificates::inherited;
    use crate::types::arguments::Argument;
    let mut f = Elaborated::new()?;
    let (searcher, key) = (f.searcher, f.key);
    let declared = f.premise(searcher, 10, &[])?;
    let reverse = f.premise(searcher, 11, &[])?;
    let keyed = f.premise(searcher, 12, &[Argument::Type(key)])?;
    let other_subject = f.premise(key, 11, &[])?;
    let rebound = f.quantified(11)?;
    let elaborated = sourced(&[reverse, keyed, other_subject, rebound]);
    assert_eq!(inherited(&f.store, &[declared], &elaborated)?, [reverse]);
    assert!(inherited(&f.store, &[], &elaborated)?.is_empty());
    // A quantified certified bound passes on to a premise under the same binder.
    let quantified = f.quantified(10)?;
    assert_eq!(inherited(&f.store, &[quantified], &elaborated)?, [rebound]);
    Ok(())
}

/// A certification's result, the generics it ran on and each payment it made.
type Certified = (
    Result<(), &'static str>,
    crate::sema::generics::Generics,
    Vec<(usize, usize)>,
);

/// The owner of `elaborated` premises after `declared`, published in its own
/// environment, certified with a recording payment.
fn certify_owner(
    f: &Elaborated,
    declared: crate::types::parameters::clauses::Clause,
    elaborated: &[crate::types::parameters::clauses::Clause],
    fail_at: Option<usize>,
) -> TestResult<Certified> {
    use crate::types::parameters::environment::{Abilities, Environment};
    let owner = crate::resolve::model::DefId(7);
    let mut generics = crate::sema::generics::Generics::default();
    let mut all = vec![declared];
    all.extend_from_slice(elaborated);
    generics.predicates.insert(owner, sourced(&all));
    generics.assume_declared_count(owner, 1);
    let ordinary = all
        .iter()
        .map(|clause| clause.ordinary())
        .collect::<Result<Vec<_>, _>>()?;
    let environment = Environment::new(&f.store, &[(f.parameter, Abilities::default())])?
        .with_predicates(&f.store, &ordinary)?;
    generics
        .environments
        .insert(owner, std::sync::Arc::new(environment));
    let payments = std::cell::RefCell::new(Vec::new());
    let pay = |work: usize, bytes: usize| {
        let mut payments = payments.borrow_mut();
        if fail_at == Some(payments.len()) {
            return Err("certification work exhausted");
        }
        payments.push((work, bytes));
        Ok(())
    };
    let result = generics.certify_environment(&f.store, owner, &[declared], Some(&pay));
    Ok((result, generics, payments.into_inner()))
}

#[test]
fn elaborated_certification_costs_linear_work_and_fails_closed() -> TestResult {
    let owner = crate::resolve::model::DefId(7);
    let mut f = Elaborated::new()?;
    let searcher = f.searcher;
    let declared = f.premise(searcher, 10, &[])?;
    let mut observed = Vec::new();
    for count in [1_u32, 16, 256] {
        let elaborated = (0..count)
            .map(|index| f.premise(searcher, 20 + index, &[]))
            .collect::<TestResult<Vec<_>>>()?;
        let (result, generics, payments) = certify_owner(&f, declared, &elaborated, None)?;
        result?;
        let environment = generics.environments.get(&owner).ok_or("environment")?;
        assert!(
            elaborated
                .iter()
                .all(|&clause| environment.certified_normal_premise(clause))
        );
        let work: usize = payments.iter().map(|&(work, _)| work).sum();
        // The scan pays one step per declared and elaborated premise, and the
        // certificate set its map growth: linear in the premises it certifies.
        let count = usize::try_from(count)?;
        assert!(work <= 6 * (count + 1) + 2, "{count}: {payments:?}");
        observed.push((count, work));
        for fail_at in 0..payments.len() {
            let (result, generics, _) = certify_owner(&f, declared, &elaborated, Some(fail_at))?;
            assert!(result.is_err());
            let environment = generics.environments.get(&owner).ok_or("environment")?;
            assert!(!environment.has_normal_premises(), "{count} at {fail_at}");
        }
    }
    println!("elaborated-certification work={observed:?}");
    Ok(())
}

/// An owner without elaborated premises pays exactly the certificate map's
/// growth, as before this batch; one without certified bounds pays nothing.
#[test]
fn uncertified_owner_pays_no_elaborated_scan() -> TestResult {
    let mut f = Elaborated::new()?;
    let searcher = f.searcher;
    let declared = f.premise(searcher, 10, &[])?;
    let (result, _, payments) = certify_owner(&f, declared, &[], None)?;
    result?;
    let growth = crate::support::retained::Growth::map::<
        crate::types::parameters::clauses::Clause,
        (),
    >(0, 0, 1);
    assert_eq!(payments, [(growth.work + 1, growth.bytes)]);
    let owner = crate::resolve::model::DefId(7);
    let mut generics = crate::sema::generics::Generics::default();
    let called = std::cell::Cell::new(false);
    let pay = |_: usize, _: usize| {
        called.set(true);
        Ok(())
    };
    generics.certify_environment(&f.store, owner, &[], Some(&pay))?;
    assert!(!called.get());
    Ok(())
}

const CONDITIONS: &str = "pa21/tests/spec/assoc-bounds/projection-conditions";
/// Census 21's record 1: the projection path found no selected implementation.
const RECORD: &str = "associated type requires an unambiguous validated implementation";

/// Batch 19's outcomes. The first six are batch 18's probes, whose rustc 1.98.0
/// outcomes were observed; the next five were declared before rustc ran, and
/// the last two after it. A member's conformance contract carries its owner's
/// certified premises when the trait method contributes no clause of its own.
const CONDITION_CASES: [(&str, Option<&str>); 13] = [
    ("cond-elaborated", None),
    ("cond-written", None),
    ("cond-where-form", None),
    ("cond-item-signature", None),
    ("cond-no-bound", None),
    ("cond-no-item", None),
    ("cond-body-call", None),
    ("cond-hrtb", None),
    ("cond-missing", Some(REQUIREMENTS)),
    (
        "cond-other-lifetime",
        Some(
            "implementation method lifetime contract does not satisfy trait: required lifetime does not outlive its target",
        ),
    ),
    // The declared signature mismatch at `next_back`, with its first observed text.
    ("cond-wrong-item", Some("incompatible types")),
    // rustc accepts; the trait method's own `Q: Copy` keeps the exact compare. Stricter.
    ("cond-method-bound", Some(RECORD)),
    // An unbounded method type parameter's implicit `Sized` is its kind, not a clause.
    ("cond-method-type-parameter", None),
];

#[test]
fn projection_conditions_match_declared_outcomes() -> TestResult {
    let differences = declared_differences(CONDITIONS, &CONDITION_CASES)?;
    assert!(differences.is_empty(), "{differences:#?}");
    Ok(())
}

/// An owner environment over `clauses` that certified `certified`, and a
/// member contract over the same clauses plus `extra`, with one more type
/// parameter, as a trait method's unbounded `Q` adds.
fn contract_pair(
    f: &mut Elaborated,
    clauses: &[crate::types::parameters::clauses::Clause],
    certified: &[crate::types::parameters::clauses::Clause],
    extra: &[crate::types::parameters::clauses::Clause],
) -> TestResult<(
    std::sync::Arc<crate::types::parameters::environment::Environment>,
    crate::types::parameters::environment::Environment,
)> {
    use crate::types::{
        ItemId,
        parameters::{
            Parameter, ParameterKind,
            environment::{Abilities, Environment},
        },
    };
    let method = f.store.intern_parameter(Parameter {
        owner: ItemId(101),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let mut owner = Environment::new(&f.store, &[(f.parameter, Abilities::default())])?
        .with_clauses(&f.store, clauses)?;
    owner.certify_normal_premises(&f.store, certified)?;
    let mut all = clauses.to_vec();
    all.extend_from_slice(extra);
    let parameters = [
        (f.parameter, Abilities::default()),
        (method, Abilities::default()),
    ];
    let contract = Environment::new(&f.store, &parameters)?.with_clauses(&f.store, &all)?;
    Ok((std::sync::Arc::new(owner), contract))
}

/// A contract whose method adds no clause carries exactly its owner's certified
/// premises; a method clause, a contract clause the owner lacks, or an owner
/// without certificates carries none, and the owner itself never changes.
#[test]
fn member_contracts_carry_exactly_their_owners_certified_premises() -> TestResult {
    use crate::sema::traits::conformance::carried_certificates;
    let mut f = Elaborated::new()?;
    let (searcher, key) = (f.searcher, f.key);
    let declared = f.premise(searcher, 10, &[])?;
    let reverse = f.premise(searcher, 11, &[])?;
    let rebound = f.quantified(12)?;
    let uncertified = f.premise(key, 13, &[])?;
    let clauses = [declared, reverse, rebound, uncertified];
    let certified = [declared, reverse, rebound];
    let (owner, contract) = contract_pair(&mut f, &clauses, &certified, &[])?;
    let count = std::sync::Arc::strong_count(&owner);
    let mut steps = 0;
    let mut carried = carried_certificates(Some(&owner), 0, &contract, &mut steps)?;
    carried.sort_unstable_by_key(|clause| format!("{clause:?}"));
    let mut expected = certified.to_vec();
    expected.sort_unstable_by_key(|clause| format!("{clause:?}"));
    assert_eq!(carried, expected);
    assert_eq!(std::sync::Arc::strong_count(&owner), count);
    assert!(
        clauses.iter().all(|&clause| {
            owner.certified_normal_premise(clause) == certified.contains(&clause)
        })
    );
    // Certification writes only through this contract's own clause set.
    let shared = contract.clone();
    let mut copy = contract;
    copy.certify_normal_premises(&f.store, &carried)?;
    assert!(!copy.has_normal_premises() && !shared.has_normal_premises());
    drop(shared);
    copy.certify_normal_premises(&f.store, &carried)?;
    assert!(
        certified
            .iter()
            .all(|&clause| copy.certified_normal_premise(clause))
    );
    assert!(!copy.certified_normal_premise(uncertified));
    // A method clause of its own, after elaboration, carries nothing.
    assert!(carried_certificates(Some(&owner), 1, &copy, &mut steps)?.is_empty());
    // A contract clause the owner lacks carries nothing, even when uncounted.
    let foreign = f.premise(searcher, 14, &[])?;
    let (owner, contract) = contract_pair(&mut f, &clauses, &certified, &[foreign])?;
    assert!(carried_certificates(Some(&owner), 0, &contract, &mut steps)?.is_empty());
    // An owner without certificates pays one step and scans nothing.
    let (owner, contract) = contract_pair(&mut f, &clauses, &[], &[])?;
    let mut steps = 0;
    assert!(carried_certificates(Some(&owner), 0, &contract, &mut steps)?.is_empty());
    assert!(carried_certificates(None, 0, &contract, &mut steps)?.is_empty());
    assert_eq!(steps, 2);
    Ok(())
}

/// The carry reads each contract clause a fixed number of times.
#[test]
fn member_contract_carry_costs_linear_work() -> TestResult {
    use crate::sema::traits::conformance::carried_certificates;
    let mut f = Elaborated::new()?;
    let searcher = f.searcher;
    let mut observed = Vec::new();
    for count in [1_u32, 16, 256] {
        let clauses = (0..count)
            .map(|index| f.premise(searcher, 20 + index, &[]))
            .collect::<TestResult<Vec<_>>>()?;
        let (owner, contract) = contract_pair(&mut f, &clauses, &clauses, &[])?;
        let mut steps = 0;
        let carried = carried_certificates(Some(&owner), 0, &contract, &mut steps)?;
        assert_eq!(carried.len(), clauses.len());
        let held = contract.trait_clauses()?.count();
        assert_eq!(held, clauses.len());
        assert_eq!(steps, 3 * held + 1, "{count}");
        observed.push((count, steps));
    }
    println!("member-contract-carry work={observed:?}");
    Ok(())
}
