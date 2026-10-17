//! Associated ownership is shared metadata, independent of method selection and impl proof.
#[path = "associated_declarations/const_arguments.rs"]
mod const_arguments;
#[path = "associated_declarations/discriminants.rs"]
mod discriminants;
#[path = "harness/native_references.rs"]
mod native;
#[path = "associated_declarations/readiness.rs"]
mod readiness;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "associated_declarations/shorthand.rs"]
mod shorthand;
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

const VALUE_CASES: &[(&str, &str, bool)] = &[
    (
        "boundary",
        "trait A{const C:u8;}struct S;impl A for S{const C:u8=7;}fn main(){let _=<S as A>::C;}",
        true,
    ),
    (
        "default",
        "trait A{const C:u8=7;}struct S;impl A for S{}fn main(){let x=<S as A>::C;let _=x+1;}",
        true,
    ),
    (
        "override",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}fn main(){let x=<S as A>::C;let _=x+1;}",
        true,
    ),
    (
        "inherent",
        "struct S;impl S{const C:u8=7;}fn main(){let _=S::C;}",
        true,
    ),
    (
        "two_instances",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn main(){let _=S::<3>::C+S::<5>::C+S::<3>::C;}",
        true,
    ),
    (
        "generic_body",
        "trait A{const C:u8;}impl A for u8{const C:u8=7;}fn get<T:A>()->u8{T::C}fn main(){let _=get::<u8>();}",
        true,
    ),
    (
        "two_traits",
        "trait A{const C:u8=7;}trait B{const C:u8=9;}struct S;impl A for S{}impl B for S{}fn main(){let _=<S as A>::C+<S as B>::C;}",
        true,
    ),
    (
        "owner_const",
        "trait A<const N:usize>{const C:usize=N;}impl<const N:usize>A<N> for u8{}fn main(){let _=<u8 as A<3>>::C+<u8 as A<5>>::C;}",
        true,
    ),
    (
        "const_override",
        "trait A<const N:usize>{const C:usize=N;}impl<const N:usize>A<N> for u8{const C:usize=N+1;}fn main(){let _=<u8 as A<3>>::C+<u8 as A<5>>::C;}",
        true,
    ),
    (
        "tuple",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:(usize,u8)=(N,7);}fn main(){let _=S::<3>::C.0+S::<5>::C.0;}",
        true,
    ),
    (
        "array",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:[u8;N]=[7;N];}fn main(){let _=S::<3>::C[2]+S::<5>::C[4];}",
        true,
    ),
    (
        "reference",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:&'static usize=&N;}fn main(){let _=*S::<3>::C+*S::<5>::C;}",
        true,
    ),
    (
        "dependency",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const A:usize=N;const B:usize=Self::A+1;}fn main(){let _=S::<3>::B+S::<5>::B;}",
        true,
    ),
    (
        "global",
        "trait A{const C:u8=7;}impl A for u8{}const X:u8=<u8 as A>::C;fn main(){let _=X;}",
        true,
    ),
    (
        "cycle",
        "struct S;impl S{const A:u8=Self::B;const B:u8=Self::A;}fn main(){let _=S::A;}",
        false,
    ),
    (
        "bad_bound",
        "trait A{const C:u8;}fn get<T>()->u8{<T as A>::C}fn main(){}",
        false,
    ),
    (
        "ambiguous",
        "trait A{const C:u8=7;}trait B{const C:u8=9;}struct S;impl A for S{}impl B for S{}fn main(){let _=S::C;}",
        false,
    ),
    (
        "non_method",
        "struct S;impl S{const C:u8=7;}fn main(){S.C();}",
        false,
    ),
    (
        "extra_args",
        "struct S;impl S{const C:u8=7;}fn main(){let _=S::C::<u8>;}",
        false,
    ),
    (
        "bad_result",
        "trait A{const C:u8=7;}impl A for u8{}fn main(){let _:u16=<u8 as A>::C;}",
        false,
    ),
    (
        "overflow",
        "struct S;impl S{const C:u8=255u8+1;}fn main(){let _=S::C;}",
        false,
    ),
    (
        "unused_fault",
        "struct S;impl S{const C:u8=1/0;}fn main(){}",
        true,
    ),
    (
        "trait_dependency",
        "trait A{const C:u8;const D:u8=Self::C+1;}impl A for u8{const C:u8=7;}fn main(){let _=<u8 as A>::D;}",
        true,
    ),
    (
        "generic_trait_dependency",
        "trait A<const N:usize>{const C:usize=N;const D:usize=Self::C+1;}impl<const N:usize>A<N> for u8{}fn main(){let _=<u8 as A<3>>::D+<u8 as A<5>>::D;}",
        true,
    ),
    (
        "unused_cycle",
        "struct S;impl S{const A:u8=Self::B;const B:u8=Self::A;}fn main(){}",
        false,
    ),
    (
        "skipped_cycle",
        "struct S;impl S{const A:u8=if true{7}else{Self::A};}fn main(){let _=S::A;}",
        false,
    ),
    (
        "unused_borrow_fault",
        "struct S;impl S{const A:&'static u8=&(1/0);}fn main(){}",
        true,
    ),
    (
        "used_borrow_fault",
        "struct S;impl S{const A:&'static u8=&(1/0);}fn main(){let _=S::A;}",
        false,
    ),
    (
        "qualified_lifetime",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'a u8=&7;}fn main(){let _=*<S<'static>>::C;}",
        true,
    ),
    (
        "lifetime_escape",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'a u8=&7;}fn f<'a>()->&'static u8{<S<'a>>::C}fn main(){}",
        false,
    ),
    (
        "private",
        "mod m{pub struct S;impl S{const A:u8=7;}}fn main(){let _=m::S::A;}",
        false,
    ),
    (
        "infer_erased",
        "struct S<T>(T);impl<T>S<T>{const A:u8=7;}fn main(){let _=S::A;}",
        false,
    ),
    (
        "direct_trait",
        "trait A{const C:Self;}impl A for u8{const C:Self=7;}fn main(){let _:u8=A::C;}",
        true,
    ),
    (
        "generic_unused_call",
        "fn f()->u8{7}struct S<T>(T);impl<T>S<T>{const C:u8=f();}fn main(){}",
        false,
    ),
    (
        "cycle-closed-unused",
        "struct S;impl S{const A:u8=Self::B;const B:u8=Self::A;}fn main(){}",
        false,
    ),
    (
        "cycle-generic-unused",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const A:usize=Self::B;const B:usize=Self::A;}fn main(){}",
        true,
    ),
    (
        "cycle-trait-unused",
        "trait A{const C:u8=Self::D;const D:u8=Self::C;}fn main(){}",
        true,
    ),
    (
        "cycle-trait-override",
        "trait A{const C:u8=Self::D;const D:u8=Self::C;}impl A for u8{const D:u8=7;}fn main(){let _=<u8 as A>::C;}",
        true,
    ),
    (
        "cycle-closed-lazy",
        "struct S;impl S{const A:u8=if true{7}else{Self::A};}fn main(){}",
        true,
    ),
];

