//! Result ownership includes the captured formal arguments, not just member identity.
use super::*;
use crate::types::{GenericArgs, projection};

#[test]
fn quantified_results_belong_only_to_the_captured_complete_formal_parent()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(1);
    let member = ItemId(2);
    let mut header = header(&mut store, owner, 1)?;
    let ty = type_parameter(&mut store, owner, 2)?;
    header.parameters.push(Parameter {
        binding: DefId(3),
        argument: Argument::Type(ty),
        default: None,
    });
    store.declare_projection(
        member,
        projection::Schema {
            trait_item: owner,
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let mut retained = Vec::new();
    for (depth, slot, actual_type, expected) in [
        (1, 0, ty, true),
        (0, 0, ty, false),
        (1, 1, ty, false),
        (1, 0, TypeId::UNIT, false),
    ] {
        let application = predicate(
            &mut store,
            owner,
            header.self_type,
            &[lifetime(depth, slot)?, Argument::Type(actual_type)],
        )?;
        let Bound::Trait(trait_application) = application.bound else {
            return Err("missing parent");
        };
        let projection = store.intern(Kind::Projection(projection::Application {
            item: member,
            subject: header.self_type,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let value = clause(
            &mut store,
            1,
            Predicate {
                ty: projection,
                bound: Bound::Outlives(RegionId::bound(0, 0)?),
            },
        )?;
        assert_eq!(
            header.owned_result(&store, owner, value)?,
            expected.then_some(member)
        );
        header.predicates.push(Located {
            source: source(slot.into()),
            value,
        });
        retained.push((value, expected));
    }
    header.partition_results(&store, owner)?;
    assert_eq!(header.results.len(), 1);
    assert_eq!(header.predicates.len(), 3);
    for (value, owned) in retained {
        assert_eq!(header.results.iter().any(|(_, p)| p.value == value), owned);
        assert_eq!(header.predicates.iter().any(|p| p.value == value), !owned);
    }
    Ok(())
}
