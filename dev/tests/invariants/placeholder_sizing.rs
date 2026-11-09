//! rustc 1.98 negates one impl's predicates under the other impl's
//! `param_env`, with each unified header variable a placeholder. Under the
//! other header's Sized premises, the builtin proof's shape walk and size
//! queries follow rustc's `sized_conditions`, read neither a parameter's
//! declaration nor an interned property of a type with parameters, agree with
//! the rigid parameter rule of the search, and cost one lookup per node.
use super::NegativeSizing;
use crate::{
    sema::{
        model::Declarations,
        nominals::{PLACEHOLDER_EQUALITY_FENCE, PLACEHOLDER_REINSTALL, Proof, sizes},
    },
    support::bindings::LangItem,
    types::{
        ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        parameters::{
            ParameterKind,
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
            shape::{Placeholders, Properties, Shapes},
        },
    },
};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/negative-premise-sides";
const SIDES: &str = "pa31/tests/spec/negative-metasized";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const WELL_FORMED: [&str; 3] = [
    "trait implementation header must be well-formed",
    "trait argument must be well-formed",
    "bound subject must be well-formed",
];
const TAIL: &str = "pub struct Wrap<X: ?Sized>(X); pub struct Carrier<A, B: ?Sized>(A, *const B); pub trait Tr { type Out: ?Sized; } impl<X: ?Sized> Tr for X { type Out = X; } pub struct P<T: ?Sized + Tr>(<T as Tr>::Out); pub trait Tn { type Out: ?Sized; } pub struct Q<T: ?Sized + Tn>(<T as Tn>::Out); pub trait Ts { type Out; } pub struct R<T: Ts>(<T as Ts>::Out);";

/// The no-core prefix every twin of the packet shares.
fn prefix() -> TestResult<String> {
    let fixture = crate::test_source::fixture(&format!("{PACKET}/nested-tuple-head.rs"))?;
    Ok(fixture
        .split("pub trait Seal")
        .next()
        .ok_or("twin prefix")?
        .to_owned())
}

fn analyze(source: &str) -> TestResult<Result<Declarations, String>> {
    let report = crate::test_source::source_provider(source, "placeholder_sizing")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|analysis| analysis.facts)
        .map_err(|error| error.message.clone()))
}

fn definition(report_source: &str, name: &str) -> TestResult<ItemId> {
    let report = crate::test_source::source_provider(report_source, "placeholder_sizing")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == name)
        .ok_or(name.to_owned())?;
    Ok(ItemId(u32::try_from(index)?))
}

fn item(report_source: &str, facts: &Declarations, name: &str) -> TestResult<ItemId> {
    let item = definition(report_source, name)?;
    facts
        .types
        .nominal_size_rule(item)
        .ok_or("sizedness rule")?;
    Ok(item)
}

/// Every rigid type parameter the store interned, with its declared sizedness.
fn parameters(facts: &Declarations) -> TestResult<Vec<(TypeId, ItemId, bool)>> {
    let mut found = Vec::new();
    for index in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(index)?);
        if let Some(Kind::Parameter(id)) = facts.types.kind(ty)
            && let Some(parameter) = facts.types.parameter(id)
            && let ParameterKind::Type { sized } = parameter.kind
        {
            found.push((ty, parameter.owner, sized));
        }
    }
    Ok(found)
}

struct Fixture {
    facts: Declarations,
    wrap: ItemId,
    projected: ItemId,
    /// `Q`, whose projected field has no implementation to normalize it.
    failing: ItemId,
    /// `<T as Tn>::Out` and `<T as Ts>::Out`: rigid projections whose
    /// declarations relax and keep Sized.
    rigid: [TypeId; 2],
    sized: TypeId,
    unsized_: TypeId,
}

fn fixture() -> TestResult<Fixture> {
    let source = format!("{}{TAIL}", prefix()?);
    let facts = analyze(&source)?.map_err(|error| format!("fixture: {error}"))?;
    let wrap = item(&source, &facts, "Wrap")?;
    let carrier = item(&source, &facts, "Carrier")?;
    let projected = item(&source, &facts, "P")?;
    let failing = item(&source, &facts, "Q")?;
    let rigid = |name: &str| -> TestResult<TypeId> {
        let trait_item = definition(&source, name)?;
        for index in 0..facts.types.len() {
            let ty = TypeId(u32::try_from(index)?);
            if let Some(Kind::Projection(application)) = facts.types.kind(ty)
                && facts
                    .types
                    .projection_schema(application.item)
                    .is_some_and(|schema| schema.trait_item == trait_item)
            {
                return Ok(ty);
            }
        }
        Err(format!("{name} projection").into())
    };
    let rigid = [rigid("Tn")?, rigid("Ts")?];
    let owned: Vec<_> = parameters(&facts)?
        .into_iter()
        .filter(|(_, owner, _)| *owner == carrier)
        .collect();
    let find = |sized: bool| {
        owned
            .iter()
            .find(|(_, _, declared)| *declared == sized)
            .map(|(ty, _, _)| *ty)
            .ok_or("Carrier parameter")
    };
    Ok(Fixture {
        wrap,
        projected,
        failing,
        rigid,
        sized: find(true)?,
        unsized_: find(false)?,
        facts,
    })
}