fn object_outcome(
    name: &str,
    source: &str,
    accepted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/constant-instances-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let source_file = root.join(format!("{name}.rs"));
    let object = root.join(format!("{name}.o"));
    std::fs::write(&source_file, source)?;
    let result = std::process::Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=constant_instances",
            "--emit=obj",
            "--cap-lints=allow",
            "-C",
            "overflow-checks=yes",
            "-C",
            "panic=abort",
        ])
        .arg(&source_file)
        .arg("-o")
        .arg(&object)
        .output()?;
    std::fs::write(root.join(format!("{name}.stderr")), &result.stderr)?;
    if object.exists() {
        std::fs::remove_file(object)?;
    }
    assert_eq!(
        result.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn constant_value_instances_match_object_qualified_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in VALUE_CASES {
        object_outcome(name, source, accepted)?;
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
fn repeated_values_share_only_their_complete_instance() -> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn main(){let _=S::<3>::C+S::<5>::C+S::<3>::C;}";
    object_outcome("value-cache-identity", source, true)?;
    let (mut report, mut facts) = semantic::analyze(source)?;
    let mut uses = facts
        .constant_uses
        .iter()
        .map(|(key, instance)| (*key, *instance))
        .collect::<Vec<_>>();
    uses.sort_by_key(|(key, _)| (key.file.0, key.node.0));
    assert_eq!(uses.len(), 3);
    assert_eq!(uses[0].1, uses[2].1);
    assert_ne!(uses[0].1.arguments, uses[1].1.arguments);
    assert_eq!(uses[0].1.definition, uses[1].1.definition);
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let reader = rustgm::sema::values::Reader::new(&facts);
    for (index, (_, instance)) in uses.into_iter().enumerate() {
        assert!(reader.definition(instance.definition).is_none());
        assert_eq!(
            reader
                .associated(instance)
                .ok_or("missing instance value")?
                .as_scalar()?,
            rustgm::sema::model::Scalar::Integer(if index == 1 { 5 } else { 3 })
        );
    }
    let invalid = rustgm::sema::values::Instance {
        definition: DefId(u32::MAX),
        arguments: rustgm::types::GenericArgs::EMPTY,
    };
    assert!(
        rustgm::sema::values::Evaluator::new(&report, &mut facts)
            .associated(invalid)
            .is_err()
    );
    Ok(())
}

#[test]
fn selected_constant_patterns_resolve_owner_instances() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "trait-pattern",
            "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}fn main(){let _=match 9u8{S::C=>true,_=>false};}",
        ),
        (
            "generic-pattern",
            "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn main(){let _=match 3usize{S::<3>::C=>true,_=>false};}",
        ),
    ] {
        object_outcome(name, source, true)?;
        let (mut report, mut facts) = semantic::analyze(source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build(&report, &mut facts, entry)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_constant_instances_preserve_native_values() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "constant-instances",
        &[
            (
                "selected-values",
                "trait A{const C:u8=7;}struct S;struct T;impl A for S{}impl A for T{const C:u8=11;}fn main(){check(<S as A>::C==7);check(<T as A>::C==11);check(<S as A>::C==7)}",
                false,
            ),
            (
                "trait-identity",
                "trait A{const C:u8=7;}trait B{const C:u8=9;}struct S;impl A for S{}impl B for S{}fn main(){check(<S as A>::C==7);check(<S as B>::C==9)}",
                false,
            ),
            (
                "const-instances",
                "struct S<const N:usize>;impl<const N:usize>S<N>{const A:usize=N;const B:usize=Self::A+1;}fn main(){check(S::<3>::B==4);check(S::<5>::B==6);check(S::<3>::B==4)}",
                false,
            ),
            (
                "generic-function",
                "trait A{const C:u8;}impl A for u8{const C:u8=7;}impl A for u16{const C:u8=9;}fn get<T:A>()->u8{T::C}fn main(){check(get::<u8>()==7);check(get::<u16>()==9)}",
                false,
            ),
            (
                "trait-const-arguments",
                "trait A<const N:usize>{const C:usize=N;const D:usize=Self::C+1;}impl<const N:usize>A<N> for u8{}impl<const N:usize>A<N> for u16{const C:usize=N+7;}fn main(){check(<u8 as A<3>>::D==4);check(<u16 as A<5>>::D==13)}",
                false,
            ),
            (
                "products",
                "struct S<const N:usize>;impl<const N:usize>S<N>{const A:(usize,u8)=(N,7);const B:[u8;N]=[9;N];}fn main(){check(S::<3>::A.0==3);check(S::<5>::A.0==5);check(S::<3>::B[2]==9);check(S::<5>::B[4]==9)}",
                false,
            ),
            (
                "references",
                "struct S<const N:usize>;impl<const N:usize>S<N>{const C:&'static usize=&N;}fn main(){check(*S::<3>::C==3);check(*S::<5>::C==5);check(*S::<3>::C==3)}",
                false,
            ),
            (
                "global-initializer",
                "trait A{const C:u8=7;}impl A for u8{}const X:u8=<u8 as A>::C;fn main(){check(X==7)}",
                false,
            ),
            (
                "constructor",
                "struct V(usize);struct S<const N:usize>;impl<const N:usize>S<N>{const C:V=V(N);}fn main(){check(S::<3>::C.0==3);check(S::<5>::C.0==5)}",
                false,
            ),
            (
                "callable",
                "fn f()->u8{7}struct S;impl S{const C:fn()->u8=f;}fn main(){check(S::C()==7)}",
                false,
            ),
            (
                "trait-callable",
                "trait A{fn f()->u8{7}const C:fn()->u8=Self::f;}impl A for u8{fn f()->u8{9}}impl A for u16{fn f()->u8{11}}fn main(){check(<u8 as A>::C()==9);check(<u16 as A>::C()==11)}",
                false,
            ),
            (
                "trait-callable-product",
                "trait A{fn f()->u8{7}const C:(fn()->u8,u8)=(Self::f,3);}impl A for u8{fn f()->u8{9}}fn main(){check((<u8 as A>::C.0)()==9);check(<u8 as A>::C.1==3)}",
                false,
            ),
            (
                "trait-required-callable",
                "trait A{fn f()->u8;const C:fn()->u8=Self::f;}impl A for u8{fn f()->u8{9}}fn main(){check(<u8 as A>::C()==9)}",
                false,
            ),
            (
                "trait-mixed-callable",
                "trait A<const N:usize>{fn f()->usize{N}const C:fn()->usize=Self::f;}impl<const N:usize>A<N>for u8{fn f()->usize{N+1}}fn main(){check(<u8 as A<3>>::C()==4);check(<u8 as A<5>>::C()==6)}",
                false,
            ),
            (
                "trait-recursive-callable",
                "trait A{fn f(x:u8)->u8{if x==0{7}else{Self::C(x-1)}}const C:fn(u8)->u8=Self::f;}impl A for u8{}fn main(){check(<u8 as A>::C(2)==7)}",
                false,
            ),
            (
                "inherent-pattern",
                "struct S;impl S{const C:u8=9;}fn f(x:u8)->bool{match x{S::C=>true,_=>false}}fn main(){check(f(9));check(!f(7))}",
                false,
            ),
            (
                "text",
                "struct S;impl S{const A:&'static str=\"foo\";const B:&'static [u8;3]=b\"bar\";}fn main(){check(match S::A{\"foo\"=>true,_=>false});check(S::B[1]==97)}",
                false,
            ),
        ],
    )
}

