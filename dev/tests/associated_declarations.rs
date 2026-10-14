//! Associated ownership is shared metadata, independent of method selection and impl proof.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    resolve::model::{DefId, DefKind},
    sema::associated::{self, Inventory, OwnerId, OwnerKind},
    types::Kind,
};
use std::fmt::Write as _;

fn names(
    report: &rustgm::resolve::expansion::Report,
    inventory: &Inventory,
    owner: OwnerId,
) -> Vec<String> {
    inventory
        .members()
        .iter()
        .filter(|m| m.owner == owner)
        .map(|m| {
            let definition = &report.resolution.definitions[m.definition.0 as usize];
            report.package.names.get(definition.name).to_owned()
        })
        .collect()
}

#[test]
fn trait_impl_and_nested_method_members_keep_their_direct_owners()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "trait Q<A>{type Output;const C:u8;fn required(&self,x:A)->A;fn default<T>(&self,x:T)->T{fn nested(){}x}}struct S;impl Q<u8> for S{type Output=u8;const C:u8=7;fn required(&self,x:u8)->u8{x}}impl S{fn new()->Self{S}fn extra(&mut self){}}fn outer(){struct N;impl N{fn local(){}}}";
    rustc::qualify("associated-owner-inventory", source, true)?;
    let report = semantic::resolve(source)?;
    let inventory = associated::collect(&report)?;
    assert_eq!(inventory.owners().len(), 4);
    assert_eq!(inventory.members().len(), 10);
    let expected = [
        vec!["Output", "C", "required", "default"],
        vec!["Output", "C", "required"],
        vec!["new", "extra"],
        vec!["local"],
    ];
    for (index, owner) in inventory.owners().iter().enumerate() {
        let id = OwnerId(u32::try_from(index)?);
        assert_eq!(names(&report, &inventory, id), expected[index]);
        assert_eq!(inventory.owner(owner.self_definition), Some(id));
        assert_eq!(inventory.get(id), Some(owner));
        if let OwnerKind::Trait { definition, .. } = owner.kind {
            assert_eq!(inventory.owner(definition), Some(id));
            assert_eq!(
                report.resolution.definitions[definition.0 as usize].kind,
                DefKind::Trait
            );
        }
    }
    for member in inventory.members() {
        assert_eq!(inventory.owner(member.definition), Some(member.owner));
    }
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if matches!(
            definition.kind,
            DefKind::Generic | DefKind::Parameter | DefKind::Local
        ) || report.package.names.get(definition.name) == "nested"
        {
            assert!(inventory.owner(DefId(u32::try_from(index)?)).is_none());
        }
    }
    Ok(())
}

#[test]
fn configured_and_generated_owners_have_deterministic_declaration_order()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "macro_rules! make{($name:ident)=>{struct $name;impl $name{fn get(&self)->u8{7}}}}make!(A);make!(B);#[cfg(any())]trait Gone{fn absent(&self);}trait T{#[cfg(any())]fn absent(&self);fn visible();}impl T for A{fn visible(){}}";
    rustc::qualify("associated-configured-inventory", source, true)?;
    let report = semantic::resolve(source)?;
    let first = associated::collect(&report)?;
    assert_eq!(first.owners().len(), 4);
    assert_eq!(first.members().len(), 4);
    let flattened = first
        .members()
        .iter()
        .map(|m| {
            report
                .package
                .names
                .get(report.resolution.definitions[m.definition.0 as usize].name)
        })
        .collect::<Vec<_>>();
    // Expansion appends generated definitions after the crate's declared items.
    assert_eq!(flattened, ["visible", "visible", "get", "get"]);
    for _ in 0..16 {
        let report = semantic::resolve(source)?;
        let next = associated::collect(&report)?;
        assert_eq!(first.owners(), next.owners());
        assert_eq!(first.members(), next.members());
    }
    Ok(())
}

