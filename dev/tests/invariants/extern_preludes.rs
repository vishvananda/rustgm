//! A real language role supplies no implicit source-crate trait import.
use crate::{source_packets, support::crates};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn source_lang_trait_is_not_a_method_prelude_without_actual_import() -> Result {
    let provider = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let bytes = crates::encode(&source_packets::exported(&provider)?)?;
    let declarations = "#![feature(no_core)]#![no_core]extern crate base;struct Local;impl base::Clone for Local{fn clone(&self)->Self{Local}}impl base::Copy for Local{}";
    let body = "fn selected(value:Local)->Local{value.clone()}";
    let absent =
        source_packets::client_analysis(crates::decode(&bytes)?, &format!("{declarations}{body}"));
    let error = absent
        .err()
        .ok_or("source Clone leaked into an implicit method prelude")?;
    assert!(error.to_string().contains("method"), "{error}");
    let (_, facts, _) = source_packets::client_analysis(
        crates::decode(&bytes)?,
        &format!("{declarations}use base::Clone;{body}"),
    )?;
    assert!(
        facts.facts.cloning.is_none(),
        "source method lookup cannot mint a bootstrap registry"
    );
    let role = facts
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual source role catalog")?
        .validated_role(crate::support::bindings::LangItem::Clone)?
        .ok_or("actual imported Clone")?;
    assert!(matches!(
        role.origin,
        crate::resolve::body_origin::DeclarationOrigin::Imported(_)
    ));
    Ok(())
}