fn nominal(facts: &mut Declarations, item: ItemId, argument: TypeId) -> TestResult<TypeId> {
    let arguments = facts.types.intern_arguments(&[Argument::Type(argument)])?;
    Ok(facts.types.intern(Kind::Adt(item, arguments))?)
}

fn tuple(facts: &mut Declarations, elements: &[TypeId]) -> TestResult<TypeId> {
    let list = facts.types.intern_list(elements)?;
    Ok(facts.types.intern(Kind::Tuple(list))?)
}

/// `[p, Wrap<p>, Wrap<Wrap<p>>, (p, u8), (u8, p), [p; 1], [p], ()]`.
fn shapes_of(facts: &mut Declarations, wrap: ItemId, p: TypeId) -> TestResult<[TypeId; 8]> {
    let byte = TypeId::primitive(Primitive::U8);
    let once = nominal(facts, wrap, p)?;
    Ok([
        p,
        once,
        nominal(facts, wrap, once)?,
        tuple(facts, &[p, byte])?,
        tuple(facts, &[byte, p])?,
        facts.types.intern(Kind::Array {
            element: p,
            length: 1,
        })?,
        facts.types.intern(Kind::Slice(p))?,
        TypeId::UNIT,
    ])
}

/// A Sized premise set in any order, with no `MetaSized` premise;
/// [`Placeholders::new`] orders it. This is not the shape production gives a
/// set, so only the walk and size-query tests, which read no `MetaSized`
/// premise, use it.
fn placeholders(set: Vec<TypeId>) -> TestResult<Placeholders> {
    Ok(Placeholders::new(set.into(), Arc::from([]))?.0)
}

/// Premises shaped as the overlap inputs build them: every Sized premise is a
/// `MetaSized` premise too, and `meta_only` are `MetaSized` alone, as a
/// `?Sized` parameter is.
fn premises(sized: &[TypeId], meta_only: &[TypeId]) -> TestResult<Placeholders> {
    let meta: Vec<_> = sized.iter().chain(meta_only).copied().collect();
    Ok(Placeholders::new(sized.into(), meta.into())?.0)
}

fn empty(facts: &Declarations) -> TestResult<Arc<Environment>> {
    Ok(Arc::new(Environment::new(&facts.types, &[])?))
}

/// One fresh placeholder walk per type: its answer and its work.
fn walk(facts: &Declarations, set: &Placeholders, ty: TypeId) -> TestResult<(Properties, usize)> {
    let mut shapes = Shapes::with_placeholders(empty(facts)?, set.clone());
    let answer = shapes.demand(&facts.types, ty)?;
    Ok((answer, shapes.steps()))
}

/// One fresh premise-less walk with every parameter of `facts` in scope.
fn walk_without(
    facts: &Declarations,
    environment: &Arc<Environment>,
    ty: TypeId,
) -> (Result<Properties, &'static str>, usize) {
    let mut shapes = Shapes::new(Arc::clone(environment));
    let answer = shapes.demand(&facts.types, ty);
    (answer, shapes.steps())
}

fn scoped(facts: &Declarations) -> TestResult<Arc<Environment>> {
    let scope: Vec<_> = parameters(facts)?
        .into_iter()
        .filter_map(|(ty, _, _)| match facts.types.kind(ty) {
            Some(Kind::Parameter(id)) => Some((id, Abilities::default())),
            _ => None,
        })
        .collect();
    Ok(Arc::new(Environment::new(&facts.types, &scope)?))
}

#[test]
fn placeholders_decide_sizedness_by_rustc_sized_conditions_alone() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let mut observed = Vec::new();
    for p in [fixture.sized, fixture.unsized_] {
        let types = shapes_of(facts, fixture.wrap, p)?;
        let mut rows = Vec::new();
        for set in [vec![], vec![p], vec![types[1]]] {
            let set = placeholders(set)?;
            let mut row = Vec::new();
            for ty in types {
                let (answer, steps) = walk(facts, &set, ty)?;
                assert!(
                    answer.well_formed,
                    "{ty:?}: placeholder walks are well-formed"
                );
                row.push((answer.sized, steps));
            }
            rows.push(row);
        }
        let sized = |row: &[(bool, usize)]| row.iter().map(|(sized, _)| *sized).collect::<Vec<_>>();
        assert_eq!(
            sized(&rows[0]),
            [false, false, false, true, false, true, false, true]
        );
        assert_eq!(
            sized(&rows[1]),
            [true, true, true, true, true, true, false, true]
        );
        assert_eq!(
            sized(&rows[2]),
            [false, true, true, true, false, true, false, true]
        );
        observed.push(rows);
    }
    // The representatives' interned properties differ, so equal walks read none.
    let [sized, unsized_] =
        [fixture.sized, fixture.unsized_].map(|p| shapes_of(facts, fixture.wrap, p));
    let (sized, unsized_) = (sized?, unsized_?);
    assert_eq!(facts.types.is_sized(sized[1]), Some(true));
    assert_eq!(facts.types.is_sized(unsized_[1]), Some(false));
    assert_eq!(facts.types.is_well_formed(unsized_[3]), Some(false));
    assert_eq!(facts.types.is_well_formed(unsized_[5]), Some(false));
    assert_eq!(
        observed[0], observed[1],
        "a declaration or interned property was read"
    );
    Ok(())
}

