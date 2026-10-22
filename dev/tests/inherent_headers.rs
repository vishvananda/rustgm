//! Method declarations reuse the ordinary body and generic proof pipelines.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    resolve::model::{DefId, DefKind},
    sema::model::{Declarations, Function},
    types::{ItemId, Kind, TypeId},
};

fn method<'a>(
    report: &rustgm::resolve::expansion::Report,
    facts: &'a Declarations,
    name: &str,
) -> Result<&'a Function, &'static str> {
    facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == name
        })
        .ok_or("missing method")
}
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
fn inherent_receivers_keep_whole_values_mutability_and_output_elision()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S(u8);impl S{fn take(self)->u8{self.0}fn change(mut self)->u8{self.0=8;self.0}fn shared(&self,x:&u8)->&u8{&self.0}fn explicit(self:&Self,x:&u8)->&u8{&self.0}fn named<'a>(&'a self,x:&u8)->&'a u8{&self.0}fn nested(self:&&Self)->u8{self.0}fn make()->Self{S(7)}}fn main(){}";
    rustc::qualify("inherent-receivers", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    assert_eq!(facts.receivers.len(), 6);
    for name in ["shared", "explicit", "named"] {
        let function = method(&report, &facts, name)?;
        let Some(Kind::Function {
            parameters, result, ..
        }) = facts.types.kind(function.signature)
        else {
            return Err("missing signature".into());
        };
        let parameters = facts.types.list(parameters).ok_or("missing parameters")?;
        let region = |ty: TypeId| match facts.types.kind(ty) {
            Some(Kind::Reference { region, .. }) => Ok(region),
            _ => Err("missing reference"),
        };
        assert_eq!(region(result)?, region(parameters[0])?);
        assert_ne!(region(result)?, region(parameters[1])?);
    }
    for (name, mutable) in [("take", false), ("change", true)] {
        let function = method(&report, &facts, name)?;
        let parameter = report
            .package
            .unit(function.node.file)
            .configured
            .ast
            .list(function.parameters)[0];
        let receiver = facts
            .receivers
            .get(&rustgm::resolve::unit::NodeKey {
                node: parameter,
                ..function.node
            })
            .ok_or("missing receiver")?;
        assert_eq!(receiver.mutable, mutable);
        assert_eq!(
            receiver.ty,
            facts
                .associated
                .schema(function.definition)
                .ok_or("missing owner")?
                .self_type
        );
    }
    prove(source)
}

