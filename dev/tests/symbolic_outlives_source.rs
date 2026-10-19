#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "inferred-early-local",
        "fn need<'a,T:'a>(){}fn f<T>(){need::<T>();}",
        true,
    ),
    (
        "erased-static-item",
        "fn need<T:'static>(){}fn f<'a>(){let item=need::<&'a u8>;}",
        false,
    ),
    (
        "const-array-static",
        "fn need<T:'static>(){}fn f<const N:usize>(){need::<[u8;N]>();}",
        true,
    ),
    (
        "const-array-bound",
        "fn f<'a,const N:usize>()where [&'a u8;N]:'static{}fn g<'b:'static,const M:usize>(){f::<'b,M>();}",
        true,
    ),
    (
        "const-array-bound-missing",
        "fn f<'a,const N:usize>()where [&'a u8;N]:'static{}fn g<'b,const M:usize>(){f::<'b,M>();}",
        false,
    ),
    (
        "relaxed-sized-outlives",
        "fn f<'a,T:?Sized+'a>(){}fn g(){f::<'static,str>();}",
        true,
    ),
    (
        "copy-and-outlives",
        "fn need<'a,T:Copy+'a>(x:T)->T{x}fn f<'a,U:Copy+'a>(x:U)->U{need::<'a,U>(x)}",
        true,
    ),
    (
        "alias-outlives-unenforced",
        "type A<T:'static>=T;fn f<'a>(x:A<&'a u8>)->&'a u8{x}",
        true,
    ),
    (
        "alias-subject-wf",
        "type A<'a,T>=&'a T;fn need<'a,T:'a>(){}fn f<'a,T>()where A<'a,T>:Copy{need::<'a,T>();}",
        false,
    ),
    (
        "higher-fn-outlives-assumption",
        "fn need<T:'static>(){}fn f<T>()where (for<'a>fn(&'a T)):'static{need::<T>();}",
        true,
    ),
    (
        "higher-fn-not-static-assumption",
        "fn need<T:'static>(){}fn f<T>(x:for<'a>fn(&'a T)){need::<T>();}",
        false,
    ),
    (
        "reference-components-same",
        "fn f<'a,T>()where &'a T:'a{}",
        true,
    ),
    (
        "reference-components-static",
        "fn f<'a,T>()where &'a T:'static{}",
        true,
    ),
    (
        "reference-components-fixed",
        "fn need<T:'static>(){} fn f<'a,T>()where &'a T:'static{need::<T>();}",
        true,
    ),
    (
        "reference-components-nested",
        "fn f<'a,'b>()where &'a &'b u8:'static{}",
        true,
    ),
    (
        "reference-components-nested-weaker",
        "fn f<'a,'b,'c>()where &'a &'b u8:'c{}",
        false,
    ),
    (
        "reference-components-nested-copy",
        "fn f<'a,'b>()where &'a &'b u8:Copy{}",
        false,
    ),
    (
        "reference-components-same-copy",
        "fn f<'a,T>()where &'a T:Copy{}",
        false,
    ),
    (
        "reference-components-static-return",
        "fn f<'a,'b>(x:&'b u8)->&'a u8 where &'a &'b u8:'static{x}",
        true,
    ),
    ("plain-identity", "fn f<T>(x:T)->T{x}", true),
    ("inline-lifetime", "fn f<'a,T:'a>(x:&'a T)->&'a T{x}", true),
    (
        "where-lifetime",
        "fn f<'a,T>(x:&'a T)->&'a T where T:'a{x}",
        true,
    ),
    (
        "implied-reference",
        "fn need<'a,T:'a>(){} fn f<'a,T>(x:&'a T){need::<'a,T>();}",
        true,
    ),
    (
        "missing-reference",
        "fn need<'a,T:'a>(){} fn f<'a,T>(){need::<'a,T>();}",
        false,
    ),
    (
        "bound-reference-explicit",
        "fn f<'a,T:'a>()where &'a T:Copy{}",
        true,
    ),
    (
        "bound-reference-implied",
        "fn f<'a,T>(x:&'a T)where &'a T:Copy{}",
        true,
    ),
    (
        "bound-reference-missing",
        "fn f<'a,T>()where &'a T:Copy{}",
        false,
    ),
    (
        "where-does-not-imply",
        "fn f<'a,T>()where &'a T:Copy{}",
        false,
    ),
    (
        "bound-mut-explicit",
        "fn f<'a,T:'a>()where &'a mut T:Copy{}",
        true,
    ),
    (
        "bound-mut-forward",
        "fn f<'a,T:'a>()where &'a mut T:Copy{} fn g<'b,U:'b>()where &'b mut U:Copy{f::<'b,U>();}",
        true,
    ),
    (
        "static-parameter",
        "fn need<T:'static>(x:T){}fn f<U:'static>(x:U){need(x)}",
        true,
    ),
    (
        "static-parameter-missing",
        "fn need<T:'static>(x:T){}fn f<U>(x:U){need(x)}",
        false,
    ),
    (
        "static-integer",
        "fn need<T:'static>(x:T){}fn f(x:u8){need(x)}",
        true,
    ),
    (
        "static-reference",
        "fn need<T:'static>(x:T){}fn f(x:&'static u8){need(x)}",
        true,
    ),
    (
        "static-short-reference",
        "fn need<T:'static>(x:T){}fn f<'a>(x:&'a u8){need(x)}",
        false,
    ),
    (
        "static-fn-argument",
        "fn need<T:'static>(x:T){}fn f<'a>(x:fn(&'a u8)){need(x)}",
        true,
    ),
    (
        "static-fn-result",
        "fn need<T:'static>(x:T){}fn f<'a>(x:fn()->&'a u8){need(x)}",
        false,
    ),
    (
        "static-higher-fn",
        "fn need<T:'static>(x:T){}fn f(x:for<'a>fn(&'a u8)){need(x)}",
        true,
    ),
    (
        "static-fn-parameter",
        "fn need<T:'static>(x:T){}fn f<U>(x:fn(U)){need(x)}",
        false,
    ),
    (
        "static-raw-pointer",
        "fn need<T:'static>(x:T){}fn f<'a>(x:*const &'a u8){need(x)}",
        false,
    ),
    (
        "static-zero-array",
        "fn need<T:'static>(x:T){}fn f<'a>(x:[&'a u8;0]){need(x)}",
        false,
    ),
    (
        "outlives-tuple-member",
        "fn need<'a,T:'a>(){}fn f<'a,T,U>()where(T,U):'a{need::<'a,T>();}",
        true,
    ),
    (
        "outlives-array-member",
        "fn need<'a,T:'a>(){}fn f<'a,T>()where[T;0]:'a{need::<'a,T>();}",
        true,
    ),
    (
        "outlives-ref-member",
        "fn need<'a,T:'a>(){}fn f<'a,'b,T>()where &'b T:'a{need::<'a,T>();}",
        false,
    ),
    (
        "outlives-ref-valid",
        "fn need<'a,T:'a>(){}fn f<'a,'b,T:'b>()where &'b T:'a{need::<'a,T>();}",
        true,
    ),
    (
        "outlives-fn-member",
        "fn need<'a,T:'a>(){}fn f<'a,T>()where fn(T):'a{need::<'a,T>();}",
        true,
    ),
    (
        "outlives-fn-ref-implied",
        "fn need<'a,T:'a>(){}fn f<'a,T>(x:fn(&'a T)){need::<'a,T>();}",
        true,
    ),
    (
        "outlives-nested-implied",
        "fn need<'a,T:'a>(){}fn f<'a,T>(x:(&'a T,)){need::<'a,T>();}",
        true,
    ),
    (
        "outlives-result-implied",
        "fn need<'a,T:'a>(){}fn f<'a,T>()->&'a T{need::<'a,T>();loop{}}",
        true,
    ),
    (
        "outlives-transitive",
        "fn need<'a,T:'a>(){}fn f<'a,'b:'a,T:'b>(){need::<'a,T>();}",
        true,
    ),
    (
        "outlives-wrong-direction",
        "fn need<'a,T:'a>(){}fn f<'a:'b,'b,T:'b>(){need::<'a,T>();}",
        false,
    ),
    (
        "outlives-free-alias",
        "type R<'a,T>=&'a T;fn need<'a,T:'a>(){}fn f<'a,T>(x:R<'a,T>){need::<'a,T>();}",
        true,
    ),
    ("placeholder-bound", "fn f<T:'_>(){}", false),
    (
        "bound-lifetime-early",
        "fn f<'a,T:'a>(x:&'a T)->&'a T{x}fn g<'b,U:'b>(x:&'b U)->&'b U{f::<'b,U>(x)}",
        true,
    ),
    (
        "static-fn-argument-explicit",
        "fn need<T:'static>(x:T){}fn f<'a>(x:fn(&'a u8)){need::<fn(&'a u8)>(x)}",
        false,
    ),
    (
        "static-fn-result-explicit",
        "fn need<T:'static>(x:T){}fn f<'a>(x:fn()->&'a u8){need::<fn()->&'a u8>(x)}",
        false,
    ),
    (
        "static-fn-param-bound",
        "fn need<T:'static>(x:T){}fn f<U:'static>(x:fn(U)){need(x)}",
        true,
    ),
    (
        "static-fn-arg-where",
        "fn f<'a>(x:fn(&'a u8))where fn(&'a u8):'static{}",
        true,
    ),
    (
        "static-raw-fn",
        "fn need<T:'static>(x:T){}fn f<'a>(x:*const fn(&'a u8)){need(x)}",
        true,
    ),
    (
        "static-zero-fn-array",
        "fn need<T:'static>(x:T){}fn f<'a>(x:[fn(&'a u8);0]){need(x)}",
        true,
    ),
    (
        "fn-type-implied-local",
        "fn need<'a,T:'a>(){}fn f<'a,T>(){let x:fn(&'a T);need::<'a,T>();}",
        false,
    ),
    (
        "fn-bound-implied-local",
        "fn need<'a,T:'a>(){}fn f<'a,T>()where fn(&'a T):Copy{need::<'a,T>();}",
        false,
    ),
    (
        "copy-impossible-implies-static",
        "fn need<T:'static>(){}fn f<'a>()where &'a mut u8:Copy{need::<&'a u8>();}",
        false,
    ),
    (
        "copy-slice-implies-element",
        "fn need<T:Copy>(){}fn f<T>()where[T]:Copy{need::<T>();}",
        false,
    ),
    (
        "outlives-bound-is-not-wf-proof",
        "fn f<'a,'b,T>()where &'a T:'b{}",
        false,
    ),
    (
        "outlives-inherent-reference-member",
        "fn need<'a,T:'a>(){}fn f<'a,'b,T>(x:&'b T)where &'b T:'a{need::<'a,T>();}",
        true,
    ),
    (
        "outlives-reverse-assumption",
        "fn need<'a,T:'a>(){}fn f<'a,'b,T>(x:&'b T)where &'a T:'b{need::<'a,T>();}",
        false,
    ),
];
#[test]
fn symbolic_outlives_are_proved_against_declared_and_signature_assumptions() -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        let result = semantic::analyze(source).and_then(|(_, facts)| {
            let contracts = rustgm::body::regions::collect(&facts).map_err(str::to_owned)?;
            rustgm::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != *accepted {
            failures.push(format!(
                "{name}: wanted {accepted}; {}",
                result.err().unwrap_or_default()
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
#[test]
fn source_outlives_expectations_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("type-outlives-{name}"), source, *accepted)?;
    }
    Ok(())
}

#[test]
fn higher_ranked_where_predicates_supply_declared_outlives_assumptions()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn need<T:'static>(){}fn f<T>()where for<'a>fn(&'a T):'static{need::<T>();}";
    rustc::qualify("outlives-higher-where-assumption", source, true)?;
    semantic::analyze(source)?;
    Ok(())
}