#[test]
fn generated_trait_owners_keep_module_and_source_identities()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "macro_rules! make{($m:ident)=>{mod $m{pub trait T{fn get(&self)->u8;}pub struct S;impl T for S{fn get(&self)->u8{7}}}}}make!(a);make!(b);";
    rustc::qualify("associated-generated-traits", source, true)?;
    let report = semantic::resolve(source)?;
    let inventory = associated::collect(&report)?;
    assert_eq!(inventory.owners().len(), 4);
    assert_eq!(inventory.members().len(), 4);
    let mut traits = Vec::new();
    for owner in inventory.owners() {
        if let OwnerKind::Trait { definition, .. } = owner.kind {
            let declaration = &report.resolution.definitions[definition.0 as usize];
            assert_eq!(declaration.node, Some(owner.source));
            assert_eq!(
                inventory.owner(definition),
                inventory.owner(owner.self_definition)
            );
            traits.push((definition, owner.source));
        }
    }
    assert_eq!(traits.len(), 2);
    assert_ne!(traits[0].0, traits[1].0);
    assert_ne!(traits[0].1, traits[1].1);
    Ok(())
}

#[test]
fn drop_uses_shared_ownership_and_receivers_without_granting_user_traits_language_behavior()
-> Result<(), Box<dyn std::error::Error>> {
    for (receiver, mutable) in [
        ("&mut self", false),
        ("self:&mut Self", false),
        ("mut self:&mut Self", true),
    ] {
        let source = format!(
            "mod shadow{{pub trait Drop{{}}pub struct S;}}struct D;impl ::core::ops::Drop for D{{fn drop({receiver}){{}}}}fn main(){{let _d=D;}}"
        );
        rustc::qualify("associated-drop-receiver", &source, true)?;
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.associated.owners().len(), 2);
        assert_eq!(facts.destructors.implementations().len(), 1);
        assert_eq!(facts.receivers.len(), 1);
        let implementation = &facts.destructors.implementations()[0];
        assert_eq!(
            facts.associated.owner(implementation.method),
            facts.associated.owner(implementation.self_definition)
        );
        let receiver = facts
            .receivers
            .values()
            .next()
            .ok_or("missing shared receiver")?;
        assert_eq!(receiver.mutable, mutable);
        assert!(
            matches!(facts.types.kind(receiver.ty),Some(Kind::Reference{mutable:true,referent,..}) if referent==implementation.target)
        );
    }
    let source = "mod shadow{pub trait Drop{}pub struct S;impl Drop for S{}}struct D;impl ::core::ops::Drop for D{fn drop(&mut self){}}fn main(){}";
    rustc::qualify("associated-shadow-drop-impl", source, true)?;
    let report = semantic::resolve(source)?;
    let inventory = associated::collect(&report)?;
    assert_eq!(inventory.owners().len(), 3);
    let Err(error) = semantic::analyze(source) else {
        return Err("user impl passed without conformance".into());
    };
    assert!(
        error.contains("trait implementation requires conformance"),
        "{error}"
    );
    Ok(())
}

#[test]
fn wide_owner_inventory_excludes_parameters_and_is_bounded_by_direct_members()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::new();
    for index in 0..256 {
        write!(
            source,
            "struct S{index}<T>(T);impl<T>S{index}<T>{{fn get(&self)->&T{{&self.0}}fn wrap<U>(self,x:U)->(T,U){{(self.0,x)}}}}"
        )?;
    }
    rustc::qualify("associated-wide-inventory", &source, true)?;
    let report = semantic::resolve(&source)?;
    let inventory = associated::collect(&report)?;
    assert_eq!(inventory.owners().len(), 256);
    assert_eq!(inventory.members().len(), 512);
    for (index, owner) in inventory.owners().iter().enumerate() {
        let id = OwnerId(u32::try_from(index)?);
        assert_eq!(inventory.owner(owner.self_definition), Some(id));
        assert_eq!(names(&report, &inventory, id), ["get", "wrap"]);
    }
    Ok(())
}

#[test]
fn unresolved_and_missing_identities_cannot_fabricate_associated_owners()
-> Result<(), Box<dyn std::error::Error>> {
    let report = semantic::resolve("fn plain()->u8{7}")?;
    let inventory = associated::collect(&report)?;
    assert!(inventory.owners().is_empty() && inventory.members().is_empty());
    assert!(inventory.owner(DefId(u32::MAX)).is_none());
    assert!(inventory.get(OwnerId(u32::MAX)).is_none());
    let report = semantic::expand("trait T{fn m(&self);}")?;
    assert!(associated::collect(&report).is_err());
    Ok(())
}
