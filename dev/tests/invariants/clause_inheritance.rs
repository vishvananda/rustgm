//! Declaration closure must preserve quantifiers and distinguish assumptions from requirements.
#[path = "clause_inheritance/formation.rs"]
mod formation;
#[path = "clause_inheritance/ownership.rs"]
mod ownership;
use super::{Contracts, Header};
use crate::support::hash::HashSet;
use crate::{
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    sema::generics::{Generics, Parameter, Predicate as Located},
    syntax::ast::NodeId,
    types::{
        ItemId, Kind, RegionId, Store, TypeId,
        arguments::Argument,
        parameters::{
            Parameter as Declaration, ParameterKind,
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};

fn source(index: u32) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: NodeId(index),
    }
}
fn type_parameter(store: &mut Store, owner: ItemId, index: u32) -> Result<TypeId, &'static str> {
    let id = store.intern_parameter(Declaration {
        owner,
        index,
        kind: ParameterKind::Type { sized: false },
    })?;
    store.intern(Kind::Parameter(id))
}
fn header(store: &mut Store, owner: ItemId, count: u16) -> Result<Header, &'static str> {
    let self_type = type_parameter(store, owner, u32::from(count))?;
    let parameters = (0..count)
        .map(|slot| {
            Ok(Parameter {
                anonymous: false,
                binding: DefId(u32::from(slot)),
                default: None,
                argument: Argument::Lifetime(RegionId::bound(0, slot)?),
            })
        })
        .collect::<Result<_, &'static str>>()?;
    Ok(Header {
        self_type,
        parameters,
        self_bounds: HashSet::default(),
        predicates: Vec::new(),
        results: Vec::new(),
        regions: Vec::new(),
    })
}
fn predicate(
    store: &mut Store,
    item: ItemId,
    ty: TypeId,
    args: &[Argument],
) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(args)?;
    let application = store.intern_trait_application(Application { item, arguments })?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(application),
    })
}
fn clause(store: &mut Store, count: u32, predicate: Predicate) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}
fn lifetime(depth: u16, slot: u16) -> Result<Argument, &'static str> {
    Ok(Argument::Lifetime(RegionId::bound(depth, slot)?))
}
fn implied(header: &mut Header, value: Clause, index: u32) {
    header.self_bounds.insert(source(index));
    header.predicates.push(Located {
        source: source(index),
        value,
    });
}

#[test]
fn nested_supertraits_compose_independent_slots_and_preserve_outer_captures()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let (a, b, c) = (ItemId(1), ItemId(2), ItemId(3));
    let mut first = header(&mut store, a, 1)?;
    let mut second = header(&mut store, b, 2)?;
    let third = header(&mut store, c, 3)?;
    let value = predicate(
        &mut store,
        b,
        first.self_type,
        &[lifetime(1, 0)?, lifetime(0, 0)?],
    )?;
    implied(&mut first, clause(&mut store, 1, value)?, 1);
    let value = predicate(
        &mut store,
        c,
        second.self_type,
        &[lifetime(1, 1)?, lifetime(0, 0)?, lifetime(1, 0)?],
    )?;
    implied(&mut second, clause(&mut store, 1, value)?, 2);
    let mut contracts = Contracts::default();
    for (item, header) in [(a, first), (b, second), (c, third)] {
        contracts.declare(item, header)?;
    }
    contracts.check_cycles(&store).map_err(|(_, error)| error)?;
    let owner = DefId(10);
    let parameter = type_parameter(&mut store, ItemId(owner.0), 0)?;
    let subject = store.intern(Kind::Reference {
        referent: parameter,
        region: RegionId::bound(1, 2)?,
        mutable: false,
    })?;
    let input = predicate(&mut store, a, subject, &[lifetime(0, 0)?])?;
    let input = clause(&mut store, 1, input)?;
    let middle = predicate(&mut store, b, subject, &[lifetime(0, 0)?, lifetime(0, 1)?])?;
    let middle = clause(&mut store, 2, middle)?;
    let last = predicate(
        &mut store,
        c,
        subject,
        &[lifetime(0, 1)?, lifetime(0, 2)?, lifetime(0, 0)?],
    )?;
    let last = clause(&mut store, 3, last)?;
    let mut generics = Generics::default();
    generics.traits = Some(Box::new(contracts));
    generics.parameters.insert(
        owner,
        vec![Parameter {
            anonymous: false,
            binding: DefId(11),
            argument: Argument::Type(parameter),
            default: None,
        }],
    );
    generics.predicates.insert(
        owner,
        vec![Located {
            source: source(3),
            value: input,
        }],
    );
    generics.complete_environment(&mut store, owner)?;
    let expected = HashSet::from_iter([input, middle, last]);
    assert_eq!(
        generics.environments[&owner]
            .trait_clauses()
            .collect::<HashSet<_>>(),
        expected
    );
    assert_eq!(generics.declared_predicates_in_scope(owner).count(), 1);
    let repeated = generics.elaborated_environment(&mut store, owner)?;
    assert_eq!(repeated.trait_clauses().collect::<HashSet<_>>(), expected);
    assert_eq!(generics.predicates[&owner].len(), 3);
    assert_eq!(
        generics
            .parameter_environment(&store, DefId(20))?
            .trait_clauses()
            .count(),
        0
    );
    Ok(())
}

