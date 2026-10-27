//! Role identity and completion remain fresh before structural query reuse.
use super::*;
use crate::{sema, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_unsizing_lookup_requires_its_exact_completed_role() -> Result {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/unsizing-roles/array-slice.rs"),
    )?;
    let report = test_source::source_provider(&source, "coercion_evidence")?;
    let mut facts = sema::declarations::analyze(&report)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Unsize)?
        .ok_or("Unsize")?;
    for _ in 0..128 {
        assert_eq!(
            kind(&facts, declaration.definition)?,
            Some(LangItem::Unsize)
        );
    }
    let array = facts.types.intern(Kind::Array {
        element: TypeId::primitive(crate::types::Primitive::U8),
        length: 3,
    })?;
    let slice = facts
        .types
        .intern(Kind::Slice(TypeId::primitive(crate::types::Primitive::U8)))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(slice)])?;
    let application = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: crate::types::ItemId(declaration.definition.0),
            arguments,
        })?;
    let goal = Predicate {
        ty: array,
        bound: crate::types::parameters::predicates::Bound::Trait(application),
    };
    let mut proof = crate::sema::nominals::traits::State::instance(&mut facts, 0)?;
    for _ in 0..128 {
        assert!(proof.prove(&mut facts, goal, false)?);
    }
    let catalog = facts.source_bindings.as_deref_mut().ok_or("catalog")?;
    let saved = catalog.traits.as_deref().ok_or("roles")?.roles;
    catalog
        .traits
        .as_deref_mut()
        .ok_or("roles")?
        .roles
        .remove(LangItem::Unsize);
    assert!(kind(&facts, declaration.definition).is_err());
    assert!(proof.prove(&mut facts, goal, false).is_err());
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .traits
        .as_deref_mut()
        .ok_or("roles")?
        .roles = saved;
    assert_eq!(
        kind(&facts, declaration.definition)?,
        Some(LangItem::Unsize)
    );
    assert!(kind(&facts, DefId(u32::MAX))?.is_none());
    println!(
        "Catalog={} Reservations={} Declaration={} Validated={} OwnerKind={} Body={} Store={}",
        size_of::<Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<Declaration>(),
        size_of::<super::super::Validated>(),
        size_of::<crate::sema::associated::OwnerKind>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::types::Store>()
    );
    Ok(())
}
