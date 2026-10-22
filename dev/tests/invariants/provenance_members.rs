use crate::{
    resolve::{
        model::{Access, DefId},
        unit::{FileId, NodeKey},
    },
    sema::members::{Field, FieldId, Identity, Members, Name, Origin},
    syntax::{ast::NodeId, item::Shape},
    types::{ItemId, TypeId},
};

#[test]
fn source_and_library_locations_keep_distinct_lookup_authority() -> Result<(), String> {
    let source = NodeKey {
        file: FileId(1),
        node: NodeId::new(7),
    };
    let library = Origin::Library(source);
    assert_eq!(library.diagnostic(), Some(source));
    assert_eq!(library.source(), None);
    assert_eq!(Origin::Source(source).source(), Some(source));
    let mut members = Members::default();
    let source_id = members.insert(ItemId(3), 0, source, Shape::Record, &[])?;
    let library_id = members.insert_metadata(ItemId(4), 0, source, Shape::Record, &[])?;
    assert_eq!(members.at_node(source), Some(source_id));
    assert_eq!(members.variants[library_id.0 as usize].origin, library);
    assert_eq!(
        members.variants[source_id.0 as usize].origin,
        Origin::Source(source)
    );
    assert!(
        members
            .insert(ItemId(5), 0, source, Shape::Record, &[])
            .is_err()
    );
    Ok(())
}

fn fields(members: &mut Members, owner: DefId) -> Result<[Field; 2], String> {
    let first = Name::Named(members.names.intern("first")?);
    let second = Name::Named(members.names.intern("second")?);
    Ok([
        Field {
            origin: Origin::External(owner),
            name: first,
            ty: TypeId::UNIT,
            access: Access::Public,
        },
        Field {
            origin: Origin::External(owner),
            name: second,
            ty: TypeId::UNIT,
            access: Access::ExternalPrivate,
        },
    ])
}

#[test]
fn external_members_keep_owner_and_order_without_source_identity() -> Result<(), String> {
    let mut members = Members::default();
    for owner in [DefId(9), DefId(11)] {
        let fields = fields(&mut members, owner)?;
        let id = members.insert_external(ItemId(owner.0), &fields)?;
        let variant = members.variants[id.0 as usize];
        assert_eq!(variant.origin.diagnostic(), None);
        assert_eq!(variant.origin.source(), None);
        assert_eq!(variant.origin.external_owner(), Some(owner));
        assert!(variant.origin.require_diagnostic().is_err());
        for ordinal in 0..2 {
            let field = members
                .field(id, fields[ordinal as usize].name)
                .ok_or("missing field")?;
            assert_eq!(
                members.identity(field),
                Some(Identity {
                    owner,
                    variant: 0,
                    field: ordinal
                })
            );
            let field = members.fields[field.0 as usize];
            assert_eq!(field.origin, variant.origin);
            assert_eq!(
                field.access,
                if ordinal == 0 {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                }
            );
        }
    }
    assert!(members.identity(FieldId(u32::MAX)).is_none());
    Ok(())
}

#[test]
fn external_publication_rejects_foreign_owners_and_duplicate_rows() -> Result<(), String> {
    let mut members = Members::default();
    let foreign = fields(&mut members, DefId(8))?;
    assert!(members.insert_external(ItemId(7), &foreign).is_err());
    assert!(members.fields.is_empty());
    assert!(members.variants.is_empty());
    let valid = fields(&mut members, DefId(7))?;
    let mut unnamed = valid;
    unnamed[0].name = Name::Index(0);
    assert!(members.insert_external(ItemId(7), &unnamed).is_err());
    let mut duplicate = valid;
    duplicate[1].name = duplicate[0].name;
    assert!(members.insert_external(ItemId(7), &duplicate).is_err());
    assert!(members.fields.is_empty());
    assert!(members.variants.is_empty());
    members.insert_external(ItemId(7), &valid)?;
    assert!(members.insert_external(ItemId(7), &valid).is_err());
    assert_eq!(members.fields.len(), 2);
    assert_eq!(members.variants.len(), 1);
    let source = NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    };
    assert!(
        members
            .insert(ItemId(8), 0, source, Shape::Record, &valid)
            .is_err()
    );
    assert!(
        members
            .insert_metadata(ItemId(8), 0, source, Shape::Record, &valid)
            .is_err()
    );
    assert_eq!(members.variants.len(), 1);
    Ok(())
}
