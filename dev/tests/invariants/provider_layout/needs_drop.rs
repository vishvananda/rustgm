//! The bool intrinsic identity is distinct from layout data and caller permission.
use super::*;
use crate::{
    support::bindings::{LayoutIntrinsic, SourceIntrinsic},
    types::{GenericArgs, ItemId, arguments::Argument, instance::Instance},
};

use std::mem::size_of;

const DECLARATION: &str = "#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;";

#[test]
fn source_needs_drop_requires_its_actual_completed_declaration() -> Result {
    let report = resolve(DECLARATION)?;
    let reserved = Declarations::new(&report)?;
    let entry = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?
        .intrinsics()
        .next()
        .ok_or("intrinsic")?;
    assert_eq!(entry.kind, SourceIntrinsic::NeedsDrop);
    let catalog = reserved.source_bindings.as_deref().ok_or("catalog")?;
    assert!(
        catalog
            .intrinsic_kind(entry.declaration.definition)
            .is_err()
    );
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert_eq!(
        catalog.intrinsic_kind(entry.declaration.definition)?,
        Some(SourceIntrinsic::NeedsDrop)
    );
    assert_eq!(catalog.intrinsic_kind(DefId(u32::MAX))?, None);
    Ok(())
}

#[test]
fn source_needs_drop_selection_retains_full_closed_identity_without_layout_permission() -> Result {
    let report = resolve(DECLARATION)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let entry = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?
        .intrinsics()
        .next()
        .ok_or("intrinsic")?;
    let function = facts.functions[facts.function_indices[&entry.declaration.definition]];
    let mut selected = Vec::new();
    for element in [
        TypeId::primitive(Primitive::U8),
        TypeId::primitive(Primitive::Str),
    ] {
        let arguments = facts.types.intern_arguments(&[Argument::Type(element)])?;
        let ty = facts.types.intern(Kind::FunctionItem {
            item: ItemId(function.definition.0),
            arguments,
            signature: function.signature,
        })?;
        let instance = Instance::new(&facts.types, ty)?;
        let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
        let data = catalog
            .selected_intrinsic(&facts, instance)?
            .ok_or("selection")?;
        assert_eq!(data.kind, SourceIntrinsic::NeedsDrop);
        assert_eq!(data.element, element);
        assert_eq!(data.definition, function.definition);
        assert!(!data.fallback);
        assert!(catalog.layout_selection(&facts, instance)?.is_none());
        selected.push(instance);
    }
    assert_ne!(selected[0], selected[1]);
    let malformed = facts.types.intern(Kind::FunctionItem {
        item: ItemId(function.definition.0),
        arguments: GenericArgs::EMPTY,
        signature: function.signature,
    })?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .selected_intrinsic(&facts, Instance::new(&facts.types, malformed)?)
            .is_err()
    );
    Ok(())
}

#[test]
fn source_needs_drop_exact_pinned_signature_is_not_an_ordinary_name_hook() -> Result {
    for declaration in [
        "#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->usize;",
        "#[rustc_intrinsic]pub const unsafe fn needs_drop<T:?Sized>()->bool;",
        "#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>(x:&T)->bool;",
        "#[rustc_intrinsic]pub fn needs_drop<T:?Sized>()->bool;",
        // rustc permits a narrower declaration; this is an explicit source-profile boundary.
        "#[rustc_intrinsic]pub const fn needs_drop<T>()->bool;",
    ] {
        let report = resolve(declaration)?;
        assert!(
            crate::sema::declarations::analyze(&report).is_err(),
            "{declaration}"
        );
    }
    let facts = inspect("pub const fn needs_drop<T:?Sized>()->bool{true}")?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert!(!catalog.has_intrinsics());
    assert!(
        facts
            .functions
            .iter()
            .all(|f| catalog.intrinsic_kind(f.definition) == Ok(None))
    );
    Ok(())
}

#[test]
fn incomplete_source_drop_roles_never_execute_an_intrinsic_fallback() -> Result {
    for body in [";", "{false}", "{true}"] {
        let tail = format!(
            "#[rustc_intrinsic]const fn needs_drop<T:?Sized>()->bool{body}const N:bool=needs_drop::<u8>();type A=[u8;N as usize];"
        );
        let error = inspect(&tail)
            .err()
            .ok_or("source needs_drop ran without its actual source roles")?
            .to_string();
        assert!(
            error.contains("completed source Drop and Copy roles"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn source_intrinsic_kind_keeps_ordinary_storage_density() {
    use crate::resolve::bindings::{Declaration, IntrinsicDeclaration};
    assert_eq!(size_of::<LayoutIntrinsic>(), 1);
    assert!(size_of::<SourceIntrinsic>() <= 2);
    assert_eq!(
        size_of::<IntrinsicDeclaration>(),
        size_of::<(Declaration, LayoutIntrinsic)>()
    );
    assert_eq!(
        size_of::<super::super::Intrinsic>(),
        size_of::<(Declaration, LayoutIntrinsic, Option<TypeId>)>()
    );
    println!(
        "SourceIntrinsic={} IntrinsicDeclaration={} Intrinsic={} SourceCatalog={} CodeCatalog={}",
        size_of::<SourceIntrinsic>(),
        size_of::<IntrinsicDeclaration>(),
        size_of::<super::super::Intrinsic>(),
        size_of::<super::super::Catalog>(),
        size_of::<crate::const_eval::code::Catalog>()
    );
}