#[test]
fn placeholder_sub_walkers_inherit_the_premises() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let p = fixture.unsized_;
    let wrapped = shapes_of(facts, fixture.wrap, p)?[2];
    for (set, expected) in [(vec![], false), (vec![p], true)] {
        let set = placeholders(set)?;
        let mut shapes = Shapes::with_placeholders(empty(facts)?, set.clone());
        assert_eq!(shapes.normalized(&facts.types, wrapped)?.sized, expected);
        let mut terms = Shapes::for_clause_terms(empty(facts)?, Some(set.clone()));
        assert_eq!(terms.demand(&facts.types, wrapped)?.sized, expected);
        let mut proof = Proof::new(None);
        proof.assume_placeholders(&facts.types, set)?;
        let requirement = Predicate {
            ty: wrapped,
            bound: Bound::Sized,
        };
        assert_eq!(proof.prove(facts, requirement)?, expected);
    }
    Ok(())
}

fn prove_sized(
    proof: &mut Proof,
    facts: &mut Declarations,
    ty: TypeId,
) -> Result<bool, &'static str> {
    proof.prove(
        facts,
        Predicate {
            ty,
            bound: Bound::Sized,
        },
    )
}

/// A placeholder proof decides a projected-size nominal through its trait
/// State under the same premises, never by an interned property or a
/// declaration: `P<A>` is Sized under {A}, though its interned property, set
/// by its projected rule, is not, and it is not Sized when A is only
/// `MetaSized`, though A's declaration is Sized. The State the walk starts
/// holds the walk's own set. The placeholder size query descends into the
/// nominal's tail.
#[test]
fn placeholder_proofs_decide_projected_sizes_by_premises_and_never_fall_back() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let [sized, unsized_] = [fixture.sized, fixture.unsized_];
    let projected = nominal(facts, fixture.projected, sized)?;
    let hidden = nominal(facts, fixture.projected, unsized_)?;
    assert_eq!(facts.types.is_sized(projected), Some(false));
    for (ty, set, expected) in [
        (projected, premises(&[sized], &[])?, true),
        (projected, premises(&[], &[sized])?, false),
        (hidden, premises(&[unsized_], &[])?, true),
    ] {
        let mut proof = Proof::new(None);
        proof.assume_placeholders(&facts.types, set)?;
        assert_eq!(prove_sized(&mut proof, facts, ty), Ok(expected), "{ty:?}");
        assert_eq!(proof.trait_state_premised(), Some(true), "{ty:?}");
    }
    let mut query = sizes::Query::with_placeholders(None, Some(placeholders(vec![])?));
    let sizes::Step::Projection(tail) = query.advance(facts, projected)? else {
        return Err("a projected-size nominal descends into its tail".into());
    };
    assert!(
        matches!(facts.types.kind(tail), Some(Kind::Projection(application)) if application.subject == sized)
    );
    let mut size = |set: Option<Placeholders>, ty| {
        let mut query = sizes::Query::with_placeholders(None, set);
        match query.advance(facts, ty) {
            Ok(sizes::Step::Ready { sized, .. }) => Ok(Some(sized)),
            Ok(_) => Ok(None),
            Err(error) => Err(error),
        }
    };
    for (set, expected) in [(vec![], false), (vec![sized], true)] {
        let set = placeholders(set)?;
        assert_eq!(size(Some(set.clone()), sized), Ok(Some(expected)));
        let unsized_set = placeholders(if expected { vec![unsized_] } else { vec![] })?;
        assert_eq!(size(Some(unsized_set), unsized_), Ok(Some(expected)));
    }
    assert_eq!(size(None, sized), Ok(Some(true)));
    assert_eq!(size(None, unsized_), Ok(Some(false)));
    Ok(())
}

