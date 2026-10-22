use crate::qualify as rustc;
use crate::semantic;

const ACCEPTED: &[(&str, &str)] = &[
    (
        "inferred-array-index",
        "fn f<const N:usize>(x:[u8;N])->[u8;N]{x}fn run()->u8{f([1,2,3])[1]}",
    ),
    (
        "inferred-array-unsize",
        "fn f<const N:usize>(x:&[u8;N])->&[u8;N]{x}fn run(x:&[u8;3])->&[u8]{f(x)}",
    ),
    (
        "returned-item",
        "fn id<T>(x:T)->T{x}fn add(x:u8)->u8{x+1}fn run()->u8{let f=id(add);f(3)}",
    ),
    (
        "distinct-const-items",
        "fn f<const N:usize>(){}fn run(b:bool)->fn(){if b{f::<3>}else{f::<4>}}",
    ),
    (
        "nested-parameter-types",
        "fn f<T>(x:fn(T)->T,y:T)->T{x(y)}fn id(x:u8)->u8{x}fn run()->u8{f(id,3)}",
    ),
    (
        "fresh-type-positions",
        "fn pair<T,U>(x:T,y:U)->(T,U){(x,y)}fn run<'a>(x:&'a u8)->(&'a u8,&'a u8){let chosen=pair::<&'a u8,&'a u8>;chosen(x,x)}",
    ),
    (
        "argument-inference",
        "fn id<T>(x:T)->T{x}fn run()->u8{id(3u8)}",
    ),
    (
        "return-inference",
        "fn empty<T>()->[T;0]{[]}fn run()->[u16;0]{empty()}",
    ),
    (
        "explicit-type",
        "fn id<T>(x:T)->T{x}fn run()->u16{id::<u16>(3)}",
    ),
    (
        "placeholder-type",
        "fn id<T>(x:T)->T{x}fn run()->u8{id::<_>(3)}",
    ),
    (
        "partial-type",
        "fn id<T>(x:T)->T{x}fn run()->(u8,bool){id::<(_,_)>((3,true))}",
    ),
    (
        "expected-pointer",
        "fn id<T>(x:T)->T{x}fn run()->fn(u8)->u8{id}",
    ),
    (
        "later-call",
        "fn id<T>(x:T)->T{x}fn run()->u8{let f=id;f(3)}",
    ),
    (
        "independent-uses",
        "fn id<T>(x:T)->T{x}fn run()->(u8,u16){let f=id;let g=id;(f(3),g(4))}",
    ),
    (
        "erased-identity-coercion",
        "fn f<T>(){}fn run(b:bool)->fn(){if b{f::<u8>}else{f::<u16>}}",
    ),
    (
        "array-inference",
        "fn pass<T,const N:usize>(x:[T;N])->[T;N]{x}fn run()->[u8;3]{pass([1,2,3])}",
    ),
    (
        "placeholder-const",
        "fn pass<T,const N:usize>(x:[T;N])->[T;N]{x}fn run()->[u8;3]{pass::<u8,_>([1,2,3])}",
    ),
    (
        "const-type-order",
        "fn f<const N:usize,T>(x:[T;N])->[T;N]{x}fn run()->[u8;3]{f::<3,u8>([1,2,3])}",
    ),
    (
        "const-bool",
        "fn f<const B:bool>()->bool{B}fn run()->bool{f::<true>()}",
    ),
    (
        "const-expression",
        "fn f<const N:usize>()->usize{N}fn run()->usize{f::<{1+2}>()}",
    ),
    (
        "const-name",
        "const N:usize=3;fn f<const C:usize>()->usize{C}fn run()->usize{f::<N>()}",
    ),
    ("recursive-type", "fn f<T>(x:T)->T{f(x)}"),
    (
        "recursive-const",
        "fn f<const N:usize>(x:[u8;N])->[u8;N]{f(x)}",
    ),
    (
        "explicit-forward",
        "fn id<T>(x:T)->T{x}fn f<U>(x:U)->U{id::<U>(x)}",
    ),
    (
        "early-lifetime",
        "fn f<'a:'a,T>(x:&'a T)->&'a T{x}fn run(x:&'static u8)->&'static u8{f::<'static,u8>(x)}",
    ),
    (
        "late-lifetime",
        "fn f<'a,T>(x:&'a T)->&'a T{x}fn run(x:&u8)->&u8{f(x)}",
    ),
    (
        "borrowed-type",
        "fn id<T>(x:T)->T{x}fn run(x:&u8)->&u8{id(x)}",
    ),
    (
        "explicit-borrow",
        "fn id<T>(x:T)->T{x}fn run<'a>(x:&'a u8)->&'a u8{id::<&'a u8>(x)}",
    ),
    (
        "partial-borrow",
        "fn id<T>(x:T)->T{x}fn run<'a>(x:&'a u8)->&'a u8{id::<&'a _>(x)}",
    ),
    (
        "inferred-item-branch",
        "fn id<T>(x:T)->T{x}fn run(b:bool)->u8{let f=if b{id}else{id};f(1)}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "conflicting-lengths",
        "fn f<const N:usize>(x:[u8;N],y:[u8;N]){}fn run(){f([1],[1,2]);}",
    ),
    (
        "const-parameter-arithmetic",
        "fn f<const N:usize>(){}fn g<const M:usize>(){f::<{M+1}>();}",
    ),
    (
        "one-item-use",
        "fn id<T>(x:T)->T{x}fn run(){let f=id;f(3u8);f(4u16);}",
    ),
    ("unconstrained-erased-type", "fn f<T>(){}fn run(){f();}"),
    (
        "wrong-argument-kind",
        "fn pass<T,const N:usize>(x:[T;N])->[T;N]{x}fn run()->[u8;3]{pass::<3,u8>([1,2,3])}",
    ),
    (
        "explicit-late-lifetime",
        "fn f<'a,T>(x:&'a T)->&'a T{x}fn run(x:&'static u8)->&'static u8{f::<'static,u8>(x)}",
    ),
    (
        "infer-const-bool",
        "fn f<const B:bool>()->bool{B}fn run()->bool{f()}",
    ),
    (
        "missing-argument",
        "fn f<T,U>(x:T,y:U){}fn run(){f::<u8>(1,true);}",
    ),
    ("unsized-type", "fn f<T>(){}fn run(){f::<str>();}"),
    (
        "unsized-through-reference",
        "fn f<T>(_:&T){}fn run(x:&str){f(x);}",
    ),
    (
        "wrong-array-length",
        "fn f<const N:usize>(x:[u8;N])->[u8;N]{x}fn run()->[u8;4]{f::<3>([1,2,3])}",
    ),
    (
        "wrong-const-type",
        "fn f<const N:usize>(){}fn run(){f::<true>();}",
    ),
    (
        "erased-const-mismatch",
        "fn f<const N:usize>(){}fn run(){let mut g=f::<3>;g=f::<4>;}",
    ),
    (
        "rigid-bool",
        "fn f<T>(x:T){id::<bool>(x);}fn id<U>(x:U)->U{x}",
    ),
];
#[test]
fn calls_infer_and_retain_type_const_and_lifetime_arguments() -> Result<(), String> {
    let errors: Vec<_> = ACCEPTED
        .iter()
        .filter_map(|(name, source)| {
            semantic::analyze(source)
                .err()
                .map(|e| format!("{name}: {e}"))
        })
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}
#[test]
fn invalid_generic_uses_do_not_erase_unsolved_or_incompatible_arguments() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}
#[test]
fn generic_call_expectations_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in ACCEPTED {
        rustc::qualify(&format!("generic-call-accept-{name}"), source, true)?;
    }
    for &(name, source) in REJECTED {
        rustc::qualify(&format!("generic-call-reject-{name}"), source, false)?;
    }
    Ok(())
}

