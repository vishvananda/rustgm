//! Origin rows are structural data; these adversaries never acquire execution proof.
use super::*;
use crate::{
    body::ir::{Owner, Value},
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    syntax::ast::NodeId,
    types::{GenericArgs, ItemId, callable::Convention},
};

fn fixture() -> Result<(Store, Body, TypeId, TypeId), &'static str> {
    let mut types = Store::default();
    let parameters = types.intern_list(&[])?;
    let signature = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let mut item = |id| {
        types.intern(Kind::FunctionItem {
            item: ItemId(id),
            signature,
            arguments: GenericArgs::EMPTY,
        })
    };
    let first = item(1)?;
    let second = item(2)?;
    let body = Body::new(
        Owner::Function {
            definition: DefId(1),
            signature,
            execution: None,
        },
        NodeKey {
            file: FileId(0),
            node: NodeId::checked(1).ok_or("source node")?,
        },
        TypeId::UNIT,
    );
    Ok((types, body, first, second))
}

#[test]
fn indexed_chains_and_shared_prefixes_are_linear_cold_and_constant_warm() -> Result<(), &'static str>
{
    assert_eq!(std::mem::size_of::<ValueId>(), 4);
    for depth in [1_u32, 64, 2048] {
        let (types, mut body, ty, _) = fixture()?;
        body.values.push(Value {
            ty,
            kind: Rvalue::Function(DefId(1)),
        });
        for id in 0..depth {
            body.values.push(Value {
                ty,
                kind: Rvalue::Constrain(ValueId(id)),
            });
        }
        for _ in 0..depth {
            body.values.push(Value {
                ty,
                kind: Rvalue::Constrain(ValueId(depth)),
            });
        }
        let other = body.header();
        let mut origins = Origins::new(&body);
        assert!(origins.origin(&types, &body, ValueId(0)).is_err());
        let mut charged = 0;
        origins.prepare(&types, |count| {
            charged += count;
            Ok(())
        })?;
        assert_eq!(charged, body.values.len());
        assert_eq!(origins.attempted, body.values.len());
        assert_eq!(origins.roots.len(), body.values.len());
        assert_eq!(origins.storage, 4 * origins.roots.capacity());
        assert!(origins.storage >= 4 * body.values.len());
        assert!(origins.storage <= WORK_LIMIT * 4);
        for id in 0..u32::try_from(body.values.len()).map_err(|_| "value count")? {
            assert_eq!(origins.origin(&types, &body, ValueId(id))?, ValueId(0));
        }
        assert!(origins.origin(&types, &other, ValueId(0)).is_err());
        origins.prepare(&types, |count| {
            assert_eq!(count, 0);
            Ok(())
        })?;
        assert_eq!(origins.attempted, charged);
        println!(
            "depth={depth} rows={charged} bytes={} warm_extra_work=0",
            origins.storage
        );
        assert!(
            origins
                .prepare(&types, |_| Err("failed cumulative owner"))
                .is_err()
        );
        assert!(origins.origin(&types, &body, ValueId(0)).is_err());
        assert!(origins.prepare(&types, |_| Ok(())).is_err());
        assert_eq!(origins.attempted, charged);
    }
    Ok(())
}