/// A failed size proof cancels its walk and its scope's size query, never
/// keeping either suspended: the same placeholder proof fails its root again
/// for the same walk work, then answers a closed type, a type with parameters
/// and the failing root as a fresh placeholder proof does, for the same work
/// and never from the representative's interned properties. `Q`'s field has
/// no implementation to normalize it.
#[test]
fn failed_placeholder_size_proofs_cancel_their_walk_and_answer_afresh() -> TestResult {
    const UNIMPLEMENTED: &str = "associated type requires an unambiguous validated implementation";
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let sized = fixture.sized;
    let failing = nominal(facts, fixture.failing, sized)?;
    let wrapped = nominal(facts, fixture.wrap, sized)?;
    let closed = nominal(facts, fixture.wrap, TypeId::primitive(Primitive::U8))?;
    let mut proof = Proof::new(None);
    proof.assume_placeholders(&facts.types, premises(&[sized], &[])?)?;
    assert_eq!(prove_sized(&mut proof, facts, failing), Err(UNIMPLEMENTED));
    assert_eq!(proof.trait_state_premised(), Some(true));
    let failed = proof.shape_steps();
    assert!(failed > 0);
    assert_eq!(prove_sized(&mut proof, facts, failing), Err(UNIMPLEMENTED));
    assert_eq!(proof.shape_steps(), 2 * failed);
    for (ty, expected) in [
        (closed, Ok(true)),
        (wrapped, Ok(true)),
        (failing, Err(UNIMPLEMENTED)),
    ] {
        let mut fresh = Proof::new(None);
        fresh.assume_placeholders(&facts.types, premises(&[sized], &[])?)?;
        assert_eq!(prove_sized(&mut fresh, facts, ty), expected, "{ty:?}");
        assert_ne!(fresh.trait_state_premised(), Some(false), "{ty:?}");
        let before = proof.shape_steps();
        assert_eq!(prove_sized(&mut proof, facts, ty), expected, "{ty:?}");
        assert_eq!(proof.shape_steps() - before, fresh.shape_steps(), "{ty:?}");
    }
    assert_eq!(proof.trait_state_premised(), Some(true));
    Ok(())
}

/// `sizes/query.rs:82`: a projection that normalizes to itself is rigid.
/// Under placeholders it is Sized by a premise, which the query looks up
/// before the projection can be pending, or by its own declared bound. The
/// interned bit cannot disagree with that bound: `projection_properties` and
/// `opaque_properties` set it from the same immutable declaration, which the
/// seal recomputes and `declare_projection` never replaces; a placeholder
/// State, Cursor or Search has no environment to add a projection promise;
/// and membership of the alias itself is decided first. So the query reads
/// the declaration and asserts the agreement.
#[test]
fn rigid_aliases_decide_their_size_by_premises_and_declared_bounds() -> TestResult {
    fn rigid(
        facts: &mut Declarations,
        set: Option<Placeholders>,
        ty: TypeId,
    ) -> TestResult<(bool, usize)> {
        let mut query = sizes::Query::with_placeholders(None, set);
        let mut calls = 0_usize;
        let answer = query.demand(facts, ty, |_, ty| {
            calls += 1;
            Ok(ty)
        })?;
        let sizes::Progress::Ready {
            sized,
            provisional: false,
        } = answer
        else {
            return Err("a rigid alias answers at once".into());
        };
        Ok((sized, calls))
    }
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let [relaxed, bounded] = fixture.rigid;
    for (ty, declared) in [(relaxed, false), (bounded, true)] {
        let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
            return Err("rigid projection".into());
        };
        let schema = facts
            .types
            .projection_schema(application.item)
            .ok_or("schema")?;
        assert_eq!(schema.sized, declared, "{ty:?}");
        assert_eq!(facts.types.is_sized(ty), Some(declared), "{ty:?}");
        assert_eq!(
            rigid(facts, Some(placeholders(vec![])?), ty)?,
            (declared, 1)
        );
        assert_eq!(rigid(facts, None, ty)?, (declared, 1));
    }
    assert_eq!(
        rigid(facts, Some(placeholders(vec![relaxed])?), relaxed)?,
        (true, 0)
    );
    Ok(())
}

