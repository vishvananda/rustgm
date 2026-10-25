//! Formation metadata is not a proof, and deferral is specific to an input's binder.
use super::{
    Argument, Bound, Clause, Contracts, DefId, Header, ItemId, Kind, Located, Parameter, Predicate,
    RegionId, Store, TypeId, clause, header, predicate, source, type_parameter,
};
use crate::sema::traits::formation::Formation;
use std::sync::Arc;

type TestResult = Result<(), &'static str>;
const BYTE: TypeId = TypeId::primitive(crate::types::Primitive::U8);

fn with_type(
    store: &mut Store,
    owner: ItemId,
    lifetimes: u16,
) -> Result<(Header, TypeId), &'static str> {
    let mut value = header(store, owner, lifetimes)?;
    let ty = type_parameter(store, owner, u32::from(lifetimes) + 1)?;
    value.parameters.push(Parameter {
        anonymous: false,
        binding: DefId(owner.0 + 100),
        default: None,
        argument: Argument::Type(ty),
    });
    Ok((value, ty))
}
fn reference(store: &mut Store, depth: u16, referent: TypeId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(depth, 0)?,
        referent,
    })
}
fn requirement(header: &mut Header, clause: Clause) -> Result<(), &'static str> {
    header.source_inputs_mut()?.predicates.push(Located {
        source: source(20),
        value: clause,
    });
    Ok(())
}

#[test]
fn formation_defers_whole_local_subjects_but_keeps_independent_arguments() -> TestResult {
    let mut store = Store::default();
    let item = ItemId(1);
    let (header, _) = with_type(&mut store, item, 0)?;
    let mut contracts = Contracts::default();
    contracts.declare(item, header)?;
    let local = reference(&mut store, 0, BYTE)?;
    let captured = reference(&mut store, 1, BYTE)?;
    let tuple = store.intern_list(&[BYTE, local])?;
    let tuple = store.intern(Kind::Tuple(tuple))?;
    let input = predicate(&mut store, item, tuple, &[Argument::Type(captured)])?;
    let ordinary = contracts.formation(&mut store, input.into())?;
    assert_eq!(ordinary.subject(), Some(tuple));
    assert_eq!(ordinary.declaration(), Some(input));
    assert_eq!(ordinary.types(), &[captured]);
    assert!(contracts.formation.is_none());
    let value = clause(&mut store, 1, input)?;
    let formed = contracts.formation(&mut store, value)?;
    assert_eq!(formed.subject(), None);
    assert_eq!(formed.declaration(), None);
    assert_eq!(formed.types(), &[local]);
    // The lowered capture has the same representation as the previous local,
    // but belongs to the caller now. Cached metadata confers no permissions.
    let reverse = predicate(&mut store, item, captured, &[Argument::Type(tuple)])?;
    let reverse = clause(&mut store, 1, reverse)?;
    let reverse = contracts.formation(&mut store, reverse)?;
    assert_eq!(reverse.subject(), Some(local));
    assert_eq!(reverse.declaration(), None);
    assert!(reverse.types().is_empty());
    let again = contracts.formation(&mut store, input.into())?;
    assert_eq!(again.subject(), Some(tuple));
    assert_eq!(again.types(), &[captured]);
    Ok(())
}

#[test]
fn formation_keeps_nested_quantifiers_and_lifts_only_independent_requirements() -> TestResult {
    let mut store = Store::default();
    let item = ItemId(1);
    let (mut header, parameter) = with_type(&mut store, item, 2)?;
    for slot in [0, 1] {
        let ordinary = predicate(
            &mut store,
            ItemId(2),
            parameter,
            &[Argument::Lifetime(RegionId::bound(0, slot)?)],
        )?;
        requirement(&mut header, ordinary.into())?;
        let inner = predicate(
            &mut store,
            ItemId(3),
            parameter,
            &[
                Argument::Lifetime(RegionId::bound(0, 0)?),
                Argument::Lifetime(RegionId::bound(1, slot)?),
            ],
        )?;
        requirement(&mut header, clause(&mut store, 1, inner)?)?;
    }
    header.regions = vec![
        (RegionId::bound(0, 0)?, RegionId::bound(0, 1)?),
        (RegionId::bound(0, 1)?, RegionId::STATIC),
    ];
    let mut contracts = Contracts::default();
    contracts.declare(item, header)?;
    let application = predicate(
        &mut store,
        item,
        BYTE,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 0)?),
            Argument::Type(BYTE),
        ],
    )?;
    let value = clause(&mut store, 1, application)?;
    let formed = contracts.formation(&mut store, value)?;
    let ordinary = predicate(
        &mut store,
        ItemId(2),
        BYTE,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let inner = predicate(
        &mut store,
        ItemId(3),
        BYTE,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 0)?),
        ],
    )?;
    let inner = clause(&mut store, 1, inner)?;
    assert_eq!(formed.required(), &[ordinary.into(), inner]);
    assert_eq!(
        formed.regions(),
        &[(RegionId::bound(0, 0)?, RegionId::STATIC)]
    );
    assert_eq!(
        contracts
            .requirements(&mut store, application)?
            .required
            .len(),
        4
    );
    Ok(())
}

