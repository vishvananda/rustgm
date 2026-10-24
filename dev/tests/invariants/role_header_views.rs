//! Neutral header views preserve actual Self, member and qualifier identities.
#[path = "ordinary_clone_role.rs"]
mod ordinary;
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{declaration_inputs::Input, model::Declarations},
    support::{
        bindings::{LangItem, Mode},
        crates::CrateIdentity,
        source::SourceFile,
    },
    types::{arguments::Argument, parameters::Parameter},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;fn additional(&self)->u8;}
#[lang="copy"]pub trait Copy:Clone{}
"#;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn prepare() -> Result<(Report, Declarations)> {
    let provider = CrateIdentity::new("minicore", "neutral_header_v1")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(provider.clone()),
        mode: Mode::Source { provider },
        ..Default::default()
    };
    let report = resolve::analyze_expanded(resolve::expansion::analyze_with_context(
        InputFile {
            identity: "header.rs".into(),
            path: "header.rs".into(),
            source: SourceFile::new(SOURCE.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn role_declaration(
    facts: &Declarations,
    kind: LangItem,
) -> Result<resolve::bindings::Declaration> {
    Ok(facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(kind)
        .ok_or("role")?)
}
fn flags() -> Flags {
    Flags {
        unsafe_: false,
        constant: true,
        auto_: false,
    }
}

#[test]
fn source_role_views_borrow_complete_actual_owner_vectors_and_bodyless_methods() -> Result {
    let (report, facts) = prepare()?;
    let declaration = role_declaration(&facts, LangItem::Clone)?;
    let header = Trait::source(&report, &facts, declaration)?;
    header.profile(&facts.types, flags(), 0)?;
    assert_eq!(header.origin.source(), Some(declaration.origin));
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or("owner")?;
    assert!(std::ptr::eq(
        header.members,
        facts.associated.owner_members(owner).ok_or("members")?
    ));
    assert!(std::ptr::eq(
        header.parameters,
        facts
            .generics
            .parameters
            .get(&declaration.definition)
            .ok_or("parameters")?
            .as_slice()
    ));
    assert_eq!(
        header.members.len(),
        2,
        "retain additional provided/required members"
    );
    let clone = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::CloneFn)
        .ok_or("clone_fn")?;
    let function = facts.function_header(clone.definition)?.ok_or("function")?;
    assert!(
        function.body.is_none(),
        "required method has no invented original"
    );
    clone_method(
        &facts.types,
        &header,
        &Method {
            definition: clone.definition,
            parent: clone.parent,
            signature: function.signature,
            parameters: 0,
        },
    )?;
    Ok(())
}

#[test]
fn neutral_role_headers_reject_foreign_self_owner_slot_flags_and_parent() -> Result {
    let (mut report, mut facts) = prepare()?;
    let declaration = role_declaration(&facts, LangItem::Clone)?;
    let (other, foreign_self) = {
        let header = Trait::source(&report, &facts, role_declaration(&facts, LangItem::Copy)?)?;
        (header.self_type, header.self_definition)
    };
    let self_definition = Trait::source(&report, &facts, declaration)?.self_definition;
    let wrong_slot = facts.types.intern_parameter(Parameter {
        owner: ItemId(self_definition.0),
        index: 1,
        kind: ParameterKind::Type { sized: false },
    })?;
    let wrong_slot = facts.types.intern(Kind::Parameter(wrong_slot))?;
    for case in 0..6 {
        let mut header = Trait::source(&report, &facts, declaration)?;
        match case {
            0 => header.self_type = other,
            1 => header.self_type = wrong_slot,
            2 => header.parent = Some(header.definition),
            3 => header.flags.constant = false,
            4 => header.flags.auto_ = true,
            _ => header.captures = 1,
        }
        assert!(
            header.profile(&facts.types, flags(), 0).is_err(),
            "case={case}"
        );
    }
    facts
        .generics
        .parents
        .insert(declaration.definition, declaration.definition);
    assert!(Trait::source(&report, &facts, declaration).is_err());
    facts
        .generics
        .parents
        .insert(declaration.definition, self_definition);
    facts
        .generics
        .parameters
        .get_mut(&self_definition)
        .ok_or("Self")?[0]
        .argument = Argument::Type(other);
    assert!(Trait::source(&report, &facts, declaration).is_err());
    let self_type = facts
        .associated
        .schema(declaration.definition)
        .ok_or("schema")?
        .self_type;
    facts
        .generics
        .parameters
        .get_mut(&self_definition)
        .ok_or("Self")?[0]
        .argument = Argument::Type(self_type);
    report.resolution.definitions[self_definition.0 as usize].scope =
        report.resolution.definitions[foreign_self.0 as usize].scope;
    assert!(Trait::source(&report, &facts, declaration).is_err());
    Ok(())
}

#[test]
fn transported_header_inputs_preserve_exact_traits_qualifiers_and_method_owners() -> Result {
    let (report, facts) = prepare()?;
    let mut header = Trait::source(&report, &facts, role_declaration(&facts, LangItem::Copy)?)?;
    let clone = role_declaration(&facts, LangItem::Clone)?.definition;
    let source: Vec<Input> = header.inputs.iter().collect();
    header.inputs = Inputs::new(&[], &source);
    assert!(header.requires(&facts.types, clone, None, Constness::Ordinary));
    assert!(!header.requires(&facts.types, clone, None, Constness::Const));
    assert!(header.requires(&facts.types, header.definition, None, Constness::Ordinary));
    let unrelated = role_declaration(&facts, LangItem::LegacyReceiver)?.definition;
    assert!(!header.requires(&facts.types, unrelated, None, Constness::Ordinary));
    let clone_header = Trait::source(&report, &facts, role_declaration(&facts, LangItem::Clone)?)?;
    let clone_fn = role_declaration(&facts, LangItem::CloneFn)?;
    let signature = facts
        .function_header(clone_fn.definition)?
        .ok_or("function")?
        .signature;
    let mut method = Method {
        definition: clone_fn.definition,
        parent: Some(header.definition),
        signature,
        parameters: 0,
    };
    assert!(clone_method(&facts.types, &clone_header, &method).is_err());
    method.parent = clone_fn.parent;
    method.definition = header.definition;
    assert!(clone_method(&facts.types, &clone_header, &method).is_err());
    method.definition = clone_fn.definition;
    method.parameters = 1;
    assert!(clone_method(&facts.types, &clone_header, &method).is_err());
    Ok(())
}

#[test]
fn clone_header_signature_checks_keep_referent_mutability_binders_and_result_exact() -> Result {
    let (report, mut facts) = prepare()?;
    let clone = role_declaration(&facts, LangItem::Clone)?;
    let clone_fn = role_declaration(&facts, LangItem::CloneFn)?;
    let self_type = Trait::source(&report, &facts, clone)?.self_type;
    let other =
        Trait::source(&report, &facts, role_declaration(&facts, LangItem::Copy)?)?.self_type;
    let signature = facts
        .function_header(clone_fn.definition)?
        .ok_or("function")?
        .signature;
    let mut signatures = Vec::new();
    for case in 0..6 {
        let region = if case == 0 {
            crate::types::RegionId::STATIC
        } else {
            crate::types::RegionId::bound(0, 0)?
        };
        let receiver = facts.types.intern(Kind::Reference {
            region,
            mutable: case == 1,
            referent: if case == 2 { other } else { self_type },
        })?;
        let parameters = facts.types.intern_list(&[receiver])?;
        signatures.push(facts.types.intern(Kind::Function {
            bound_regions: if case == 3 { 2 } else { 1 },
            parameters,
            result: if case == 4 { other } else { self_type },
            unsafe_: case == 5,
            convention: crate::types::callable::Convention::RUST,
        })?);
    }
    let header = Trait::source(&report, &facts, clone)?;
    for wrong in signatures {
        assert_ne!(wrong, signature);
        assert!(
            clone_method(
                &facts.types,
                &header,
                &Method {
                    definition: clone_fn.definition,
                    parent: clone_fn.parent,
                    signature: wrong,
                    parameters: 0,
                },
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn neutral_role_views_do_not_grow_dense_owners_or_allocate_import_catalogs() -> Result {
    use crate::sema::associated::{Owner, OwnerKind};
    assert_eq!(
        size_of::<crate::types::parameters::predicates::Predicate>(),
        12
    );
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    let (report, facts) = prepare()?;
    assert!(facts.crates.is_none());
    let header = Trait::source(&report, &facts, role_declaration(&facts, LangItem::Clone)?)?;
    for _ in 0..128 {
        header.profile(&facts.types, flags(), 0)?;
    }
    assert!(facts.crates.is_none());
    println!(
        "Owner={} OwnerKind={} DeclarationOrigin={} TraitView={} FunctionHeader={} TraitContract={} Predicate={} Value={}",
        size_of::<Owner>(),
        size_of::<OwnerKind>(),
        size_of::<DeclarationOrigin>(),
        size_of::<Trait<'_>>(),
        size_of::<crate::sema::function_headers::FunctionHeader<'_>>(),
        size_of::<crate::sema::traits::Header>(),
        size_of::<crate::types::parameters::predicates::Predicate>(),
        size_of::<crate::body::ir::Value>()
    );
    Ok(())
}
