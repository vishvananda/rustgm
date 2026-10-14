#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::types::{Kind, arguments::Argument};
use std::fmt::Write as _;

const ACCEPTED: &[(&str, &str)] = &[
    ("identity", "fn identity<T>(x:T)->T{x}"),
    ("unused", "fn unused<T>(){}"),
    ("separate-owners", "fn a<T>(x:T)->T{x}fn b<T>(x:T)->T{x}"),
    ("swap", "fn swap<T,U>(x:T,y:U)->(U,T){(y,x)}"),
    ("branch", "fn choose<T>(a:T,b:T,c:bool)->T{if c{a}else{b}}"),
    ("mutable-local", "fn assign<T>(mut x:T,y:T)->T{x=y;x}"),
    ("annotation", "fn f<T>(x:T)->T{let y:T=x;y}"),
    (
        "alias",
        "type Pair<T>=(T,T);fn f<T>(x:T,y:T)->Pair<T>{(x,y)}",
    ),
    ("array", "fn f<T,const N:usize>(x:[T;N])->[T;N]{x}"),
    ("const-value", "fn f<const N:usize>()->usize{N}"),
    (
        "const-bool",
        "fn f<const B:bool>(a:u8,b:u8)->u8{if B{a}else{b}}",
    ),
    ("const-char", "fn f<const C:char>()->char{C}"),
    ("const-arithmetic", "fn f<const N:usize>()->usize{N+1}"),
    ("const-order", "fn f<const N:usize,T>(x:[T;N])->[T;N]{x}"),
    ("reference", "fn f<'a,T>(x:&'a T)->&'a T{x}"),
    ("mutable-reference", "fn f<T>(x:&mut T)->&mut T{x}"),
    ("late-order", "fn f<'a,'b,T>(x:&'b T,y:&'a T)->&'b T{x}"),
    ("early-lifetime", "fn f<'a:'b,'b,T>(x:&'a T)->&'b T{x}"),
    ("repeat-shared", "fn f<T>(x:&T)->[&T;2]{[x;2]}"),
    ("repeat-once", "fn f<T>(x:T)->[T;1]{[x;1]}"),
    ("fn-pointer", "fn f<T>(call:fn(T)->T,x:T)->T{call(x)}"),
];
const REJECTED: &[(&str, &str)] = &[
    ("rigid", "fn f<T>(x:T)->u8{x}"),
    ("distinct", "fn f<T,U>(x:T)->U{x}"),
    ("arithmetic", "fn f<T>(x:T)->T{x+1}"),
    ("numeric-default", "fn f<T>()->T{0}"),
    ("annotation", "fn f<T>(x:T){let y:u8=x;}"),
    ("repeat-unbounded", "fn f<T>(x:T)->[T;2]{[x;2]}"),
    ("const-result", "fn f<const N:usize>()->bool{N}"),
    ("const-condition", "fn f<const N:usize>(){if N{}}"),
    ("const-in-type", "fn f<const N:usize>(x:N){}"),
    ("const-type", "fn f<const N:f32>(){}"),
    ("dependent-const-type", "fn f<T,const N:T>(){}"),
    ("array-length-type", "fn f<const N:u8>(x:[u8;N]){}"),
    ("type-default", "fn f<T=u8>(){}"),
    ("const-default", "fn f<const N:usize=1>(){}"),
    ("duplicate", "fn f<T,const T:usize>(){}"),
    ("lifetime-order", "fn f<T,'a>(x:&'a T){}"),
    ("unused-invalid-body", "fn broken<T>(x:T)->u8{x}fn ok(){}"),
    (
        "drop-const-parameter",
        "struct S;impl Drop for S{fn drop<const N:usize>(&mut self){}}",
    ),
];

#[test]
fn source_function_templates_are_checked_with_rigid_parameters() -> Result<(), String> {
    for &(name, source) in ACCEPTED {
        semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

#[test]
fn unused_generic_bodies_and_declarations_are_checked() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}

#[test]
fn function_template_outcomes_are_independently_qualified() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, source) in ACCEPTED {
        rustc::qualify(&format!("generic-function-accept-{name}"), source, true)?;
    }
    for &(name, source) in REJECTED {
        rustc::qualify(&format!("generic-function-reject-{name}"), source, false)?;
    }
    Ok(())
}

#[test]
fn lifetime_binding_does_not_renumber_rigid_parameter_declarations() -> Result<(), String> {
    let (_, facts) =
        semantic::analyze("fn f<'a,'b,T,const N:usize>(x:&'b T,y:&'a [T;N])->&'b T{x}")?;
    let function = facts.functions[0];
    let parameters = &facts.generics.parameters[&function.definition];
    let Argument::Type(ty) = parameters[0].argument else {
        return Err("missing type parameter".into());
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("type parameter is not rigid".into());
    };
    let declaration = facts.types.parameter(id).ok_or("missing declaration")?;
    assert_eq!(declaration.index, 2);
    assert_eq!(declaration.owner.0, function.definition.0);
    let environment = &facts.generics.environments[&function.definition];
    assert!(!environment.abilities(id).ok_or("missing assumptions")?.copy);
    let lifetimes = &facts.lifetimes.functions[&function.definition];
    assert_eq!(lifetimes.late, [0, 1]);
    assert!(lifetimes.early.is_empty());
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(function.signature)
    else {
        return Err("missing signature".into());
    };
    let inputs = facts.types.list(parameters).ok_or("missing inputs")?;
    for (ty, index) in [(inputs[0], 0), (inputs[1], 1), (result, 0)] {
        let Some(Kind::Reference { region, .. }) = facts.types.kind(ty) else {
            return Err("missing reference parameter".into());
        };
        assert_eq!(region.bound_parts(), Some((0, index)));
    }
    Ok(())
}

#[test]
fn unfinished_instantiation_and_body_proofs_remain_explicit_boundaries() -> Result<(), String> {
    for source in ["fn f<T>(){}fn g(){f::<u8>();}", "fn f<T>(){}fn g(){f();}"] {
        let Err(message) = semantic::analyze(source) else {
            return Err("accepted an uninstantiated generic function".into());
        };
        assert!(message.contains("instance inference"), "{message}");
    }
    // Type inspection is not permission to erase a generic body or its drop work.
    let (report, facts) = semantic::analyze("fn f<T>(x:T)->T{x}")?;
    let Err(error) = rustgm::body::build(&report, &facts) else {
        return Err("constructed an unproved generic body".into());
    };
    assert!(
        error
            .to_string()
            .contains("parameterized ownership and destruction")
    );
    Ok(())
}

#[test]
fn executable_entry_rejects_unused_generic_parameters() -> Result<(), String> {
    for source in ["fn main<T>(){}", "fn main<const N:usize>(){}"] {
        let (mut report, facts) = semantic::analyze(source)?;
        let Err(error) = rustgm::sema::entry::select(&mut report, &facts) else {
            return Err("selected a generic main function".into());
        };
        assert!(
            error
                .to_string()
                .contains("main cannot have type or const parameters")
        );
    }
    Ok(())
}

#[test]
fn independent_symbolic_functions_reuse_syntax_and_preserve_owners() -> Result<(), String> {
    let mut source = String::new();
    for index in 0..512 {
        write!(source, "fn f{index}<T>(x:&T)->[&T;2]{{[x;2]}}").map_err(|e| e.to_string())?;
    }
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.functions.len(), 512);
    assert_eq!(facts.generics.environments.len(), 512);
    let signatures: std::collections::HashSet<_> =
        facts.functions.iter().map(|f| f.signature).collect();
    assert_eq!(signatures.len(), 512);
    Ok(())
}