/// A placeholder proof of `P^d<B>` under its own set normalizes exactly as
/// often as the premise-less proof of `P^d<A>` in an environment, and its
/// accounted work stays under the line through depths 1 and 2. The walk's
/// own steps are constant in d, because a projected-rule nominal has no sized
/// children: the walk raises one size request, and the State's size proof
/// descends the levels, so the bound counts the State's accounted work as
/// well. The count itself is not asserted: both proofs start 2^d - 1
/// normalizations, a recorded pre-existing performance item, because the
/// nested proofs of each level share no work and their work is not charged
/// to this proof.
#[test]
fn placeholder_projected_size_work_is_linear_in_depth() -> TestResult {
    use crate::sema::nominals::traits::size_normalizations;
    let mut fixture = fixture()?;
    let (b, a) = (fixture.unsized_, fixture.sized);
    let environment = scoped(&fixture.facts)?;
    for (sized, meta_only, expected) in [(vec![b], vec![], true), (vec![a], vec![b], false)] {
        // The premise-less twin is over the Sized A: a hand-built environment
        // gives a `?Sized` parameter no `MetaSized` premise.
        let mut work = Vec::new();
        for depth in [1_usize, 2, 4, 10] {
            let facts = &mut fixture.facts;
            let (mut ty, mut plain_ty) = (b, a);
            for _ in 0..depth {
                ty = nominal(facts, fixture.projected, ty)?;
                plain_ty = nominal(facts, fixture.projected, plain_ty)?;
            }
            let mut plain = Proof::new(Some(Arc::clone(&environment)));
            let before = size_normalizations();
            assert_eq!(
                prove_sized(&mut plain, facts, plain_ty),
                Ok(true),
                "{depth}"
            );
            assert_eq!(plain.trait_state_premised(), Some(false), "{depth}");
            let premise_less = size_normalizations()[0] - before[0];
            let mut proof = Proof::new(None);
            proof.assume_placeholders(&facts.types, premises(&sized, &meta_only)?)?;
            let before = size_normalizations();
            assert_eq!(prove_sized(&mut proof, facts, ty), Ok(expected), "{depth}");
            assert_eq!(proof.trait_state_premised(), Some(true), "{depth}");
            let after = size_normalizations();
            assert_eq!(
                after[0], before[0],
                "{depth}: every normalization is under the set"
            );
            assert_eq!(after[1] - before[1], premise_less, "{depth}");
            work.push(proof.shape_steps() + proof.trait_state_work());
        }
        eprintln!("placeholder projected size work, Sized {expected}: {work:?}");
        let slope = work[1].checked_sub(work[0]).ok_or("work fell with depth")?;
        assert!(work[2] <= work[0] + 3 * slope, "{work:?}");
        assert!(work[3] <= work[0] + 9 * slope, "{work:?}");
    }
    Ok(())
}

