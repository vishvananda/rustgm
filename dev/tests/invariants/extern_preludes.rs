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
    let method = facts
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual source role catalog")?
        .validated_role(crate::support::bindings::LangItem::CloneFn)?
        .ok_or("actual imported clone method")?;
    let header = facts
        .facts
        .function_header(method.definition)?
        .ok_or("actual imported method header")?;
    let declaration = facts
        .facts
        .declaration_header(method.definition)?
        .ok_or("actual imported declaration")?;
    assert_eq!(declaration.declaration, header.declaration);
    assert_eq!(declaration.body, header.body);
    assert_eq!(declaration.signature, header.signature);
    assert_eq!(declaration.universals, header.universals);
    let Some(crate::types::Kind::Function { parameters, .. }) =
        facts.facts.types.kind(header.signature)
    else {
        return Err("actual clone signature".into());
    };
    let receiver = *facts
        .facts
        .types
        .list(parameters)
        .and_then(|parameters| parameters.first())
        .ok_or("actual clone first parameter")?;
    assert_eq!(
        facts.facts.methods.receivers.get(&method.definition),
        Some(&(receiver, 1))
    );
    assert!(matches!(
        facts.facts.types.kind(receiver),
        Some(crate::types::Kind::Reference { mutable: false, .. })
    ));
    Ok(())
}