#[test]
fn invalid_origin_edges_and_failed_budgets_never_publish_partial_or_warm_rows()
-> Result<(), &'static str> {
    for variant in 0..4 {
        let (types, mut body, first, second) = fixture()?;
        body.values.push(Value {
            ty: first,
            kind: Rvalue::Function(DefId(1)),
        });
        body.values.push(Value {
            ty: first,
            kind: Rvalue::Constrain(ValueId(0)),
        });
        match variant {
            0 => body.values[0].kind = Rvalue::Constrain(ValueId(0)),
            1 => body.values[0].kind = Rvalue::Constrain(ValueId(1)),
            2 => body.values[1].ty = second,
            _ => body.values[0].ty = TypeId::UNIT,
        }
        let mut origins = Origins::new(&body);
        assert!(origins.prepare(&types, |_| Ok(())).is_err());
        assert_eq!(origins.attempted, 2);
        assert!(origins.roots.is_empty());
        assert!(origins.origin(&types, &body, ValueId(0)).is_err());
        assert!(origins.prepare(&types, |_| Ok(())).is_err());
        assert_eq!(origins.attempted, 2);
    }
    for external in [false, true] {
        let (types, mut body, ty, _) = fixture()?;
        body.values.push(Value {
            ty,
            kind: Rvalue::Function(DefId(1)),
        });
        let mut origins = Origins::new(&body);
        if !external {
            origins.attempted = WORK_LIMIT;
        }
        let mut charged = 0;
        assert!(
            origins
                .prepare(&types, |count| {
                    charged += count;
                    if external {
                        Err("cumulative dependency work budget exceeded")
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        assert_eq!(charged, 1);
        assert_eq!(
            origins.roots.capacity(),
            0,
            "budget rejection precedes allocation"
        );
        let attempted = origins.attempted;
        assert!(origins.prepare(&types, |_| Ok(())).is_err());
        assert_eq!(origins.attempted, attempted);
        assert!(origins.origin(&types, &body, ValueId(0)).is_err());
    }
    Ok(())
}

#[test]
fn actual_capacity_is_charged_and_an_excess_payload_is_released() -> Result<(), &'static str> {
    let (types, mut body, ty, _) = fixture()?;
    body.values.push(Value {
        ty,
        kind: Rvalue::Function(DefId(1)),
    });
    let mut origins = Origins::new(&body);
    origins
        .roots
        .try_reserve_exact(WORK_LIMIT + 1)
        .map_err(|_| "capacity opponent allocation")?;
    assert_eq!(
        origins.prepare(&types, |_| Ok(())),
        Err("callable value origin retained payload budget exceeded")
    );
    assert!(origins.storage > WORK_LIMIT * 4);
    assert_eq!(origins.roots.capacity(), 0);
    assert_eq!(origins.attempted, 1);
    assert!(origins.origin(&types, &body, ValueId(0)).is_err());
    assert!(origins.prepare(&types, |_| Ok(())).is_err());
    Ok(())
}

#[test]
fn physical_local_lineage_is_demand_only_paid_and_invalidated_by_aliases_or_multiple_stores()
-> Result<(), &'static str> {
    use crate::body::ir::{Action, Local, LocalId, LocalKind, Place, Statement};
    indexed_chains_and_shared_prefixes_are_linear_cold_and_constant_warm()?;
    invalid_origin_edges_and_failed_budgets_never_publish_partial_or_warm_rows()?;
    actual_capacity_is_charged_and_an_excess_payload_is_released()?;
    for width in [1_u32, 32, 128] {
        for opponent in 0..5 {
            let (types, mut body, ty, _) = fixture()?;
            body.values.push(Value {
                ty,
                kind: Rvalue::Function(DefId(1)),
            });
            for previous in 0..width {
                body.values.push(Value {
                    ty,
                    kind: Rvalue::Constrain(ValueId(previous)),
                });
            }
            let source = ValueId(width);
            body.locals.push(Local {
                scope: crate::body::scopes::Id(0),
                storage_parent: None,
                ty,
                kind: LocalKind::Binding,
                definition: None,
                mutable: true,
                source: body.root_site(),
            });
            body.statements.push(Statement {
                source: body.root_site(),
                action: Action::Store {
                    place: Place::Local(LocalId(0)),
                    value: source,
                },
            });
            let read = ValueId(u32::try_from(body.values.len()).map_err(|_| "read identity")?);
            body.values.push(Value {
                ty,
                kind: Rvalue::Read(Place::Local(LocalId(0))),
            });
            match opponent {
                1 => body.values.push(Value {
                    ty: TypeId::UNIT,
                    kind: Rvalue::Address(Place::Local(LocalId(0))),
                }),
                2 => body.values.push(Value {
                    ty: TypeId::UNIT,
                    kind: Rvalue::Borrow {
                        place: Place::Local(LocalId(0)),
                        kind: crate::body::ir::BorrowKind::Immediate,
                    },
                }),
                3 => body.statements.push(Statement {
                    source: body.root_site(),
                    action: Action::Store {
                        place: Place::Local(LocalId(0)),
                        value: source,
                    },
                }),
                4 => body.statements.push(Statement {
                    source: body.root_site(),
                    action: Action::Store {
                        place: Place::Temporary(source),
                        value: source,
                    },
                }),
                _ => {}
            }
            let mut origins = Origins::new(&body);
            origins.prepare(&types, |_| Ok(()))?;
            assert_eq!(origins.roots.len(), body.values.len());
            assert!(origins.transport_origin(&body, read).is_err());
            let old = origins.origin(&types, &body, source)?;
            origins.prepare_transport(&types)?;
            let prepared = origins.attempted;
            assert_eq!(origins.roots.len(), 2 * body.values.len());
            assert_eq!(origins.origin(&types, &body, source)?, old);
            assert_eq!(
                origins.transport_origin(&body, read)?,
                if opponent == 0 { source } else { read }
            );
            assert_eq!(origins.attempted, prepared + 1);
            origins.prepare_transport(&types)?;
            assert_eq!(origins.attempted, prepared + 1);
            let other = body.header();
            assert!(origins.transport_origin(&other, read).is_err());
            eprintln!(
                "physical-lineage width={width} opponent={opponent} work={prepared} bytes={}",
                origins.storage
            );
            origins.attempted = WORK_LIMIT;
            assert!(origins.transport_origin(&body, read).is_err());
            assert!(origins.origin(&types, &body, source).is_err());
        }
    }
    Ok(())
}
