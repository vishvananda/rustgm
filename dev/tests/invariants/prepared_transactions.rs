//! Failed/stale source-free batches cannot acquire another member or shape owner.
use crate::{
    resolve::model::{Access, DefId},
    sema::members::{Field, Members, Name, Origin},
    types::{
        ItemId, Store, TypeId,
        aggregate::{Definitions, Kind, Repr, Variant},
    },
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
fn field(members: &mut Members, owner: u32) -> std::result::Result<Field, &'static str> {
    Ok(Field {
        origin: Origin::External(DefId(owner)),
        name: Name::Named(members.names.intern("value")?),
        ty: TypeId::UNIT,
        access: Access::Public,
    })
}
#[test]
fn external_member_batches_reject_failure_duplicate_owner_stale_and_foreign_destinations() -> Result
{
    let mut members = Members::default();
    let value = field(&mut members, 7)?;
    assert!(
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![value])], Vec::new(), |_, _| Err(
                "unpaid"
            ))
            .is_err()
    );
    assert!(members.variants.is_empty() && members.fields.is_empty());
    assert!(members.insert_external(ItemId(7), &[value, value]).is_err());
    assert!(members.variants.is_empty() && members.fields.is_empty());
    let foreign = field(&mut members, 8)?;
    assert!(
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![foreign])], Vec::new(), |_, _| Ok(()))
            .is_err()
    );
    let plan =
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![value])], Vec::new(), |_, _| Ok(()))?;
    assert!(plan.ready(&mut Members::default()).is_err());
    let plan =
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![value])], Vec::new(), |_, _| Ok(()))?;
    members.insert_external(ItemId(8), &[foreign])?;
    assert!(plan.ready(&mut members).is_err());
    let plan =
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![value])], Vec::new(), |_, _| Ok(()))?;
    plan.ready(&mut members)?.install();
    assert_eq!(members.fields.len(), 2);
    assert!(
        members
            .prepare_external_nominals(vec![(ItemId(7), vec![value])], Vec::new(), |_, _| Ok(()))
            .is_err()
    );
    Ok(())
}
#[test]
fn external_shape_batches_require_paid_unique_current_destination_frontiers() -> Result {
    let mut shapes = Definitions::default();
    let mut types = Store::default();
    let fields = types.intern_list(&[])?;
    let row = (ItemId(7), Repr::default(), fields);
    assert!(
        shapes
            .prepare_imported_nominals(&types, &[row], &[], None, |_, _| Err("unpaid"))
            .is_err()
    );
    assert!(shapes.get(ItemId(7)).is_none());
    assert!(
        shapes
            .prepare_imported_nominals(&types, &[row, row], &[], None, |_, _| Ok(()))
            .is_err()
    );
    let plan = shapes.prepare_imported_nominals(&types, &[row], &[], None, |_, _| Ok(()))?;
    assert!(plan.ready(&mut Definitions::default()).is_err());
    let plan = shapes.prepare_imported_nominals(&types, &[row], &[], None, |_, _| Ok(()))?;
    shapes.insert(
        ItemId(8),
        Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert!(plan.ready(&mut shapes).is_err());
    let plan = shapes.prepare_imported_nominals(&types, &[row], &[], None, |_, _| Ok(()))?;
    plan.ready(&mut shapes)?.install();
    assert_eq!(shapes.iter().count(), 2);
    Ok(())
}