#[test]
fn empty_and_nonempty_binders_preserve_captured_inputs_and_nested_callable_locals() -> TestResult {
    let mut store = Store::default();
    let item = ItemId(1);
    let (header, _) = with_type(&mut store, item, 0)?;
    let mut contracts = Contracts::default();
    contracts.declare(item, header)?;
    let inner_local = reference(&mut store, 0, BYTE)?;
    let outer_capture = reference(&mut store, 2, BYTE)?;
    let parameters = store.intern_list(&[inner_local, outer_capture])?;
    let function = store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let lifted_capture = reference(&mut store, 1, BYTE)?;
    let parameters = store.intern_list(&[inner_local, lifted_capture])?;
    let expected = store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let application = predicate(&mut store, item, function, &[Argument::Type(function)])?;
    for count in [0, 1] {
        let value = clause(&mut store, count, application)?;
        let formed = contracts.formation(&mut store, value)?;
        assert_eq!(formed.subject(), Some(expected));
        assert_eq!(formed.types(), &[expected]);
    }
    Ok(())
}

#[test]
fn formation_caches_are_invalidated_with_their_trait_header_epoch() -> TestResult {
    let mut store = Store::default();
    let item = ItemId(1);
    let (header, parameter) = with_type(&mut store, item, 0)?;
    let mut contracts = Contracts::default();
    contracts.declare(item, header)?;
    let application = predicate(&mut store, item, BYTE, &[Argument::Type(BYTE)])?;
    let value = clause(&mut store, 1, application)?;
    let first = contracts.formation(&mut store, value)?;
    assert!(first.required().is_empty());
    let before = contracts.steps;
    let again = contracts.formation(&mut store, value)?;
    assert_eq!(contracts.steps, before);
    let (Formation::Quantified(first), Formation::Quantified(again)) = (first, again) else {
        return Err("missing quantified formation");
    };
    assert!(Arc::ptr_eq(&first, &again));
    contracts.complete_bindings(
        &store,
        vec![(
            item,
            vec![Located {
                source: source(21),
                value: Predicate {
                    ty: parameter,
                    bound: Bound::Copy,
                }
                .into(),
            }],
        )],
    )?;
    assert!(contracts.formation.is_none());
    let changed = contracts.formation(&mut store, value)?;
    assert_eq!(
        changed.required(),
        &[Predicate {
            ty: BYTE,
            bound: Bound::Copy
        }
        .into()]
    );
    contracts.steps = 1_000_001;
    assert!(contracts.formation(&mut store, value).is_err());
    Ok(())
}

#[test]
fn distinct_formation_queries_share_growing_captured_type_tails() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512] {
        let mut store = Store::default();
        let item = ItemId(1);
        let (mut header, parameter) = with_type(&mut store, item, 0)?;
        requirement(
            &mut header,
            Predicate {
                ty: parameter,
                bound: Bound::Copy,
            }
            .into(),
        )?;
        let mut contracts = Contracts::default();
        contracts.declare(item, header)?;
        let mut tail = BYTE;
        let mut inputs = Vec::new();
        for _ in 0..width {
            tail = reference(&mut store, 1, tail)?;
            let input = predicate(&mut store, item, BYTE, &[Argument::Type(tail)])?;
            inputs.push(clause(&mut store, 1, input)?);
        }
        let before = store.len();
        for &input in &inputs {
            let formed = contracts.formation(&mut store, input)?;
            assert_eq!(formed.required().len(), 1);
            assert_eq!(formed.types().len(), 1);
            assert!(
                matches!(store.kind(formed.types()[0]), Some(Kind::Reference { region, .. })
                if region == RegionId::bound(0, 0)?)
            );
        }
        let steps = contracts.steps;
        let types = store.len() - before;
        println!("formation width={width} steps={steps} types={types}");
        assert!(steps <= 128 * width + 128, "{width}: {steps}");
        assert!(types <= 2 * width + 32, "{width}: {types}");
        for input in inputs {
            let _ = contracts.formation(&mut store, input)?;
        }
        assert_eq!(contracts.steps, steps);
        assert_eq!(store.len() - before, types);
        observations.push((steps, types));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].0 <= 2 * pair[0].0 + 128, "{pair:?}");
        assert!(pair[1].1 <= 2 * pair[0].1 + 32, "{pair:?}");
    }
    Ok(())
}