// Object-qualified before adoption; generic result ICE probes stay quarantined.
const PATTERN_CASES: &[(&str, &str, bool)] = &[
    (
        "trait-shorthand",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}fn f(x:u8)->bool{match x{S::C=>true,_=>false}}fn main(){check(f(9));check(!f(7))}",
        true,
    ),
    (
        "trait-qualified",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}fn f(x:u8)->bool{match x{<S as A>::C=>true,_=>false}}fn main(){check(f(9));check(!f(7))}",
        true,
    ),
    (
        "inherent-const",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn f(x:usize)->bool{match x{S::<3>::C=>true,_=>false}}fn main(){check(f(3));check(!f(5))}",
        true,
    ),
    (
        "inherent-erased",
        "struct S<T>(T);impl<T>S<T>{const C:u8=7;}fn f(x:u8)->bool{match x{S::<u16>::C=>true,_=>false}}fn main(){check(f(7));check(!f(9))}",
        true,
    ),
    (
        "trait-const",
        "trait A<const N:usize>{const C:usize=N;}impl<const N:usize>A<N>for u8{}fn f(x:usize)->bool{match x{<u8 as A<5>>::C=>true,_=>false}}fn main(){check(f(5));check(!f(3))}",
        true,
    ),
    (
        "trait-inferred",
        "trait A{const C:Self;}impl A for u8{const C:Self=7;}fn f(x:u8)->bool{match x{A::C=>true,_=>false}}fn main(){check(f(7));check(!f(9))}",
        true,
    ),
    (
        "lifetime-only",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8=7;}fn f<'a>(_:&'a u8,x:u8)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){check(f(&0,7));check(!f(&0,9))}",
        true,
    ),
    (
        "dependent-type",
        "struct S<T>(T);impl<T>S<T>{const C:u8=7;}fn f<T>(x:u8)->bool{match x{S::<T>::C=>true,_=>false}}fn main(){}",
        true,
    ),
    (
        "dependent-const",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn f<const N:usize>(x:usize)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "dependent-trait",
        "trait A{const C:u8;}fn f<T:A>(x:u8)->bool{match x{T::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "lifetime-bound-debt",
        "struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:u8=7;}fn f<'a>(_:&'a u8,x:u8)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "missing-copy",
        "struct S<T>(T);impl<T:Copy>S<T>{const C:u8=7;}struct N;fn f(x:u8)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "product",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:(usize,u8)=(N,7);}fn f(x:(usize,u8))->bool{match x{S::<3>::C=>true,_=>false}}fn main(){check(f((3,7)));check(!f((5,7)))}",
        true,
    ),
    (
        "reference",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:&'static usize=&N;}fn f(x:&usize)->bool{match x{S::<3>::C=>true,_=>false}}fn main(){check(f(&3));check(!f(&5))}",
        true,
    ),
    (
        "global-alias",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}const P:u8=<S as A>::C;fn f(x:u8)->bool{match x{P=>true,_=>false}}fn main(){check(f(9));check(!f(7))}",
        true,
    ),
    (
        "range",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N;}fn f(x:usize)->bool{match x{S::<3>::C..=S::<5>::C=>true,_=>false}}fn main(){check(f(4));check(!f(7))}",
        true,
    ),
    (
        "erased-type-called",
        "struct S<T>(T);impl<T>S<T>{const C:u8=7;}fn f<T>(x:u8)->bool{match x{S::<T>::C=>true,_=>false}}fn main(){check(f::<u8>(7));check(!f::<u16>(9))}",
        true,
    ),
    (
        "erased-const-called",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:u8=7;}fn f<const N:usize>(x:u8)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(7));check(!f::<5>(9))}",
        true,
    ),
    (
        "generic-trait-owner",
        "trait A{const C:u8=7;}struct S<T>(T);impl<T>A for S<T>{}fn f<T>(x:u8)->bool{match x{<S<T> as A>::C=>true,_=>false}}fn main(){check(f::<u8>(7));check(!f::<u16>(9))}",
        true,
    ),
    (
        "generic-bounded-owner",
        "trait A{const C:u8=7;}struct S<T>(T);impl<T:Copy>A for S<T>{}fn f<T:Copy>(x:u8)->bool{match x{<S<T> as A>::C=>true,_=>false}}fn main(){check(f::<u8>(7));check(!f::<u16>(9))}",
        true,
    ),
    (
        "generic-missing-bound",
        "trait A{const C:u8=7;}struct S<T>(T);impl<T:Copy>A for S<T>{}fn f<T>(x:u8)->bool{match x{<S<T> as A>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "blanket-trait",
        "trait A{const C:u8=7;}impl<T>A for T{}fn f<T>(x:u8)->bool{match x{T::C=>true,_=>false}}fn main(){check(f::<u8>(7));check(!f::<u16>(9))}",
        true,
    ),
    (
        "blanket-trait-bound",
        "trait A{const C:u8=7;}impl<T>A for T{}fn f<T:A>(x:u8)->bool{match x{T::C=>true,_=>false}}fn main(){check(f::<u8>(7));check(!f::<u16>(9))}",
        false,
    ),
    (
        "lifetime-bound-proven",
        "struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:u8=7;}fn f<'a:'static>(_:&'a u8,x:u8)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){check(f(&0,7));check(!f(&0,9))}",
        true,
    ),
    (
        "lifetime-value",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'a u8=&7;}fn f<'a>(x:&'a u8)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){check(f(&7));check(!f(&9))}",
        true,
    ),
    (
        "unused-branch",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=if true{7}else{N};}fn f<const N:usize>(x:usize)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(7))}",
        false,
    ),
    (
        "cancelled-value",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize=N-N;}fn f<const N:usize>(x:usize)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "concrete-assumption",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}fn f<T>(x:u8)->bool where S:A{match x{S::C=>true,_=>false}}fn main(){check(f::<u8>(9));check(!f::<u8>(7))}",
        true,
    ),
    (
        "generic-default-dependency",
        "trait A{const C:u8=Self::D;const D:u8=7;}struct S<T>(T);impl<T>A for S<T>{const D:u8=9;}fn f<T>(x:u8)->bool{match x{<S<T>as A>::C=>true,_=>false}}fn main(){check(f::<u8>(9));check(!f::<u8>(7))}",
        true,
    ),
    (
        "erased-enum-result",
        "#[derive(PartialEq,Eq)]enum K{A,B}struct S<const N:usize>;impl<const N:usize>S<N>{const C:K=K::A;}fn f<const N:usize>(x:K)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(K::A));check(!f::<3>(K::B))}",
        true,
    ),
    (
        "concrete-enum-result",
        "#[derive(PartialEq,Eq)]enum K<const N:usize>{A,B}struct S<const N:usize>;impl<const N:usize>S<N>{const C:K<N>=K::A;}fn f(x:K<3>)->bool{match x{S::<3>::C=>true,_=>false}}fn main(){check(f(K::A));check(!f(K::B))}",
        true,
    ),
    (
        "nominal-product",
        "#[derive(PartialEq,Eq)]struct K<T>(T);struct S<T>(T);impl S<u8>{const C:K<u8>=K(7);}fn f(x:K<u8>)->bool{match x{S::<u8>::C=>true,_=>false}}fn main(){check(f(K(7)));check(!f(K(9)))}",
        true,
    ),
    (
        "multi-global-alias",
        "trait A{const C:u8=7;}struct S;impl A for S{const C:u8=9;}const P:u8=<S as A>::C;const Q:u8=P;fn f(x:u8)->bool{match x{Q=>true,_=>false}}fn main(){check(f(9));check(!f(7))}",
        true,
    ),
    (
        "nonexhaustive",
        "trait A{const C:bool=false;}impl A for u8{const C:bool=true;}fn f(x:bool){match x{<u8 as A>::C=>()}}fn main(){}",
        false,
    ),
    (
        "exhaustive-bool",
        "trait A{const C:bool=false;}impl A for u8{const C:bool=true;}fn f(x:bool)->u8{match x{false=>7,<u8 as A>::C=>9}}fn main(){check(f(false)==7);check(f(true)==9)}",
        true,
    ),
    (
        "refutable-binding",
        "trait A{const C:u8=7;}impl A for u8{}fn main(){let <u8 as A>::C=7u8;}",
        false,
    ),
    (
        "skipped-dependent-constant",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const D:usize=N;const C:usize=if true{7}else{Self::D};}fn f<const N:usize>(x:usize)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(7))}",
        false,
    ),
    (
        "skipped-fixed-constant",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const D:usize=9;const C:usize=if true{7}else{Self::D};}fn f<const N:usize>(x:usize)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(7))}",
        true,
    ),
    (
        "skipped-fault",
        "struct S<const N:usize>;impl<const N:usize>S<N>{const C:u8=if true{7}else{1/0};}fn f<const N:usize>(x:u8)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){check(f::<3>(7))}",
        true,
    ),
    (
        "nominal-reference-product",
        "#[derive(PartialEq,Eq)]struct K<'a>(&'a u8);struct S<'a>(&'a u8);impl<'a>S<'a>{const C:K<'a>=K(&7);}fn f<'a>(x:K<'a>)->bool{match x{<S<'a>>::C=>true,_=>false}}fn main(){check(f(K(&7)));check(!f(K(&9)))}",
        true,
    ),
    (
        "trait-lifetime-debt",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a:'static>A for S<'a>{const C:u8=9;}fn f<'a>(_:&'a u8,x:u8)->bool{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "trait-lifetime-proven",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a:'static>A for S<'a>{const C:u8=9;}fn f<'a:'static>(_:&'a u8,x:u8)->bool{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){check(f(&0,9));check(!f(&0,7))}",
        true,
    ),
    (
        "trait-lifetime-scopes",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a:'static>A for S<'a>{const C:u8=9;}fn good<'a:'static>(_:&'a u8,x:u8)->bool{match x{<S<'a>as A>::C=>true,_=>false}}fn bad<'a>(_:&'a u8,x:u8)->bool{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){}",
        false,
    ),
    (
        "trait-reference-lifetime",
        "trait A<'a>{const C:&'a u8=&7;}impl<'a>A<'a>for u8{const C:&'a u8=&9;}fn f<'a>(x:&'a u8)->bool{match x{<u8 as A<'a>>::C=>true,_=>false}}fn main(){check(f(&9));check(!f(&7))}",
        true,
    ),
    (
        "trait-lifetime-assumed",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a:'static>A for S<'a>{const C:u8=9;}fn f<'a>(_:&'a u8,x:u8)->bool where S<'a>:A{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){check(f(&0,9));check(!f(&0,7))}",
        false,
    ),
    (
        "assumed-unrestricted-lifetime",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a>A for S<'a>{}fn f<'a>(_:&'a u8,x:u8)->bool where S<'a>:A{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){check(f(&0,7))}",
        false,
    ),
    (
        "assumed-proven-static-lifetime",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a:'static>A for S<'a>{}fn f<'a:'static>(_:&'a u8,x:u8)->bool where S<'a>:A{match x{<S<'a>as A>::C=>true,_=>false}}fn main(){check(f(&0,7))}",
        false,
    ),
    (
        "concrete-static-with-assumption",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a>A for S<'a>{}fn f<'a>(_:&'a u8,x:u8)->bool where S<'a>:A{match x{<S<'static>as A>::C=>true,_=>false}}fn main(){check(f(&0,7))}",
        false,
    ),
    (
        "trait-lifetime-argument-assumed",
        "trait A<'a>{const C:u8=7;}impl<'a>A<'a>for u8{}fn f<'a>(_:&'a u8,x:u8)->bool where u8:A<'a>{match x{<u8 as A<'a>>::C=>true,_=>false}}fn main(){check(f(&0,7))}",
        false,
    ),
    (
        "trait-static-argument-with-assumption",
        "trait A<'a>{const C:u8=7;}impl<'a>A<'a>for u8{}fn f<'a>(_:&'a u8,x:u8)->bool where u8:A<'a>{match x{<u8 as A<'static>>::C=>true,_=>false}}fn main(){check(f(&0,7))}",
        false,
    ),
    (
        "closed-trait-static-argument",
        "trait A<'a>{const C:u8=7;}impl<'a>A<'a>for u8{}fn f(x:u8)->bool where u8:A<'static>{match x{<u8 as A<'static>>::C=>true,_=>false}}fn main(){check(f(7));check(!f(9))}",
        false,
    ),
    (
        "closed-self-static-argument",
        "trait A{const C:u8=7;}struct S<'a>(&'a u8);impl<'a>A for S<'a>{}fn f<'a>(_:&'a u8,x:u8)->bool where S<'static>:A{match x{<S<'static>as A>::C=>true,_=>false}}fn main(){check(f(&0,7));check(!f(&0,9))}",
        false,
    ),
];

#[test]
fn associated_patterns_match_qualified_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in PATTERN_CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("pattern-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
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
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_patterns_preserve_native_matching() -> Result<(), Box<dyn std::error::Error>> {
    let cases = PATTERN_CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("associated-patterns", &cases)
}
