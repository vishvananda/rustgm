//! Actual body proofs distinguish deferred whole debts from independent nested formation.
use super::{Argument, Bound, Fixture, Kind, Predicate, RegionId, Result, TypeId};
use rustgm::{
    body::regions::{Contract, Contracts, Kind as ObligationKind, Obligation},
    resolve::unit::{FileId, NodeKey},
    syntax::ast::NodeId,
};

fn source(index: u32) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: NodeId(index),
    }
}
fn contract(f: &Fixture, ty: TypeId) -> Contract {
    Contract {
        source: source(1),
        signature: None,
        universals: 2,
        assumptions: Vec::new(),
        parameters: f.parameters.clone(),
        type_assumptions: Vec::new(),
        proofs: Vec::new(),
        obligations: vec![Obligation {
            source: source(2),
            kind: ObligationKind::ScopedWellFormed {
                ty,
                bound_regions: 1,
            },
        }],
    }
}
fn valid(f: &Fixture, c: Contract, expected: bool, at: u32) {
    let contracts = Contracts {
        bodies: vec![c],
        callables: &f.templates,
        nominal_requirements: &f.bounds,
    };
    let result = rustgm::borrow::regions::check(&f.types, &f.variances, &contracts);
    assert_eq!(result.is_ok(), expected, "{:?}", result.as_ref().err());
    if let Err(error) = result {
        assert_eq!(error.source, Some(source(at)));
        assert!(error.message.contains("does not outlive"), "{error}");
    }
}

#[test]
fn independent_nested_debts_survive_local_references_tuples_and_callable_binders() -> Result {
    for wrapper in 0..3 {
        for given in [false, true] {
            let mut f = Fixture::default();
            let t = f.parameter()?;
            let depth = if wrapper == 2 { 2 } else { 1 };
            let inner = f.reference(RegionId::bound(depth, 0)?, t)?;
            let local = f.reference(RegionId::bound(depth - 1, 0)?, inner)?;
            let root = match wrapper {
                0 => local,
                1 => f.tuple(&[local, t])?,
                _ => {
                    let parameters = f.types.intern_list(&[local])?;
                    f.types.intern(Kind::Function {
                        bound_regions: 1,
                        parameters,
                        result: TypeId::UNIT,
                        unsafe_: false,
                    })?
                }
            };
            let mut c = contract(&f, root);
            if given {
                c.type_assumptions.push(
                    Predicate {
                        ty: t,
                        bound: Bound::Outlives(RegionId::bound(0, 0)?),
                    }
                    .into(),
                );
            }
            valid(&f, c, given, 2);
        }
    }
    Ok(())
}

#[test]
fn mixed_local_referents_defer_whole_debts_and_grant_no_body_permissions() -> Result {
    for demand in [false, true] {
        let mut f = Fixture::default();
        let t = f.parameter()?;
        let local = f.reference(RegionId::bound(0, 0)?, TypeId::UNIT)?;
        let mixed = f.tuple(&[local, t])?;
        let root = f.reference(RegionId::bound(1, 0)?, mixed)?;
        let mut c = contract(&f, root);
        if demand {
            c.obligations.push(Obligation {
                source: source(3),
                kind: ObligationKind::TypeOutlives {
                    ty: t,
                    shorter: RegionId::bound(0, 0)?,
                },
            });
        }
        valid(&f, c, !demand, 3);
    }
    Ok(())
}