/// Decision 3b: with an empty premise set, closed types keep their answers
/// and work, rigid parameters are never Sized, a nominal chain costs exactly
/// one lookup per node and no type costs more than twice its premise-less walk.
#[test]
fn empty_placeholders_differ_from_premise_less_walks_only_by_their_lookups() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let p = fixture.sized;
    let chain = shapes_of(facts, fixture.wrap, p)?;
    let environment = scoped(facts)?;
    for (set, lookups) in [(vec![], [1, 2, 3]), (vec![p], [1, 1, 1])] {
        let set = placeholders(set)?;
        for (index, ty) in chain[..3].iter().enumerate() {
            let (answer, steps) = walk_without(facts, &environment, *ty);
            assert!(answer?.sized);
            assert_eq!(walk(facts, &set, *ty)?.1, steps + lookups[index], "{index}");
        }
    }
    let set = placeholders(vec![])?;
    let mut checked = 0_usize;
    for directory in [PACKET, SIDES] {
        for name in fixtures(directory)? {
            let source = crate::test_source::fixture(&format!("{directory}/{name}"))?;
            let Ok(facts) = analyze(&source)? else {
                continue;
            };
            let environment = scoped(&facts)?;
            for index in 0..facts.types.len() {
                let ty = TypeId(u32::try_from(index)?);
                let (answer, steps) = walk_without(&facts, &environment, ty);
                let mut shapes = Shapes::with_placeholders(empty(&facts)?, set.clone());
                let placeholder = shapes.demand(&facts.types, ty);
                let closed = facts.types.contains_parameters(ty) == Some(false)
                    && facts.types.contains_projections(ty) == Some(false)
                    && facts.types.contains_projected_sizes(ty) != Some(true);
                if closed {
                    assert_eq!(
                        (placeholder, shapes.steps()),
                        (answer, steps),
                        "{name} {index}"
                    );
                } else if matches!(facts.types.kind(ty), Some(Kind::Parameter(_))) {
                    assert!(!placeholder?.sized, "{name} {index}");
                } else if let (Ok(_), Ok(_)) = (answer, placeholder) {
                    assert!(shapes.steps() <= 2 * steps, "{name} {index}");
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 1000, "{checked}");
    Ok(())
}

fn fixtures(directory: &str) -> TestResult<Vec<String>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(directory);
    let mut names = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if std::path::Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// (f) and the placeholder walk read one set: for every rigid parameter of
/// every analyzing packet fixture they give the same answer for the same work.
#[test]
fn rigid_parameter_rule_and_placeholder_walk_agree() -> TestResult {
    let mut checked = 0_usize;
    let meta_sized: Arc<[TypeId]> = Arc::from([]);
    for directory in [PACKET, SIDES] {
        for name in fixtures(directory)? {
            let source = crate::test_source::fixture(&format!("{directory}/{name}"))?;
            let Ok(facts) = analyze(&source)? else {
                continue;
            };
            let found: Vec<_> = parameters(&facts)?
                .into_iter()
                .map(|(ty, _, _)| ty)
                .collect();
            for &p in &found {
                for mut set in [vec![], vec![p], found.clone()] {
                    set.sort_by_key(|ty| ty.0);
                    set.dedup();
                    let sized: Arc<[TypeId]> = set.into();
                    let (premises, construction) =
                        Placeholders::new(Arc::clone(&sized), Arc::clone(&meta_sized))?;
                    assert_eq!(construction, sized.len().max(1) + 1, "{name} {p:?}");
                    let sizing = NegativeSizing::assume(&premises);
                    let (holds, work) = sizing.parameter(LangItem::Sized, p)?;
                    let mut shapes =
                        Shapes::with_placeholders(empty(&facts)?, sizing.placeholders().clone());
                    let answer = shapes.demand(&facts.types, p)?;
                    assert_eq!(answer.sized, holds, "{name} {p:?}");
                    assert_eq!(shapes.steps(), 2 + work, "{name} {p:?}");
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 100, "{checked}");
    Ok(())
}

/// [`Placeholders::new`] orders and deduplicates both premise sets, each on
/// its own, so no lookup depends on its caller's order. An ordered set costs
/// one comparison per premise, at least one; any other set also costs its sort.
#[test]
fn placeholder_premises_are_ordered_whatever_their_callers_order() -> TestResult {
    let ordered: Arc<[TypeId]> = (0..64).map(|index| TypeId(3 * index + 1)).collect();
    let none: Arc<[TypeId]> = Arc::from([]);
    let (set, work) = Placeholders::new(Arc::clone(&ordered), Arc::clone(&none))?;
    assert_eq!(work, 64 + 1);
    let mut shuffled: Vec<_> = ordered.iter().rev().copied().collect();
    shuffled.extend(ordered.iter().step_by(5).copied());
    let count = shuffled.len();
    let sorting = count + count * (count.ilog2() as usize + 1);
    let (unordered, work) = Placeholders::new(shuffled.clone().into(), Arc::clone(&none))?;
    assert_eq!(work, sorting + 1);
    let (meta, work) = Placeholders::new(Arc::clone(&none), shuffled.into())?;
    assert_eq!(work, 1 + sorting);
    for index in 0..200 {
        let ty = TypeId(index);
        let member = (index % 3 == 1 && index < 192, 7);
        assert_eq!(
            (unordered.sized(ty), set.sized(ty), meta.meta_sized(ty)),
            (member, member, member),
            "{index}"
        );
        assert_eq!(
            (set.meta_sized(ty), meta.sized(ty)),
            ((false, 1), (false, 1)),
            "{index}"
        );
    }
    let (empty, work) = Placeholders::new(Arc::clone(&none), none)?;
    let answers = (empty.sized(TypeId(1)), empty.meta_sized(TypeId(1)), work);
    assert_eq!(answers, ((false, 1), (false, 1), 2));
    Ok(())
}

/// A rigid parameter `d` nominals deep against `n` premises costs at most
/// `3 (d + 1) (ilog2(n) + 1)` steps, linear in `d`.
#[test]
fn placeholder_walk_work_is_linear_in_depth_and_logarithmic_in_premises() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    for n in [1_u32, 64, 4096] {
        // Sorted identities far beyond every type the chains intern.
        let sized = (0..n).map(|index| TypeId(u32::MAX - n + index)).collect();
        let (set, _) = Placeholders::new(sized, Arc::from([]))?;
        let lookup = n.ilog2() as usize + 1;
        let mut observed = Vec::new();
        for depth in [8_usize, 16, 32] {
            let mut ty = fixture.unsized_;
            for _ in 0..depth {
                ty = nominal(facts, fixture.wrap, ty)?;
            }
            let (answer, steps) = walk(facts, &set, ty)?;
            assert!(!answer.sized);
            assert!(steps <= 3 * (depth + 1) * lookup, "{n} {depth}: {steps}");
            observed.push(steps);
        }
        assert_eq!(
            observed[2] - observed[1],
            2 * (observed[1] - observed[0]),
            "{observed:?}"
        );
    }
    Ok(())
}

/// A header that fails well-formedness under its own declarations is
/// rejected before coherence, so it never reaches the placeholder walk.
#[test]
fn well_formedness_failing_headers_never_reach_the_placeholder_walk() -> TestResult {
    let prefix = prefix()?;
    let client = "pub struct Local; impl<U: ?Sized> Holds<&U> for Local {}";
    for blanket in [
        "impl<'a, V: ?Sized, S> Holds<&'a (V, u8)> for S where &'a (V, u8): Seal {}",
        "impl<'a, V: ?Sized, S> Holds<&'a V> for S where &'a (V, u8): Seal {}",
    ] {
        let mut errors = Vec::new();
        for (first, second) in [(blanket, client), (client, blanket)] {
            let source = format!(
                "{prefix}pub trait Seal {{}} impl<T> !Seal for &T {{}} #[rustc_strict_coherence] pub trait Holds<X: ?Sized> {{}} {first} {second}"
            );
            let error = analyze(&source)?
                .err()
                .ok_or("a WF-failing header was accepted")?;
            assert_ne!(error, OVERLAP);
            assert!(WELL_FORMED.contains(&error.as_str()), "{blanket}: {error}");
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1], "{blanket}");
    }
    Ok(())
}

fn caller<'a>(
    assumptions: &'a crate::sema::traits::assumptions::Assumptions,
    placeholders: Option<&'a Placeholders>,
) -> crate::sema::traits::selection::Caller<'a> {
    crate::sema::traits::selection::Caller {
        assumptions,
        environment: None,
        first_region: 1,
        retain_sized: false,
        placeholders,
    }
}

/// A search assumes its caller's placeholder premises from its construction:
/// (f) decides its rigid parameters and its builtin proof walks under them,
/// and a placeholder proof's trait State hands them to the searches and
/// normalizations it starts. An environment with a set is rejected, and the
/// tests' premise-less constructor installs none.
#[test]
fn placeholder_callers_install_their_premises_in_every_search() -> TestResult {
    use crate::sema::traits::implementations::Availability;
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let set = placeholders(vec![fixture.sized])?;
    let assumptions = crate::sema::traits::assumptions::Assumptions::default();
    let search = super::super::Search::assuming(
        &facts.types,
        caller(&assumptions, Some(&set)),
        Availability::Declarations,
    )?;
    assert!(search.query.negative_sizing.is_some());
    assert!(search.query.builtins.assumes_placeholders());
    let plain = super::super::Search::assuming(
        &facts.types,
        caller(&assumptions, None),
        Availability::Declarations,
    )?;
    assert!(plain.query.negative_sizing.is_none());
    assert!(!plain.query.builtins.assumes_placeholders());
    let tests =
        super::super::Search::new(caller(&assumptions, Some(&set)), Availability::Validated);
    assert!(tests.query.negative_sizing.is_none());
    assert!(!tests.query.builtins.assumes_placeholders());
    let environment = empty(facts)?;
    let rejected = super::super::Search::assuming(
        &facts.types,
        crate::sema::traits::selection::Caller {
            environment: Some(&environment),
            ..caller(&assumptions, Some(&set))
        },
        Availability::Declarations,
    );
    assert_eq!(
        rejected.err(),
        Some("placeholder premises require a premise-less caller")
    );
    let mut proof = Proof::new(None);
    proof.assume_placeholders(&facts.types, set)?;
    assert!(proof.trait_state_placeholders(facts)?);
    assert!(!Proof::new(None).trait_state_placeholders(facts)?);
    Ok(())
}

/// The premises cost each caller and each nominal trait State one word.
#[test]
fn placeholder_premises_cost_callers_and_trait_states_one_word() {
    assert_eq!(size_of::<Placeholders>(), 8);
    assert_eq!(size_of::<Option<Placeholders>>(), 8);
    assert_eq!(size_of::<crate::sema::traits::selection::Caller<'_>>(), 32);
    assert_eq!(size_of::<crate::sema::traits::selection::OwnedCaller>(), 32);
    assert_eq!(size_of::<crate::sema::nominals::traits::State>(), 808);
}

/// The premise sets of the facts-memo orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Premise {
    Without,
    B,
    A,
}

/// The questions of the facts-memo orders: q1, `(u8, Wrap<B>)` Sized, which
/// the shape walk decides, and q2, `P<B>` Sized, which the proof's trait
/// State decides by normalizing `<B as Tr>::Out` under the same premises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Question {
    Q1,
    Q2,
}