#[test]
fn quantified_non_self_contracts_remain_requirements_without_granting_caller_permission()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let (a, b) = (ItemId(1), ItemId(2));
    let mut first = header(&mut store, a, 1)?;
    let other = type_parameter(&mut store, a, 2)?;
    first.parameters.push(Parameter {
        anonymous: false,
        binding: DefId(3),
        argument: Argument::Type(other),
        default: None,
    });
    let value = predicate(&mut store, b, other, &[lifetime(1, 0)?, lifetime(0, 0)?])?;
    let requirement = clause(&mut store, 1, value)?;
    // Even the same written source cannot turn a different subject into Self.
    implied(&mut first, requirement, 1);
    assert!(!first.implies_self(&store, &first.predicates[0])?);
    let second = header(&mut store, b, 2)?;
    let mut contracts = Contracts::default();
    contracts.declare(a, first)?;
    contracts.declare(b, second)?;
    let input = predicate(
        &mut store,
        a,
        TypeId::UNIT,
        &[lifetime(0, 0)?, Argument::Type(TypeId::UNIT)],
    )?;
    let found = contracts.requirements(&mut store, input)?;
    assert!(found.implied.is_empty());
    let expected = predicate(
        &mut store,
        b,
        TypeId::UNIT,
        &[lifetime(1, 0)?, lifetime(0, 0)?],
    )?;
    let expected = clause(&mut store, 1, expected)?;
    assert_eq!(found.required, [expected]);
    let mut generics = Generics::default();
    generics.traits = Some(Box::new(contracts));
    let input = clause(&mut store, 1, input)?;
    generics.predicates.insert(
        DefId(10),
        vec![Located {
            source: source(2),
            value: input,
        }],
    );
    let environment = generics.elaborated_environment(&mut store, DefId(10))?;
    assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), [input]);
    Ok(())
}

#[test]
fn quantified_supertrait_cycles_are_rejected_before_argument_growth() -> Result<(), &'static str> {
    let mut store = Store::default();
    let (a, b) = (ItemId(1), ItemId(2));
    let mut first = header(&mut store, a, 1)?;
    let mut second = header(&mut store, b, 1)?;
    for (parent, item) in [(&mut first, b), (&mut second, a)] {
        let value = predicate(&mut store, item, parent.self_type, &[lifetime(0, 0)?])?;
        implied(parent, clause(&mut store, 1, value)?, 1);
    }
    let mut contracts = Contracts::default();
    contracts.declare(a, first)?;
    contracts.declare(b, second)?;
    assert!(matches!(
        contracts.check_cycles(&store),
        Err((_, "cycle in supertrait requirements"))
    ));
    Ok(())
}

#[test]
fn distinct_supertrait_applications_share_structural_work() -> Result<(), &'static str> {
    let mut previous = 0;
    for count in [16, 32, 64, 128, 256, 512] {
        let mut store = Store::default();
        let (a, b) = (ItemId(1), ItemId(2));
        let mut first = header(&mut store, a, 1)?;
        let second = header(&mut store, b, 2)?;
        let value = predicate(
            &mut store,
            b,
            first.self_type,
            &[lifetime(1, 0)?, lifetime(0, 0)?],
        )?;
        implied(&mut first, clause(&mut store, 1, value)?, 1);
        let mut contracts = Contracts::default();
        contracts.declare(a, first)?;
        contracts.declare(b, second)?;
        let mut inputs = Vec::new();
        let mut tail = TypeId::UNIT;
        for index in 0..count {
            tail = store.intern(Kind::Reference {
                referent: tail,
                region: RegionId::bound(0, 0)?,
                mutable: false,
            })?;
            let value = predicate(&mut store, a, tail, &[lifetime(0, 0)?])?;
            inputs.push(Located {
                source: source(index),
                value: clause(&mut store, 1, value)?,
            });
        }
        let initial = store.len();
        let mut generics = Generics::default();
        generics.traits = Some(Box::new(contracts));
        generics.predicates.insert(DefId(10), inputs);
        let environment = generics.elaborated_environment(&mut store, DefId(10))?;
        assert_eq!(environment.trait_clauses().count(), 2 * count as usize);
        let steps = generics.traits.as_ref().ok_or("missing contracts")?.steps;
        assert!(steps <= 24 * count as usize + 16);
        assert!(previous == 0 || steps <= 2 * previous + 16);
        assert!(store.len() - initial <= count as usize);
        previous = steps;
        println!(
            "supertrait-composition count={count} contract_steps={steps} types={}",
            store.len() - initial
        );
    }
    Ok(())
}
