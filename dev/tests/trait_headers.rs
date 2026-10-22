//! Required trait signatures and default bodies have distinct proof obligations.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    resolve::model::{DefId, DefKind},
    sema::associated::OwnerKind,
    types::{ItemId, Kind, parameters::ParameterKind},
};

fn prove(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let main = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Function && report.package.names.get(d.name) == "main")
        .ok_or("missing main")?;
    rustgm::lower::instances::build(&report, &mut facts, DefId(u32::try_from(main)?))?;
    Ok(())
}

#[test]
fn trait_methods_validate_visibility_and_safety_after_configuration()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in [
        (
            "trait-public-required",
            "trait A{pub fn f(&self);}fn main(){}",
            false,
        ),
        (
            "trait-restricted-default",
            "trait A{pub(crate) fn f(&self){}}fn main(){}",
            false,
        ),
        (
            "trait-safe-required",
            "trait A{safe fn f(&self);}fn main(){}",
            false,
        ),
        (
            "trait-safe-default",
            "trait A{safe fn f(&self){}}fn main(){}",
            false,
        ),
        (
            "public-trait-inherited-method",
            "pub trait A{fn f(&self){}}fn main(){}",
            true,
        ),
        (
            "unsafe-trait-unsafe-method",
            "unsafe trait A{unsafe fn f(&self){}}fn main(){}",
            true,
        ),
        (
            "configured-out-public-method",
            "trait A{#[cfg(any())]pub fn f(&self);}fn main(){}",
            true,
        ),
        (
            "configured-out-safe-method",
            "trait A{#[cfg(any())]safe fn f(&self);}fn main(){}",
            true,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
        let result = prove(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn trait_implementations_require_conforming_methods_or_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in [
        (
            "missing-required-trait-method",
            "trait A{fn required(&self);}struct S;impl A for S{}fn main(){}",
            false,
        ),
        (
            "wrong-trait-method-result",
            "trait A{fn required(&self)->u8;}struct S;impl A for S{fn required(&self)->u16{7}}fn main(){}",
            false,
        ),
        (
            "unimplemented-trait-conformance",
            "trait A{fn required(&self)->u8{7}}struct S;impl A for S{}fn main(){}",
            true,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
        let result = prove(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn trait_self_and_declared_parameters_keep_separate_original_identities()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "trait A<'a,T:Copy,const N:usize>{fn pair<U:Copy>(&self,x:T,y:U)->(T,U){(x,y)}fn required(self)->Self;}fn main(){}";
    rustc::qualify("trait-parameter-identity", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let owner = facts
        .associated
        .owners()
        .first()
        .ok_or("missing trait owner")?;
    let OwnerKind::Trait { definition, .. } = owner.kind else {
        return Err("unexpected owner".into());
    };
    let schema = facts
        .associated
        .schema(definition)
        .ok_or("missing trait schema")?;
    let Some(Kind::Parameter(id)) = facts.types.kind(schema.self_type) else {
        return Err("missing trait Self parameter".into());
    };
    let parameter = facts
        .types
        .parameter(id)
        .ok_or("missing trait Self identity")?;
    assert_eq!(parameter.owner, ItemId(owner.self_definition.0));
    assert_eq!(parameter.index, 0);
    assert_eq!(parameter.kind, ParameterKind::Type { sized: false });
    assert_eq!(schema.captures, 1);
    let method = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "pair"
        })
        .ok_or("missing default method")?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(method.definition.0))
        .ok_or("missing callable parameters")?;
    let identities = parameters
        .iter()
        .map(|id| facts.types.parameter(*id).map(|p| (p.owner, p.index)))
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        [
            Some((ItemId(owner.self_definition.0), 0)),
            Some((ItemId(definition.0), 1)),
            Some((ItemId(definition.0), 2)),
            Some((ItemId(method.definition.0), 0))
        ]
    );
    let lifetimes = facts
        .lifetimes
        .functions
        .get(&method.definition)
        .ok_or("missing method lifetime scheme")?;
    assert_eq!(lifetimes.captures, 1);
    assert_eq!(lifetimes.late, [1]);
    let templates = rustgm::body::templates::build(&report, &facts)?;
    assert_eq!(templates.bodies.len(), 1);
    assert_eq!(
        templates.bodies[0].body.source,
        method.body.ok_or("missing default body")?
    );
    prove(source)
}

#[test]
fn required_signatures_allow_unsized_values_without_creating_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "required-unsized-self",
            "trait A{fn take(self);fn make()->Self;fn pair(x:Self)->Self;}fn main(){}",
        ),
        (
            "required-unsized-concrete",
            "trait A{fn take(x:str)->str;}fn main(){}",
        ),
        (
            "required-unsized-generic",
            "trait A<T:?Sized>{fn take(x:T)->T;}fn main(){}",
        ),
        (
            "required-wildcard",
            "trait A{fn take(_:u8,x:u8);}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, true)?;
        prove(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

#[test]
fn default_bodies_use_declared_sized_copy_and_lifetime_assumptions()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "borrow-default",
            "trait A{fn shared(&self,x:&u8)->&Self{self}fn nested(self:&&Self)->u8{7}}fn main(){}",
        ),
        (
            "sized-super",
            "trait A:Sized{fn take(self)->Self{self}}fn main(){}",
        ),
        (
            "sized-where",
            "trait A where Self:Sized{fn take(self)->Self{self}}fn main(){}",
        ),
        (
            "sized-method",
            "trait A{fn take(self)->Self where Self:Sized{self}}fn main(){}",
        ),
        (
            "copy-super",
            "trait A:Copy{fn twice(self)->(Self,Self){(self,self)}}fn main(){}",
        ),
        (
            "copy-method",
            "trait A{fn twice(self)->(Self,Self)where Self:Copy{(self,self)}}fn main(){}",
        ),
        (
            "generic-owners",
            "trait A<'a,T:Copy,const N:usize>{fn pair<U:Copy>(&self,x:T,y:U)->(T,U){(x,y)}fn count(&self)->usize{N}fn borrow<'b>(&'b self,x:&'a T)->&'a T{x}}fn main(){}",
        ),
        (
            "generic-default-self",
            "trait A<T:?Sized=Self>{fn take(&self,x:&T)->u8{7}}fn main(){}",
        ),
        (
            "unsafe-trait-safe-method",
            "unsafe trait A{fn shared(&self)->u8{7}}fn main(){}",
        ),
        (
            "default-destructure",
            "trait A{fn take((a,b):(u8,u8))->u8{a+b}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, true)?;
        prove(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

#[test]
fn invalid_unused_defaults_fail_the_ordinary_proof_pipeline()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "default-unsized-self",
            "trait A{fn take(self){}}fn main(){}",
        ),
        (
            "default-unsized-result",
            "trait A{fn make()->Self{loop{}}}fn main(){}",
        ),
        (
            "default-unsized-generic",
            "trait A<T:?Sized>{fn take(x:T){}}fn main(){}",
        ),
        (
            "default-move",
            "trait A:Sized{fn twice(self)->(Self,Self){(self,self)}}fn main(){}",
        ),
        (
            "default-lifetime",
            "trait A{fn take(&self,x:&Self)->&Self{x}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn invalid_required_signatures_and_trait_headers_are_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "required-destructure",
            "trait A{fn take((a,b):(u8,u8));}fn main(){}",
        ),
        (
            "required-ref-pattern",
            "trait A{fn take(ref a:u8);}fn main(){}",
        ),
        (
            "required-mut-pattern",
            "trait A{fn take(mut a:u8);}fn main(){}",
        ),
        (
            "const-required",
            "trait A{const fn take(&self);}fn main(){}",
        ),
        (
            "const-default",
            "trait A{const fn take(&self)->u8{7}}fn main(){}",
        ),
        ("relaxed-super", "trait A:?Sized{}fn main(){}"),
        (
            "trivial-false-bound",
            "trait A where str:Sized{}fn main(){}",
        ),
        (
            "unsized-tuple-required",
            "trait A{fn take()->(Self,Self);}fn main(){}",
        ),
        ("foreign-receiver", "trait A{fn take(self:u8);}fn main(){}"),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn trait_defaults_and_empty_owner_bounds_require_independent_declaration_checks()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in [
        (
            "required-mut-self",
            "trait A{fn take(mut self);}fn main(){}",
            false,
        ),
        (
            "required-mut-explicit-self",
            "trait A{fn take(mut self:&Self);}fn main(){}",
            false,
        ),
        (
            "required-mut-reference",
            "trait A{fn take(&mut self);}fn main(){}",
            true,
        ),
        (
            "default-unsized-closed",
            "trait A<T=str>{}fn main(){}",
            false,
        ),
        (
            "default-copy-closed",
            "struct S;trait A<T:Copy=S>{}fn main(){}",
            false,
        ),
        (
            "default-unsized-relaxed",
            "trait A<T:?Sized=str>{fn take(x:&T)->u8{7}}fn main(){}",
            true,
        ),
        (
            "default-ill-formed",
            "trait A<T=[str;0]>{}fn main(){}",
            false,
        ),
        (
            "inherent-trivial-bound",
            "struct S;impl S where str:Sized{}fn main(){}",
            false,
        ),
        (
            "inherent-closed-copy-bound",
            "struct S;impl S where S:Copy{}fn main(){}",
            false,
        ),
        (
            "trait-closed-copy-bound",
            "struct S;trait A where S:Copy{}fn main(){}",
            false,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
        let result = prove(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