/// `question` by a fresh builtin proof under `premise` over the fixture's
/// shared facts.
fn ask(
    fixture: &mut Fixture,
    question: Question,
    premise: Premise,
) -> TestResult<Result<bool, &'static str>> {
    let facts = &mut fixture.facts;
    let ty = match question {
        Question::Q1 => {
            let wrapped = nominal(facts, fixture.wrap, fixture.unsized_)?;
            tuple(facts, &[TypeId::primitive(Primitive::U8), wrapped])?
        }
        Question::Q2 => nominal(facts, fixture.projected, fixture.unsized_)?,
    };
    let mut proof = Proof::new(None);
    let [b, a] = [fixture.unsized_, fixture.sized];
    // q1 keeps checkpoint 3a's Sized-only sets. q2's sets are shaped as the
    // overlap inputs build them; under {A}, B is `MetaSized` alone.
    let set = match (question, premise) {
        (_, Premise::Without) => None,
        (Question::Q1, Premise::B) => Some(placeholders(vec![b])?),
        (Question::Q1, Premise::A) => Some(placeholders(vec![a])?),
        (Question::Q2, Premise::B) => Some(premises(&[b], &[])?),
        (Question::Q2, Premise::A) => Some(premises(&[a], &[b])?),
    };
    let premised = set.is_some();
    if let Some(set) = set {
        proof.assume_placeholders(&facts.types, set)?;
    }
    let answer = prove_sized(&mut proof, facts, ty);
    assert_ne!(
        proof.trait_state_premised(),
        Some(!premised),
        "{question:?} {premise:?}"
    );
    if question == Question::Q2 && premised {
        assert_eq!(proof.trait_state_premised(), Some(true), "{premise:?}");
    }
    Ok(answer)
}