#[test]
fn referenced_generic_functions_publish_one_callable_template() -> Result<(), String> {
    let (report, facts) = semantic::analyze(
        "fn unused<T>(x:T)->T{x}fn id<T>(x:T)->T{x}\n\
         fn run()->(u8,u16){(id(3),id(4))}",
    )?;
    for function in &facts.functions {
        let definition = &report.resolution.definitions[function.definition.0 as usize];
        let name = report.package.names.get(definition.name);
        let item = rustgm::types::ItemId(function.definition.0);
        assert_eq!(facts.lifetimes.callables.get(item).is_some(), name == "id");
        if name == "id" {
            assert_eq!(
                facts
                    .lifetimes
                    .callables
                    .generic_parameters(item)
                    .map(<[_]>::len),
                Some(1)
            );
        }
    }
    Ok(())
}

#[test]
fn fresh_item_signatures_follow_each_type_argument_once() -> Result<(), String> {
    use rustgm::{
        resolve::model::DefKind,
        types::{Kind, arguments::Argument},
    };
    let source = "fn pair<T,U>(x:T,y:U)->(T,U){(x,y)}fn run<'a>(x:&'a u8)->(&'a u8,&'a u8){let chosen=pair::<&'a u8,&'a u8>;chosen(x,x)}";
    let (report, facts) = semantic::analyze(source)?;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Local && report.package.names.get(d.name) == "chosen")
        .ok_or("missing chosen item")?;
    let ty = facts.definitions[index].ok_or("missing local type")?;
    let Some(Kind::FunctionItem {
        arguments,
        signature,
        ..
    }) = facts.types.kind(ty)
    else {
        return Err("local is not a function item".into());
    };
    let args = facts
        .types
        .arguments(arguments)
        .ok_or("missing arguments")?
        .filter_map(|a| {
            if let Argument::Type(t) = a {
                Some(t)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(signature)
    else {
        return Err("missing signature".into());
    };
    assert_eq!(facts.types.list(parameters), Some(args.as_slice()));
    let Some(Kind::Tuple(results)) = facts.types.kind(result) else {
        return Err("missing tuple result".into());
    };
    assert_eq!(facts.types.list(results), Some(args.as_slice()));
    assert_ne!(
        args[0], args[1],
        "independent formal slots were coupled by a shared concrete input type"
    );
    Ok(())
}

#[path = "call_inference/returned.rs"]
mod returned;

#[path = "call_inference/hint_heads.rs"]
mod hint_heads;
