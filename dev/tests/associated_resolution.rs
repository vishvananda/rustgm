//! Associated lookup consumes final namespace access and anonymous import facts.
#[path = "helpers/resolve.rs"]
mod names;
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::resolve::model::{Access, DefKind, Namespace, Origin};

#[test]
fn trait_members_inherit_access_without_exposing_nested_or_inherent_items()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "mod m{pub struct S;pub trait A{type Value;const K:u8;fn get(&self)->u8{fn helper()->u8{7}helper()}}impl A for S{type Value=u8;const K:u8=1;}impl S{fn hidden(&self){}}}fn main(){let _=m::A::get(&m::S);}";
    rustc::qualify("associated-public-access", source, true)?;
    for expanded in [false, true] {
        let report = names::resolve(source, expanded)?;
        let definitions = &report.resolution.definitions;
        let owner = definitions
            .iter()
            .find(|d| d.kind == DefKind::Trait)
            .ok_or("missing trait")?;
        let members = owner.members.ok_or("missing trait scope")?;
        let mut count = 0;
        for definition in definitions {
            if definition.scope == members && definition.kind != DefKind::SelfType {
                assert_eq!(definition.access, Access::Public);
                count += 1;
            }
            if matches!(
                report.package.names.get(definition.name),
                "helper" | "hidden"
            ) {
                assert!(matches!(definition.access, Access::Within(_)));
            }
        }
        assert_eq!(count, 3);
    }
    Ok(())
}

#[test]
fn restricted_traits_and_reexports_preserve_member_access() -> Result<(), Box<dyn std::error::Error>>
{
    for (name, source, accepted) in [
        (
            "associated-reexport",
            "mod m{pub struct S;pub trait A{fn get(&self){}}impl A for S{}}use m::A as B;fn main(){B::get(&m::S);}",
            true,
        ),
        (
            "associated-restricted",
            "mod outer{mod inner{pub struct S;pub(super) trait A{fn get(&self){}}impl A for S{}}pub fn call(){inner::A::get(&inner::S);}}fn main(){outer::call();}",
            true,
        ),
        (
            "associated-private",
            "mod m{pub struct S;trait A{fn get(&self){}}impl A for S{}}fn main(){m::A::get(&m::S);}",
            false,
        ),
        (
            "associated-restricted-outside",
            "mod outer{pub mod inner{pub struct S;pub(super) trait A{fn get(&self){}}impl A for S{}}}fn main(){outer::inner::A::get(&outer::inner::S);}",
            false,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
        for expanded in [false, true] {
            assert_eq!(names::resolve(source, expanded).is_ok(), accepted, "{name}");
        }
    }
    Ok(())
}

#[test]
fn anonymous_imports_keep_identity_and_scope_without_binding_an_underscore()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "mod m{pub trait A{}pub trait B{}pub struct S;}use m::A as _;use m::A as _;fn main(){{use m::B as _;}{use m::S as _;}}";
    rustc::qualify("associated-anonymous-scopes", source, true)?;
    for expanded in [false, true] {
        let report = names::resolve(source, expanded)?;
        let facts = &report.resolution;
        assert_eq!(facts.anonymous_types.len(), 4);
        let bindings = (0..4)
            .map(|i| {
                facts
                    .anonymous_types
                    .get(&i)
                    .ok_or("missing anonymous type")
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(bindings[0].target, bindings[1].target);
        assert_ne!(bindings[0].target, bindings[2].target);
        assert_ne!(facts.imports[2].scope, facts.imports[3].scope);
        assert_ne!(facts.imports[0].scope, facts.imports[2].scope);
        assert_eq!(
            facts.definitions[bindings[3].target.0 as usize].kind,
            DefKind::Struct
        );
        for (index, binding) in bindings.into_iter().enumerate() {
            assert_eq!(binding.origin, Origin::Import(u32::try_from(index)?));
        }
        assert!(facts.scopes.iter().all(|scope| {
            scope
                .bindings
                .keys()
                .all(|(name, _)| report.package.names.get(*name) != "_")
        }));
    }
    Ok(())
}

#[test]
fn anonymous_imports_follow_replaced_globs_and_do_not_retain_provisional_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "use a::A as _;mod a{pub use crate::b::*;pub use crate::d::A;}mod b{pub trait A{}}mod d{pub use crate::e::A;}mod e{pub trait A{}}fn main(){}";
    rustc::qualify("associated-anonymous-retarget", source, true)?;
    for expanded in [false, true] {
        let mut report = names::resolve(source, expanded)?;
        let name = report.package.names.intern("A")?;
        let module = report
            .package
            .modules
            .iter()
            .position(|m| report.package.names.get(m.name) == "e")
            .ok_or("missing final module")?;
        let scope = report.resolution.module_scopes[module];
        let final_target =
            report.resolution.scopes[scope.0 as usize].bindings[&(name, Namespace::Type)].target;
        assert_eq!(report.resolution.anonymous_types.len(), 1);
        assert_eq!(report.resolution.anonymous_types[&0].target, final_target);
    }
    Ok(())
}

#[test]
fn expanded_trait_members_and_anonymous_imports_keep_their_owner_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "macro_rules! method{()=>{fn get(&self)->u8{7}}}mod m{pub struct S;pub trait A{method!();}impl A for S{}}macro_rules! import{()=>{use m::A as _;}}fn main(){import!();let _=m::A::get(&m::S);}";
    rustc::qualify("associated-expanded-access", source, true)?;
    let report = names::resolve(source, true)?;
    let facts = &report.resolution;
    let method = facts
        .definitions
        .iter()
        .find(|d| report.package.names.get(d.name) == "get")
        .ok_or("missing generated method")?;
    assert_eq!(method.access, Access::Public);
    assert!(facts.scopes[method.scope.0 as usize].associated);
    assert_eq!(facts.anonymous_types.len(), 1);
    let (&index, &binding) = facts
        .anonymous_types
        .iter()
        .next()
        .ok_or("missing generated import")?;
    assert_eq!(binding.origin, Origin::Import(u32::try_from(index)?));
    assert_eq!(
        facts.introductions.binding(binding),
        *facts
            .introductions
            .imports
            .get(&index)
            .ok_or("missing import hygiene")?
    );
    assert_eq!(
        facts.definitions[binding.target.0 as usize].kind,
        DefKind::Trait
    );
    Ok(())
}
