use crate::{
    expand::marks::{ExpansionId, ItemContext, Marks, SyntaxContextId, Transparency},
    resolve::{
        model::{Access, DefId},
        unit::{FileId, NodeKey},
    },
    sema::members::{Field, FieldId, Identity, Members, Name, Origin},
    syntax::{ast::NodeId, item::Shape},
    types::{ItemId, TypeId},
};
#[path = "provenance_aggregates.rs"]
mod uses;

fn scoped_contexts() -> Result<(Marks, [ItemContext; 2]), String> {
    let mut marks = Marks::default();
    let mut contexts = [ItemContext::ROOT; 2];
    for (expansion, context) in (1..).zip(&mut contexts) {
        let raw = marks.apply(
            SyntaxContextId::ROOT,
            ExpansionId(expansion),
            Transparency::Opaque,
            SyntaxContextId::ROOT,
        )?;
        *context = marks.item(raw)?;
    }
    Ok((marks, contexts))
}

fn source_key(node: u32) -> NodeKey {
    NodeKey {
        file: FileId(1),
        node: NodeId::new(node),
    }
}

fn source_field(name: Name, node: u32) -> Field {
    Field {
        origin: Origin::Source(source_key(node)),
        name,
        ty: TypeId::UNIT,
        access: Access::Public,
    }
}

#[test]
fn ordinary_field_indexes_allocate_no_scoped_names() -> Result<(), String> {
    let mut members = Members::default();
    let name = Name::Named(members.names.intern("value")?);
    let fields = [source_field(name, 2)];
    let variant = members.insert(ItemId(3), 0, source_key(1), Shape::Record, &fields)?;
    assert_eq!(members.field(variant, name), Some(FieldId(0)));
    assert!(!members.has_contextual_fields());
    assert!(members.scoped_names.is_none());
    let tuple = [source_field(Name::Index(0), 4)];
    let variant = members.insert_contextual(
        ItemId(5),
        0,
        source_key(3),
        Shape::Tuple,
        &tuple,
        &[ItemContext::ROOT],
    )?;
    assert_eq!(members.field(variant, Name::Index(0)), Some(FieldId(1)));
    assert!(members.scoped_names.is_none());
    eprintln!(
        "field storage layouts: Members={} Field={} Variant={}",
        size_of::<Members>(),
        size_of::<Field>(),
        size_of::<super::Variant>(),
    );
    Ok(())
}

#[test]
fn scoped_fields_preserve_exact_context_variant_and_ordinal() -> Result<(), String> {
    let (_marks, [first, second]) = scoped_contexts()?;
    let mut members = Members::default();
    let name = Name::Named(members.names.intern("value")?);
    let fields = [
        source_field(name, 2),
        source_field(name, 3),
        source_field(name, 4),
    ];
    let variant = members.insert_contextual(
        ItemId(3),
        0,
        source_key(1),
        Shape::Record,
        &fields,
        &[ItemContext::ROOT, first, second],
    )?;
    for (offset, context) in [ItemContext::ROOT, first, second].into_iter().enumerate() {
        let field = FieldId(u32::try_from(offset).map_err(|_| "field index overflow")?);
        assert_eq!(members.field_in(variant, name, context), Some(field));
        assert_eq!(
            members.identity(field),
            Some(Identity {
                owner: DefId(3),
                variant: 0,
                field: field.0,
            })
        );
    }
    assert_eq!(members.field(variant, name), Some(FieldId(0)));
    let other = members.insert(ItemId(8), 0, source_key(5), Shape::Record, &[])?;
    assert_eq!(members.field_in(other, name, first), None);
    assert_eq!(
        members.field_in(super::VariantId(u32::MAX), name, first),
        None
    );
    assert_eq!(members.field_in(variant, Name::Index(0), first), None);
    assert_eq!(
        members.scoped_names.as_ref().map(|names| names.len()),
        Some(2)
    );
    Ok(())
}

#[test]
fn scoped_field_duplicate_failures_preserve_all_published_indexes() -> Result<(), String> {
    let (_marks, [context, _]) = scoped_contexts()?;
    let mut members = Members::default();
    let name = Name::Named(members.names.intern("value")?);
    let fields = [source_field(name, 2)];
    let original = members.insert_contextual(
        ItemId(3),
        0,
        source_key(1),
        Shape::Record,
        &fields,
        &[context],
    )?;
    let duplicate = [source_field(name, 4), source_field(name, 5)];
    for contexts in [[context, context], [ItemContext::ROOT, ItemContext::ROOT]] {
        assert_eq!(
            members
                .insert_contextual(
                    ItemId(6),
                    0,
                    source_key(3),
                    Shape::Record,
                    &duplicate,
                    &contexts,
                )
                .err(),
            Some("duplicate aggregate field")
        );
        assert_eq!(members.fields.len(), 1);
        assert_eq!(members.variants.len(), 1);
        assert_eq!(members.at_node(source_key(3)), None);
        assert_eq!(members.field_in(original, name, context), Some(FieldId(0)));
        assert!(members.by_name.is_empty());
        assert_eq!(
            members.scoped_names.as_ref().map(|names| names.len()),
            Some(1)
        );
    }
    let next = members.insert_contextual(
        ItemId(6),
        0,
        source_key(3),
        Shape::Record,
        &duplicate,
        &[ItemContext::ROOT, context],
    )?;
    assert_eq!(members.field(next, name), Some(FieldId(1)));
    assert_eq!(members.field_in(next, name, context), Some(FieldId(2)));
    assert_eq!(members.field_in(original, name, context), Some(FieldId(0)));
    Ok(())
}

#[test]
fn scoped_field_input_failures_cannot_create_contextual_storage() -> Result<(), String> {
    let (_marks, [context, _]) = scoped_contexts()?;
    let mut members = Members::default();
    let tuple = [source_field(Name::Index(0), 2)];
    assert_eq!(
        members
            .insert_contextual(ItemId(3), 0, source_key(1), Shape::Tuple, &tuple, &[],)
            .err(),
        Some("aggregate field context count differs from its fields")
    );
    assert_eq!(
        members
            .insert_contextual(
                ItemId(3),
                0,
                source_key(1),
                Shape::Tuple,
                &tuple,
                &[context],
            )
            .err(),
        Some("numeric aggregate fields cannot carry an item context")
    );
    assert!(members.scoped_names.is_none());
    assert!(members.by_name.is_empty());
    assert!(members.fields.is_empty());
    assert!(members.variants.is_empty());
    let named = Name::Named(members.names.intern("value")?);
    let foreign = [Field {
        origin: Origin::External(DefId(3)),
        ..source_field(named, 2)
    }];
    assert_eq!(
        members
            .insert_contextual(
                ItemId(3),
                0,
                source_key(1),
                Shape::Record,
                &foreign,
                &[context],
            )
            .err(),
        Some("aggregate field provenance differs from its owner")
    );
    assert!(!members.has_contextual_fields());
    Ok(())
}

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
