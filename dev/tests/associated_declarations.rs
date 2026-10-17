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
    let (report, facts) = semantic::analyze(source)?;
    assert_eq!(facts.generics.implementations().count(), 1);
    assert_eq!(facts.destructors.implementations().len(), 1);
    let implementation = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing user implementation")?;
    let rustgm::types::parameters::predicates::Bound::Trait(id) = implementation.predicate.bound
    else {
        return Err("missing user trait application".into());
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("missing user trait")?;
    assert_ne!(
        Some(DefId(application.item.0)),
        report.resolution.drop_trait
    );
    assert_ne!(
        implementation.owner,
        facts.destructors.implementations()[0].self_definition
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

const CONSTANT_CASES: &[(&str, &str, bool)] = &[
    (
        "required",
        "trait A{const C:u8;}impl A for u8{const C:u8=7;}fn main(){}\n",
        true,
    ),
    (
        "default",
        "trait A{const C:u8=7;}impl A for u8{}fn main(){}\n",
        true,
    ),
    (
        "override",
        "trait A{const C:u8=7;}impl A for u8{const C:u8=9;}fn main(){}\n",
        true,
    ),
    (
        "inherent",
        "struct S;impl S{pub const C:u8=7;}fn main(){}\n",
        true,
    ),
    (
        "missing",
        "trait A{const C:u8;}impl A for u8{}fn main(){}\n",
        false,
    ),
    (
        "extra",
        "trait A{}impl A for u8{const C:u8=7;}fn main(){}\n",
        false,
    ),
    (
        "wrong-type",
        "trait A{const C:u8;}impl A for u8{const C:u16=7;}fn main(){}\n",
        false,
    ),
    (
        "wrong-member-kind",
        "trait A{fn C()->u8;}impl A for u8{const C:u8=7;}fn main(){}\n",
        false,
    ),
    (
        "wrong-body",
        "trait A{const C:u8;}impl A for u8{const C:u8=true;}fn main(){}\n",
        false,
    ),
    (
        "wrong-default-body",
        "trait A{const C:u8=true;}fn main(){}\n",
        false,
    ),
    (
        "trait-public",
        "trait A{pub const C:u8;}fn main(){}\n",
        false,
    ),
    (
        "override-public",
        "trait A{const C:u8;}impl A for u8{pub const C:u8=7;}fn main(){}\n",
        false,
    ),
    (
        "inherent-missing",
        "struct S;impl S{const C:u8;}fn main(){}\n",
        false,
    ),
    (
        "self-type",
        "trait A{const C:Self;}impl A for u8{const C:Self=7;}fn main(){}\n",
        true,
    ),
    (
        "unsized-required",
        "trait A{const C:str;}fn main(){}\n",
        true,
    ),
    (
        "generic-result",
        "trait A<T>{const C:T;}impl A<u16>for u8{const C:u16=7;}fn main(){}\n",
        true,
    ),
    (
        "generic-wrong-result",
        "trait A<T>{const C:T;}impl A<u16>for u8{const C:u8=7;}fn main(){}\n",
        false,
    ),
    (
        "projected-result",
        "trait A{type Out;const C:<Self as A>::Out;}impl A for u8{type Out=u16;const C:u16=7;}fn main(){}\n",
        true,
    ),
    (
        "projected-wrong-result",
        "trait A{type Out;const C:<Self as A>::Out;}impl A for u8{type Out=u16;const C:u8=7;}fn main(){}\n",
        false,
    ),
    (
        "captured-const",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn main(){}\n",
        true,
    ),
    (
        "mixed-capture",
        "trait A<T,const N:usize>{const C:[T;N];}struct S<const N:usize>;impl<const N:usize>A<u8,N>for S<N>{const C:[u8;N]=[7;N];}fn main(){}\n",
        true,
    ),
    (
        "lifetimes",
        "trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for u8{const C:&'a u8=&7;}fn main(){}\n",
        true,
    ),
    (
        "longer-override",
        "trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for u8{const C:&'static u8=&7;}fn main(){}\n",
        true,
    ),
    (
        "shorter-override",
        "trait A<'a>{const C:&'static u8;}impl<'a>A<'a>for u8{const C:&'a u8=&7;}fn main(){}\n",
        false,
    ),
    (
        "equal-lifetimes",
        "trait A<'a>{const C:&'a u8;}impl<'a,'b>A<'a>for &'b u8 where 'a:'b,'b:'a{const C:&'b u8=&7;}fn main(){}\n",
        true,
    ),
    (
        "unrelated-lifetimes",
        "trait A<'a>{const C:&'a u8;}impl<'a,'b>A<'a>for &'b u8{const C:&'b u8=&7;}fn main(){}\n",
        false,
    ),
    (
        "default-lazy-fault",
        "#![allow(unconditional_panic)]\ntrait A{const C:u8=1/0;}impl A for u8{}fn main(){}\n",
        true,
    ),
    (
        "inherent-lazy-fault",
        "#![allow(unconditional_panic)]\nstruct S;impl S{const C:u8=1/0;}fn main(){}\n",
        true,
    ),
    (
        "generic-bound-result",
        "struct S<T:Copy>{x:T}trait A<T:Copy>{const C:S<T>;}fn main(){}\n",
        true,
    ),
    (
        "generic-missing-bound",
        "struct S<T:Copy>{x:T}trait A<T>{const C:S<T>;}fn main(){}\n",
        false,
    ),
    (
        "required-lifetime-debt",
        "trait A<'a>{const C:&'static &'a u8;}fn main(){}\n",
        false,
    ),
    (
        "required-lifetime-bound",
        "trait A<'a:'static>{const C:&'static &'a u8;}fn main(){}\n",
        true,
    ),
    (
        "required-parameter-debt",
        "trait A<T>{const C:&'static T;}fn main(){}\n",
        false,
    ),
    (
        "required-parameter-bound",
        "trait A<T:'static>{const C:&'static T;}fn main(){}\n",
        true,
    ),
    (
        "initializer-lifetime-debt",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'static &'a u8=&&7;}fn main(){}\n",
        false,
    ),
    (
        "initializer-lifetime-bound",
        "struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:&'static &'a u8=&&7;}fn main(){}\n",
        true,
    ),
    (
        "projected-lifetime-type",
        "trait A<'a>{type Out;const C:<Self as A<'a>>::Out;}impl<'a>A<'a>for u8{type Out=&'a u8;const C:&'a u8=&7;}fn main(){}\n",
        true,
    ),
    (
        "nested-invariance",
        "trait A<'a>{const C:&'a mut &'a u8;}impl<'a>A<'a>for u8{const C:&'static mut &'static u8=&mut &7;}fn main(){}\n",
        false,
    ),
    (
        "trait-argument-order",
        "trait A<T,U>{const C:(T,U);}impl A<u8,u16>for u32{const C:(u8,u16)=(7,9);}fn main(){}\n",
        true,
    ),
    (
        "trait-argument-swap",
        "trait A<T,U>{const C:(T,U);}impl A<u8,u16>for u32{const C:(u16,u8)=(7,9);}fn main(){}\n",
        false,
    ),
    (
        "unused-global-call",
        "fn f()->u8{7}const C:u8=f();fn main(){}\n",
        false,
    ),
    (
        "unused-trait-call",
        "fn f()->u8{7}trait A{const C:u8=f();}impl A for u8{}fn main(){}\n",
        false,
    ),
    (
        "unused-inherent-call",
        "fn f()->u8{7}struct S;impl S{const C:u8=f();}fn main(){}\n",
        false,
    ),
    (
        "unused-constructor",
        "struct S(u8);impl S{const C:Self=S(7);}fn main(){}\n",
        true,
    ),
    (
        "unused-constructor-pointer",
        "struct S(u8);const F:fn(u8)->S=S;impl S{const C:Self=F(7);}fn main(){}\n",
        false,
    ),
    (
        "unused-skipped-call",
        "fn f()->u8{7}trait A{const C:u8=if false{f()}else{9};}impl A for u8{}fn main(){}\n",
        false,
    ),
];
#[test]
fn declarations_and_unused_initializers_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CONSTANT_CASES {
        rustc::qualify(
            &format!("associated-constant-contract-{name}"),
            source,
            accepted,
        )?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn defaults_and_overrides_keep_distinct_declaration_identities()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "trait A{const C:u8=7;const D:u16;}impl A for u8{const D:u16=9;}impl A for u16{const C:u8=11;const D:u16=13;}fn main(){}";
    rustc::qualify("associated-constant-identities", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let mut selections = Vec::new();
    for implementation in facts.generics.implementations() {
        assert_eq!(implementation.constants.len(), 2);
        for &(declaration, actual) in &implementation.constants {
            let expected = &report.resolution.definitions[declaration.0 as usize];
            assert!(facts.associated.trait_owner(declaration).is_some());
            if let Some(actual) = actual {
                assert_ne!(declaration, actual);
                assert_eq!(
                    expected.name,
                    report.resolution.definitions[actual.0 as usize].name
                );
                assert_eq!(
                    facts.associated.owner(actual),
                    facts.associated.owner(implementation.owner)
                );
            }
            // Conformance proves a declaration contract; it does not evaluate a value.
            assert!(
                rustgm::sema::values::Reader::new(&facts)
                    .definition(actual.unwrap_or(declaration))
                    .is_none()
            );
            selections.push(actual.is_some());
        }
    }
    assert_eq!(selections, [false, true, true, true]);
    Ok(())
}
