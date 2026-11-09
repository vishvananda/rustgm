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
        nominals::{PLACEHOLDER_SIZE_FENCE, Proof, sizes},
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
const TAIL: &str = "pub struct Wrap<X: ?Sized>(X); pub struct Carrier<A, B: ?Sized>(A, *const B); pub trait Tr { type Out: ?Sized; } impl<X: ?Sized> Tr for X { type Out = X; } pub struct P<T: ?Sized + Tr>(<T as Tr>::Out);";

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

fn item(report_source: &str, facts: &Declarations, name: &str) -> TestResult<ItemId> {
    let report = crate::test_source::source_provider(report_source, "placeholder_sizing")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == name)
        .ok_or(name.to_owned())?;
    let item = ItemId(u32::try_from(index)?);
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
    sized: TypeId,
    unsized_: TypeId,
}

fn fixture() -> TestResult<Fixture> {
    let source = format!("{}{TAIL}", prefix()?);
    let facts = analyze(&source)?.map_err(|error| format!("fixture: {error}"))?;
    let wrap = item(&source, &facts, "Wrap")?;
    let carrier = item(&source, &facts, "Carrier")?;
    let projected = item(&source, &facts, "P")?;
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
/// [`Placeholders::new`] orders it.
fn placeholders(set: Vec<TypeId>) -> TestResult<Placeholders> {
    Ok(Placeholders::new(set.into(), Arc::from([]))?.0)
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

#[test]
fn placeholder_proofs_fence_projected_sizes_and_never_fall_back() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let [sized, unsized_] = [fixture.sized, fixture.unsized_];
    let projected = nominal(facts, fixture.projected, sized)?;
    let mut proof = Proof::new(None);
    proof.assume_placeholders(&facts.types, placeholders(vec![sized])?)?;
    assert_eq!(
        prove_sized(&mut proof, facts, projected),
        Err(PLACEHOLDER_SIZE_FENCE)
    );
    let mut size = |set: Option<Placeholders>, ty| {
        let mut query = sizes::Query::with_placeholders(None, set);
        match query.advance(facts, ty) {
            Ok(sizes::Step::Ready { sized, .. }) => Ok(Some(sized)),
            Ok(_) => Ok(None),
            Err(error) => Err(error),
        }
    };
    assert_eq!(
        size(Some(placeholders(vec![])?), projected),
        Err(PLACEHOLDER_SIZE_FENCE)
    );
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

/// A fenced walk is cancelled, never kept suspended: the same placeholder
/// proof fences its root again for the same work, then answers a closed type
/// and a type with parameters as a fresh placeholder proof does, for the same
/// work and never from the representative's interned properties.
#[test]
fn fenced_placeholder_proofs_cancel_their_walk_and_answer_afresh() -> TestResult {
    let mut fixture = fixture()?;
    let facts = &mut fixture.facts;
    let sized = fixture.sized;
    let projected = nominal(facts, fixture.projected, sized)?;
    let wrapped = nominal(facts, fixture.wrap, sized)?;
    let closed = nominal(facts, fixture.wrap, TypeId::primitive(Primitive::U8))?;
    let mut proof = Proof::new(None);
    proof.assume_placeholders(&facts.types, placeholders(vec![sized])?)?;
    assert_eq!(
        prove_sized(&mut proof, facts, projected),
        Err(PLACEHOLDER_SIZE_FENCE)
    );
    let fenced = proof.shape_steps();
    assert!(fenced > 0);
    assert_eq!(
        prove_sized(&mut proof, facts, projected),
        Err(PLACEHOLDER_SIZE_FENCE)
    );
    assert_eq!(proof.shape_steps(), 2 * fenced);
    for (ty, expected) in [
        (closed, Ok(true)),
        (wrapped, Ok(true)),
        (projected, Err(PLACEHOLDER_SIZE_FENCE)),
    ] {
        let mut fresh = Proof::new(None);
        fresh.assume_placeholders(&facts.types, placeholders(vec![sized])?)?;
        assert_eq!(prove_sized(&mut fresh, facts, ty), expected, "{ty:?}");
        let before = proof.shape_steps();
        assert_eq!(prove_sized(&mut proof, facts, ty), expected, "{ty:?}");
        assert_eq!(proof.shape_steps() - before, fresh.shape_steps(), "{ty:?}");
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
            assert!(
                error != OVERLAP && error != PLACEHOLDER_SIZE_FENCE,
                "{error}"
            );
            assert!(WELL_FORMED.contains(&error.as_str()), "{blanket}: {error}");
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1], "{blanket}");
    }
    Ok(())
}
