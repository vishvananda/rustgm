//! Declaration aliases close before external IDs enter a parameter substitution.
use super::*;
use crate::{
    sema::values::Instance,
    types::{GenericArgs, Kind, arguments::Argument},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Fixture {
    facts: Declarations,
    owner: DefId,
    source: NodeKey,
    original: Instance,
    region: RegionId,
    equations: [Range<usize>; 2],
}
fn fixture() -> Result<Fixture> {
    let report = crate::test_source::resolve(
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{const C:T::Out={let n=T::V;n};}fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let initializer = facts.initializers.last().ok_or("missing initializer")?;
    let owner = initializer.definition;
    let source = initializer.semantic_source();
    let original = facts
        .constant_uses
        .values()
        .copied()
        .find(|instance| facts.associated.trait_owner(instance.definition).is_some())
        .ok_or("missing original typed dependency")?;
    let region = facts
        .types
        .arguments(original.arguments)
        .and_then(|mut args| {
            args.find_map(|argument| match argument {
                Argument::Lifetime(region) if region.bound_parts().is_none() => Some(region),
                _ => None,
            })
        })
        .ok_or("missing source alias")?;
    let positions = facts
        .region_equalities
        .iter()
        .enumerate()
        .filter_map(|(index, row)| (row.body == source).then_some(index))
        .collect::<Vec<_>>();
    let first = *positions.first().ok_or("missing source equations")?;
    let end = positions.last().ok_or("missing final equation")? + 1;
    assert!(
        facts.region_equalities[first..end]
            .iter()
            .all(|row| row.body == source)
    );
    Ok(Fixture {
        facts,
        owner,
        source,
        original,
        region,
        equations: [first..end, end..end],
    })
}
fn append(fixture: &mut Fixture, count: usize) -> [Range<usize>; 2] {
    let rows = fixture
        .facts
        .constant_dependencies
        .entry(fixture.owner)
        .or_default();
    let first = rows.len();
    rows.extend(std::iter::repeat_n(
        Dependency::Constant(fixture.original),
        count,
    ));
    let end = rows.len();
    [first..end, end..end]
}
fn selected(fixture: &Fixture, index: usize) -> Result<Instance> {
    match fixture.facts.constant_dependencies[&fixture.owner][index] {
        Dependency::Constant(instance) => Ok(instance),
        Dependency::Callable(_) => Err("wrong dependency kind".into()),
    }
}

#[test]
fn source_aliases_close_before_numerically_colliding_external_arguments() -> Result {
    let mut fixture = fixture()?;
    let ranges = append(&mut fixture, 1);
    let index = ranges[0].start;
    let region_count = fixture.facts.body_regions.len();
    let equation_count = fixture.facts.region_equalities.len();
    canonicalize(
        &mut fixture.facts,
        fixture.owner,
        fixture.source,
        1,
        ranges,
        fixture.equations.clone(),
    )?;
    let canonical = selected(&fixture, index)?;
    let arguments = fixture
        .facts
        .types
        .arguments(canonical.arguments)
        .ok_or("missing arguments")?
        .collect::<Vec<_>>();
    assert_eq!(arguments[0], Argument::Lifetime(RegionId::bound(0, 0)?));
    let tag = fixture
        .facts
        .definitions
        .iter()
        .flatten()
        .copied()
        .find(|&ty| {
            matches!(
                fixture.facts.types.kind(ty),
                Some(Kind::Adt(_, GenericArgs::EMPTY))
            )
        })
        .ok_or("missing actual type")?;
    let arguments = fixture
        .facts
        .types
        .intern_arguments(&[Argument::Lifetime(fixture.region), Argument::Type(tag)])?;
    let enclosing = Instance {
        definition: fixture.owner,
        arguments,
    };
    let mut substitution = enclosing.substitution(&fixture.facts)?;
    let actual = canonical.substitute(&mut fixture.facts.types, &mut substitution)?;
    assert_eq!(
        fixture
            .facts
            .types
            .arguments(actual.arguments)
            .and_then(|mut args| args.next()),
        Some(Argument::Lifetime(fixture.region))
    );
    assert_eq!(fixture.facts.body_regions.len(), region_count);
    assert_eq!(fixture.facts.region_equalities.len(), equation_count);
    assert!(actual.result_type(&mut fixture.facts).is_err());
    Ok(())
}

#[test]
fn missing_equations_keep_source_identity_without_granting_caller_permission() -> Result {
    let mut fixture = fixture()?;
    let ranges = append(&mut fixture, 1);
    let index = ranges[0].start;
    let end = fixture.facts.region_equalities.len();
    canonicalize(
        &mut fixture.facts,
        fixture.owner,
        fixture.source,
        1,
        ranges,
        [end..end, end..end],
    )?;
    assert_eq!(selected(&fixture, index)?, fixture.original);
    assert!(fixture.original.result_type(&mut fixture.facts).is_err());
    Ok(())
}

#[test]
fn foreign_and_stale_equation_publication_cannot_change_dependency_rows() -> Result {
    let mut fixture = fixture()?;
    let ranges = append(&mut fixture, 1);
    let index = ranges[0].start;
    let before = fixture.facts.body_work.canonical_steps;
    let mut foreign = fixture.equations.clone();
    let position = foreign[0].start;
    fixture.facts.region_equalities[position].body.node = crate::syntax::ast::NodeId::new(
        fixture.facts.region_equalities[position].body.node.get() + 1,
    );
    assert!(
        canonicalize(
            &mut fixture.facts,
            fixture.owner,
            fixture.source,
            1,
            ranges.clone(),
            foreign.clone()
        )
        .is_err()
    );
    fixture.facts.region_equalities[position].body.node = crate::syntax::ast::NodeId::new(
        fixture.facts.region_equalities[position].body.node.get() - 1,
    );
    foreign[1] = position..position + 1;
    assert!(
        canonicalize(
            &mut fixture.facts,
            fixture.owner,
            fixture.source,
            1,
            ranges.clone(),
            foreign
        )
        .is_err()
    );
    let end = fixture.facts.region_equalities.len() + 1;
    assert!(
        canonicalize(
            &mut fixture.facts,
            fixture.owner,
            fixture.source,
            1,
            ranges.clone(),
            [end..end, end..end]
        )
        .is_err()
    );
    fixture.facts.body_regions[fixture.region.0 as usize - 1]
        .body
        .node = crate::syntax::ast::NodeId::new(
        fixture.facts.body_regions[fixture.region.0 as usize - 1]
            .body
            .node
            .get()
            + 1,
    );
    assert!(
        canonicalize(
            &mut fixture.facts,
            fixture.owner,
            fixture.source,
            1,
            ranges,
            fixture.equations.clone()
        )
        .is_err()
    );
    assert_eq!(selected(&fixture, index)?, fixture.original);
    assert!(fixture.facts.body_work.canonical_steps > before);
    Ok(())
}

#[test]
fn repeated_dependency_rows_have_linear_publication_work() -> Result {
    let mut previous = 0;
    for count in [8, 32, 128] {
        let mut fixture = fixture()?;
        let ranges = append(&mut fixture, count);
        let before = fixture.facts.body_work.canonical_steps;
        canonicalize(
            &mut fixture.facts,
            fixture.owner,
            fixture.source,
            1,
            ranges,
            fixture.equations.clone(),
        )?;
        let work = fixture.facts.body_work.canonical_steps - before;
        eprintln!("dependency rows={count} publication_work={work}");
        if previous != 0 {
            assert!(work < previous * 5);
        }
        previous = work;
    }
    Ok(())
}