#[test]
fn opaque_output_debts_are_deferred_only_if_the_whole_projection_mentions_a_local() -> Result {
    for local in [false, true] {
        for given in [false, true] {
            let mut f = Fixture::default();
            let t = f.parameter()?;
            let input = RegionId::bound(u16::from(!local), 0)?;
            let output = f.projection(12, t, &[Argument::Lifetime(input)])?;
            let root = f.reference(RegionId::bound(1, 1)?, output)?;
            let mut c = contract(&f, root);
            if given {
                let assumed = f.projection(12, t, &[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
                c.type_assumptions.push(
                    Predicate {
                        ty: assumed,
                        bound: Bound::Outlives(RegionId::bound(0, 1)?),
                    }
                    .into(),
                );
            }
            valid(&f, c, local || given, 2);
        }
    }
    Ok(())
}

#[test]
fn repeated_debts_keep_their_first_real_source_and_do_not_cross_body_environments() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let root = f.reference(RegionId::bound(1, 0)?, t)?;
    let mut c = contract(&f, root);
    c.obligations.push(Obligation {
        source: source(3),
        kind: c.obligations[0].kind,
    });
    let mut given = c.clone();
    given.type_assumptions.push(
        Predicate {
            ty: t,
            bound: Bound::Outlives(RegionId::bound(0, 0)?),
        }
        .into(),
    );
    let contracts = Contracts {
        bodies: vec![given, c],
        callables: &f.templates,
        nominal_requirements: &f.bounds,
    };
    let result = rustgm::borrow::regions::check(&f.types, &f.variances, &contracts);
    let error = result
        .err()
        .ok_or("second body inherited first body's permission")?;
    assert_eq!(error.source, Some(source(2)));
    assert!(error.message.contains("does not outlive"), "{error}");
    Ok(())
}

#[test]
fn nominal_argument_bounds_defer_whole_mixed_arguments_but_keep_independent_ones() -> Result {
    use rustgm::types::{
        ItemId,
        aggregate::{self, Definitions, Repr, Variant},
        outlives::Bounds,
        parameters::{Parameter, ParameterKind},
    };
    for mixed in [false, true] {
        for given in [false, true] {
            let mut f = Fixture::default();
            let t = f.parameter()?;
            let formal = f.types.intern_parameter(Parameter {
                owner: ItemId(50),
                index: 1,
                kind: ParameterKind::Type { sized: true },
            })?;
            let formal_ty = f.types.intern(Kind::Parameter(formal))?;
            let field = f.reference(RegionId::bound(0, 0)?, formal_ty)?;
            let fields = f.types.intern_list(&[field])?;
            let mut definitions = Definitions::default();
            definitions.insert(
                ItemId(50),
                aggregate::Kind::Struct,
                Repr::default(),
                &[Variant {
                    fields,
                    discriminant: 0,
                }],
            )?;
            f.bounds = Bounds::infer(&mut f.types, &definitions, &[])?;
            let local = f.reference(RegionId::bound(0, 0)?, TypeId::UNIT)?;
            let actual = if mixed { f.tuple(&[local, t])? } else { t };
            let arguments = f.types.intern_arguments(&[
                Argument::Lifetime(RegionId::bound(1, 0)?),
                Argument::Type(actual),
            ])?;
            let nominal = f.types.intern(Kind::Adt(ItemId(50), arguments))?;
            let root = f.reference(RegionId::bound(0, 0)?, nominal)?;
            let mut c = contract(&f, root);
            if given {
                c.type_assumptions.push(
                    Predicate {
                        ty: t,
                        bound: Bound::Outlives(RegionId::bound(0, 0)?),
                    }
                    .into(),
                );
            }
            valid(&f, c, mixed || given, 2);
        }
    }
    Ok(())
}

#[test]
fn projected_self_and_type_arguments_keep_their_independent_nested_reference_debts() -> Result {
    for self_input in [false, true] {
        for given in [false, true] {
            let mut f = Fixture::default();
            let t = f.parameter()?;
            let inner = f.reference(RegionId::bound(1, 0)?, t)?;
            let local = f.reference(RegionId::bound(0, 0)?, inner)?;
            let root = if self_input {
                f.projection(12, local, &[Argument::Lifetime(RegionId::bound(0, 0)?)])?
            } else {
                f.projection(12, TypeId::UNIT, &[Argument::Type(local)])?
            };
            let mut c = contract(&f, root);
            if given {
                c.type_assumptions.push(
                    Predicate {
                        ty: t,
                        bound: Bound::Outlives(RegionId::bound(0, 0)?),
                    }
                    .into(),
                );
            }
            valid(&f, c, given, 2);
        }
    }
    Ok(())
}

#[test]
fn growing_prefixes_with_distinct_targets_finish_the_actual_body_proof() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let mut c = contract(&f, t);
    c.universals = 4096;
    c.obligations.clear();
    let mut root = t;
    for slot in 0..4096 {
        root = f.tuple(&[root, t])?;
        let ty = f.reference(RegionId::bound(1, slot)?, root)?;
        c.obligations.push(Obligation {
            source: source(u32::from(slot) + 2),
            kind: ObligationKind::ScopedWellFormed {
                ty,
                bound_regions: 0,
            },
        });
    }
    let mut paid = c.clone();
    paid.type_assumptions.push(
        Predicate {
            ty: t,
            bound: Bound::Outlives(RegionId::STATIC),
        }
        .into(),
    );
    valid(&f, paid.clone(), true, 0);
    // One unpaid debt has a unique source; the order in which several distinct
    // failing targets are diagnosed is not part of the formation contract.
    let other = f.parameter()?;
    let ty = f.reference(RegionId::bound(1, 0)?, other)?;
    paid.parameters.clone_from(&f.parameters);
    paid.obligations.push(Obligation {
        source: source(5000),
        kind: ObligationKind::ScopedWellFormed {
            ty,
            bound_regions: 0,
        },
    });
    valid(&f, paid, false, 5000);
    Ok(())
}

#[test]
fn promotion_keeps_the_first_source_of_an_unpaid_debt() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let root = f.reference(RegionId::bound(1, 0)?, t)?;
    let other = f.reference(RegionId::bound(1, 1)?, t)?;
    let mut c = contract(&f, root);
    c.obligations.push(Obligation {
        source: source(3),
        kind: ObligationKind::ScopedWellFormed {
            ty: other,
            bound_regions: 1,
        },
    });
    // Distinct structure revisits the original component after promotion.
    let prefix = f.tuple(&[t, t])?;
    let repeated = f.reference(RegionId::bound(1, 0)?, prefix)?;
    c.obligations.push(Obligation {
        source: source(4),
        kind: ObligationKind::ScopedWellFormed {
            ty: repeated,
            bound_regions: 1,
        },
    });
    c.type_assumptions.push(
        Predicate {
            ty: t,
            bound: Bound::Outlives(RegionId::bound(0, 1)?),
        }
        .into(),
    );
    valid(&f, c, false, 2);
    Ok(())
}