/// No facts-level memo keeps a placeholder answer: q1 and q2 are Sized only
/// under {B}, and every order of premise-less, {B} and {A} proofs on shared
/// facts answers as the same proof does on fresh facts. A premise-less proof
/// of q2 fails, because its shapes have no environment for B; under {A}, B is
/// only `MetaSized`. Every State a placeholder proof starts holds its set.
/// Only answers are compared, because premise-free memos legitimately cheapen
/// a later proof.
#[test]
fn placeholder_answers_leave_no_trace_in_facts_memos() -> TestResult {
    use Premise::{A, B, Without};
    use Question::{Q1, Q2};
    let mut single = Vec::new();
    for question in [Q1, Q2] {
        for premise in [Without, B, A] {
            let answer = ask(&mut fixture()?, question, premise)?;
            single.push(((question, premise), answer));
        }
    }
    let fresh = |key: (Question, Premise)| {
        single
            .iter()
            .find(|(each, _)| *each == key)
            .map(|(_, answer)| *answer)
            .ok_or("premise")
    };
    assert_eq!(fresh((Q1, Without))?, Ok(false));
    assert_eq!(fresh((Q1, B))?, Ok(true));
    assert_eq!(fresh((Q1, A))?, Ok(false));
    assert_eq!(
        fresh((Q2, Without))?,
        Err("shape parameter is outside its environment")
    );
    assert_eq!(fresh((Q2, B))?, Ok(true));
    assert_eq!(fresh((Q2, A))?, Ok(false));
    for question in [Q1, Q2] {
        assert_ne!(
            fresh((question, Without))?,
            fresh((question, B))?,
            "the set changes {question:?}'s answer"
        );
        assert_ne!(
            fresh((question, B))?,
            fresh((question, A))?,
            "which set it is changes {question:?}'s answer"
        );
        for order in [[Without, B], [B, Without], [B, A], [A, B]] {
            let mut shared = fixture()?;
            for premise in order {
                assert_eq!(
                    ask(&mut shared, question, premise)?,
                    fresh((question, premise))?,
                    "{question:?} {order:?}"
                );
            }
        }
    }
    Ok(())
}

/// A placeholder proof's trait State and normalization cursors take their
/// premises once, before they cache any work: no set, empty set or none
/// replaces them afterwards. An associated equality under a set fails by
/// name before it normalizes anything.
#[test]
fn placeholder_owners_take_their_premises_once_before_any_work() -> TestResult {
    use crate::sema::{
        nominals::traits::State, projections::Cursor, traits::implementations::Availability,
    };
    use crate::types::lifetime_proof::{Obligations, ScopeId};
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let set = placeholders(vec![fixture.unsized_])?;
    let empty = placeholders(vec![])?;
    let mut state = State::new(facts, None, None)?;
    state.assume_placeholders(Some(set.clone()))?;
    for again in [Some(set.clone()), Some(empty.clone()), None] {
        assert_eq!(state.assume_placeholders(again), Err(PLACEHOLDER_REINSTALL));
    }
    let projected = nominal(facts, fixture.projected, TypeId::primitive(Primitive::U8))?;
    let mut used = Proof::new(None);
    assert_eq!(prove_sized(&mut used, facts, projected), Ok(true));
    assert!(used.trait_state_caches_work(facts)?);
    for late in [Some(set.clone()), Some(empty.clone())] {
        assert_eq!(
            used.reinstall_trait_placeholders(facts, late),
            Err(PLACEHOLDER_REINSTALL)
        );
    }
    let proof = Obligations::new(1);
    let mut cursor = Cursor::new(None, &proof, ScopeId::ROOT, Availability::Declarations)?;
    cursor.assume_placeholders(Some(set.clone()))?;
    for again in [Some(set.clone()), Some(empty.clone()), None] {
        assert_eq!(
            cursor.assume_placeholders(again),
            Err(PLACEHOLDER_REINSTALL)
        );
    }
    let mut started = Cursor::new(None, &proof, ScopeId::ROOT, Availability::Declarations)?;
    started.start(facts, projected)?;
    for late in [Some(set.clone()), Some(empty), None] {
        assert_eq!(
            started.assume_placeholders(late),
            Err(PLACEHOLDER_REINSTALL)
        );
    }
    let equality = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::ProjectionEqual(TypeId::UNIT),
    };
    let mut placeholder = Proof::new(None);
    placeholder.assume_placeholders(&facts.types, set)?;
    assert_eq!(
        placeholder.prove_normalized(facts, equality),
        Err(PLACEHOLDER_EQUALITY_FENCE)
    );
    assert_eq!(
        Proof::new(None).prove_normalized(facts, equality),
        Err("associated equality requires an explicit declaration proof scope")
    );
    Ok(())
}
