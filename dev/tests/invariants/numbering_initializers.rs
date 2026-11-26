//! Constant-rooted closure parameters retain exact lexical and original coordinates.
use crate::{
    lower::instances::SourcePublication,
    module_packets as test,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};

fn provider() -> test::Result<crates::Interface> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/source-enum-role-transport/provider.rs")?;
    let prefix = foundation
        .split_once("#[rustc_diagnostic_item = \"Option\"]")
        .ok_or("actual initializer fixture boundary")?
        .0;
    let tail =
        crate::test_source::fixture("pa31/tests/spec/source-initializer-numbering/provider.rs")?;
    test::exported_profile(
        &format!("#![feature(auto_traits,const_destruct)]\n{prefix}{tail}"),
        SourcePublication::ModuleBodyOperations,
        "initializer_numbering",
    )
}

#[test]
fn initializer_closure_parameters_roundtrip_and_reject_foreign_body_coordinates() -> test::Result {
    let artifact = provider()?;
    let bytes = crates::encode(&artifact)?;
    let fresh = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&fresh)?, bytes);
    let _ = crate::source_packets::client_analysis(
        crates::decode(&bytes)?,
        &crate::test_source::fixture("pa31/tests/spec/source-initializer-numbering/client.rs")?,
    )?;
    let (actual, attributes, modules) = test::raw(&fresh)?;
    let parameter = actual
        .declarations
        .iter()
        .position(|row| {
            row.definition.kind == s::DefinitionKind::ValueParameter
                && row.definition.parent.is_some_and(|parent| {
                    actual.declarations[parent.0 as usize].name.as_deref() == Some("SELECT")
                })
        })
        .ok_or("actual constant-rooted closure parameter")?;
    let parent = actual.declarations[parameter]
        .definition
        .parent
        .ok_or("actual initializer declaration")?;
    let body = actual
        .body_owners
        .iter()
        .position(|owner| owner.declaration == parent && owner.parent.is_none())
        .ok_or("actual initializer original")?;
    let initializer = crate::support::crates::owner_catalog::BodyRef(u32::try_from(body)?);
    let local = actual.declarations[parameter]
        .local
        .ok_or("actual closure binding")?;
    assert_ne!(local.body, initializer);
    assert_eq!(
        actual.body_owners[local.body.0 as usize].parent,
        Some(initializer)
    );
    let availability = fresh
        .source_descriptor()
        .ok_or("actual initializer descriptor")?
        .availability()?
        .clone();
    let select_index = actual.declarations[..=parent.0 as usize]
        .iter()
        .filter(|row| row.definition.kind == s::DefinitionKind::Constant)
        .count()
        - 1;
    let select_ty = actual.constants()[select_index].ty;
    let borrow = actual
        .declarations
        .iter()
        .position(|row| row.name.as_deref() == Some("BORROW"))
        .ok_or("actual borrowed constant")?;
    let borrowed = crate::support::crates::owner_catalog::DeclarationRef(u32::try_from(borrow)?);
    let (original, value) = actual
        .originals
        .iter()
        .enumerate()
        .find_map(|(index, body)| {
            body.values
                .iter()
                .position(|value| value.kind == s::ValueKind::Constant(borrowed))
                .map(|value| (index, value))
        })
        .ok_or("actual borrowed constant read")?;
    let s::Type::Function(signature) =
        actual.types[actual.originals[original].values[value].ty.0 as usize]
    else {
        return Err("actual borrowed function-pointer recipe".into());
    };
    for case in 0..6 {
        let mut changed = actual.clone();
        let row = &mut changed.declarations[parameter];
        match case {
            0 => row.local = None,
            1 => row.local.as_mut().ok_or("actual local")?.body = initializer,
            2 => row.definition.kind = s::DefinitionKind::TypeParameter,
            3 => row.ordinal += 1,
            4 => changed.originals[original].values[value].ty = select_ty,
            5 => changed.binders[signature.binder.0 as usize].owner = parent,
            _ => return Err("unknown initializer opponent".into()),
        }
        assert!(
            g::Payload::from_source_module_body_operations(
                fresh.identity(),
                changed,
                attributes.clone(),
                modules.clone(),
                availability.clone(),
                0,
                &mut g::Work::default(),
            )
            .is_err(),
            "initializer opponent {case}"
        );
    }
    Ok(())
}