#[test]
fn method_templates_keep_impl_and_method_parameter_owners_and_lifetime_binding_times()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S<'a,T:Copy,const N:usize>(&'a T,[u8;N]);impl<'a,T:Copy,const N:usize>S<'a,T,N>{fn pair<U:Copy>(&self,x:U)->(T,U){(*self.0,x)}fn count(&self)->usize{N}fn value(self)->T{*self.0}fn make(x:&'a T,ys:[u8;N])->Self{S(x,ys)}}fn main(){}";
    rustc::qualify("inherent-generic-environments", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let function = method(&report, &facts, "pair")?;
    let owner = facts
        .associated
        .get(
            facts
                .associated
                .owner(function.definition)
                .ok_or("missing owner")?,
        )
        .ok_or("invalid owner")?;
    let scheme = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("missing lifetime scheme")?;
    assert_eq!(scheme.captures, 1);
    assert_eq!(scheme.total, 2);
    assert!(scheme.early.is_empty());
    assert_eq!(scheme.late, [1]);
    let callable = facts
        .lifetimes
        .callables
        .get(ItemId(function.definition.0))
        .ok_or("missing callable")?;
    assert_eq!(callable.parameters, 1);
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(function.definition.0))
        .ok_or("missing callable parameters")?;
    assert_eq!(parameters.len(), 3);
    let owners = parameters
        .iter()
        .map(|id| facts.types.parameter(*id).map(|p| p.owner))
        .collect::<Vec<_>>();
    assert_eq!(
        owners,
        [
            Some(ItemId(owner.self_definition.0)),
            Some(ItemId(owner.self_definition.0)),
            Some(ItemId(function.definition.0))
        ]
    );
    prove(source)
}

#[test]
fn independently_invalid_receiver_and_owner_headers_fail_when_unused()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "foreign-self",
            "struct S;impl S{fn get(self:&u8){}}fn main(){}",
        ),
        ("free-self", "fn get(self:u8){}fn main(){}"),
        (
            "late-self",
            "struct S;impl S{fn get(x:u8,self){}}fn main(){}",
        ),
        (
            "unconstrained-type",
            "struct S;impl<T>S{fn get(&self){}}fn main(){}",
        ),
        (
            "unconstrained-const",
            "struct S;impl<const N:usize>S{fn get(&self){}}fn main(){}",
        ),
        (
            "unsafe-inherent",
            "struct S;unsafe impl S{fn get(&self){}}fn main(){}",
        ),
        ("primitive-inherent", "impl u8{fn get(&self){}}fn main(){}"),
        (
            "nested-elision",
            "struct S(u8);impl S{fn get(self:&&Self)->&u8{&self.0}}fn main(){}",
        ),
        (
            "value-self-elision",
            "struct S<'a>(&'a u8);impl<'a>S<'a>{fn get(self)->&u8{self.0}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(semantic::analyze(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn invalid_unused_method_bodies_still_require_ownership_and_region_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "method-wrong-output-lifetime",
            "struct S(u8);impl S{fn get(&self,x:&u8)->&u8{x}}fn main(){}",
        ),
        (
            "method-moved-value",
            "struct S<T>(T);impl<T>S<T>{fn twice(self)->(T,T){(self.0,self.0)}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn owner_lifetime_bounds_and_anonymous_captures_reach_method_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "impl-outlives",
            "struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'b,'b>S<'a,'b>{fn first(&self)->&'b u8{self.0}}fn main(){}",
        ),
        (
            "impl-anonymous",
            "struct S<'a>(&'a u8);impl S<'_>{fn value(self)->u8{*self.0}fn borrow(&self)->&u8{self.0}}fn main(){}",
        ),
        (
            "impl-unused-lifetime",
            "struct S<'a>(&'a u8);impl<'unused,'a>S<'a>{fn value(self)->u8{*self.0}}fn main(){}",
        ),
        (
            "equivalent-receiver-lifetimes",
            "struct S<'a>(&'a u8);impl<'a,'b>S<'a>{fn get(self:&S<'b>)->u8 where 'a:'b,'b:'a{*self.0}}fn main(){}",
        ),
        (
            "nested-const-constraint",
            "struct S<T>(T);impl<const N:usize>S<[u8;N]>{fn count(&self)->usize{N}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, true)?;
        prove(source)?;
    }
    for (name, source) in [
        (
            "impl-missing-outlives",
            "struct S<'a,'b>(&'a u8,&'b u8);impl<'a,'b>S<'a,'b>{fn first(&self)->&'b u8{self.0}}fn main(){}",
        ),
        (
            "receiver-one-way-outlives",
            "struct S<'a>(&'a u8);impl<'a,'b>S<'a>{fn get(self:&S<'b>)->u8 where 'a:'b{*self.0}}fn main(){}",
        ),
        (
            "receiver-wrong-type-argument",
            "struct S<T>(T);impl S<u8>{fn get(self:S<u16>)->u16{self.0}}fn main(){}",
        ),
        (
            "receiver-wrong-const-argument",
            "struct S<const N:usize>([u8;N]);impl S<2>{fn get(self:S<3>){}}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn callable_scope_validation_rejects_foreign_reordered_and_duplicate_parameter_owners()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S<T>(T);impl<T>S<T>{fn get<U>(&self,x:U)->U{x}}fn main(){}";
    rustc::qualify("inherent-callable-scopes", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let function = method(&report, &facts, "get")?;
    let owner = facts
        .associated
        .get(
            facts
                .associated
                .owner(function.definition)
                .ok_or("missing owner")?,
        )
        .ok_or("missing owner")?;
    let callable = facts
        .lifetimes
        .callables
        .get(ItemId(function.definition.0))
        .ok_or("missing callable")?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(function.definition.0))
        .ok_or("missing parameters")?;
    let owners = [
        ItemId(owner.self_definition.0),
        ItemId(function.definition.0),
    ];
    let mut templates = rustgm::types::callable::Templates::default();
    assert!(
        templates
            .insert_scoped(
                &facts.types,
                &owners,
                callable.signature,
                0,
                &[],
                parameters
            )
            .is_ok()
    );
    for (owners, parameters) in [
        (vec![owners[1]], parameters.to_vec()),
        (vec![owners[0], owners[0], owners[1]], parameters.to_vec()),
        (owners.to_vec(), vec![parameters[1], parameters[0]]),
        (
            owners.to_vec(),
            vec![parameters[0], parameters[0], parameters[1]],
        ),
    ] {
        let mut templates = rustgm::types::callable::Templates::default();
        assert!(
            templates
                .insert_scoped(
                    &facts.types,
                    &owners,
                    callable.signature,
                    0,
                    &[],
                    &parameters
                )
                .is_err()
        );
    }
    Ok(())
}
