//! Opaque expression checking and concrete callable instantiation have separate gates.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const VALUES: &[(&str, &str, bool)] = &[
    (
        "forward-scalar",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A>(x:<T as A>::Out)-><T as A>::Out{x}fn main(){check(id::<u8>(73)==73);}",
        true,
    ),
    (
        "forward-reference",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}fn id<T:A>(x:<T as A>::Out)-><T as A>::Out{x}fn main(){let x=73;check(*id::<&u16>(&x)==73);}",
        true,
    ),
    (
        "forward-tuple",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn pair<T:A>(x:<T as A>::Out,y:u8)->(<T as A>::Out,u8){(x,y)}fn main(){let x=pair::<u8>(73,2);check(x.0==73&&x.1==2);}",
        true,
    ),
    (
        "forward-branch",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn pick<T:A>(flag:bool,x:<T as A>::Out,y:<T as A>::Out)-><T as A>::Out{if flag{x}else{y}}fn main(){check(pick::<u8>(false,1,73)==73);}",
        true,
    ),
    (
        "forward-array",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn one<T:A>(x:<T as A>::Out)->[<T as A>::Out;1]{[x]}fn main(){check(one::<u8>(73)[0]==73);}",
        true,
    ),
    (
        "different-subjects",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn wrong<T:A,U:A>(x:<T as A>::Out)-><U as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "different-trait-inputs",
        "trait A<T>{type Out;}fn wrong<T:A<u8>+A<u16>>(x:<T as A<u8>>::Out)-><T as A<u16>>::Out{x}fn main(){}",
        false,
    ),
    (
        "equality-output",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A<Out=u16>>(x:<T as A>::Out)->u16{x}fn main(){check(id::<u8>(73)==73);}",
        true,
    ),
    (
        "copy-result",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}fn twice<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=twice::<u8>(73);check(x.0==73&&x.1==73);}",
        true,
    ),
    (
        "borrow-forward",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn id<'a,T:A>(x:&'a<T as A>::Out)->&'a<T as A>::Out{x}fn main(){let x=73;check(*id::<u8>(&x)==73);}",
        true,
    ),
    (
        "borrow-mutable",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn id<'a,T:A>(x:&'a mut<T as A>::Out)->&'a mut<T as A>::Out{x}fn main(){let mut x=1;*id::<u8>(&mut x)=73;check(x==73);}",
        true,
    ),
    (
        "discard-opaque",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn discard<T:A>(_:<T as A>::Out){}fn main(){discard::<u8>(73);}",
        true,
    ),
    (
        "discard-divergent",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn discard<T:A>(_:<T as A>::Out){loop{}}fn main(){}",
        true,
    ),
    (
        "copy-result-borrow-out",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}fn value<T:A>(x:&<T as A>::Out)-><T as A>::Out{*x}fn main(){check(value::<u8>(&73)==73);}",
        true,
    ),
    (
        "function-pointer",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A>(f:fn(<T as A>::Out)-><T as A>::Out,x:<T as A>::Out)-><T as A>::Out{f(x)}fn inc(x:u16)->u16{x+1}fn main(){check(id::<u8>(inc,72)==73);}",
        true,
    ),
    (
        "higher-ranked-reference",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl A for u8{type Out=u16;}fn apply<T:A>(f:for<'a>fn(&'a<T as A>::Out)->&'a<T as A>::Out,x:&<T as A>::Out)->bool{let _=f(x);true}fn id(x:&u16)->&u16{x}fn main(){check(apply::<u8>(id,&73));}",
        true,
    ),
    (
        "opaque-output-static",
        "trait A{type Out;}fn require_static<X:'static>(_:X){}fn f<T:A>(x:<T as A>::Out)where<T as A>::Out:'static{require_static(x);}impl A for u8{type Out=u16;}fn main(){f::<u8>(73);}",
        true,
    ),
    (
        "self-static-insufficient",
        "trait A{type Out;}fn require_static<X:'static>(_:X){}fn f<T:A+'static>(x:<T as A>::Out){require_static(x);}fn main(){}",
        true,
    ),
    (
        "opaque-drop-order",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}struct D<'a>(&'a mut u16);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}impl<'a>A for &'a mut u16{type Out=D<'a>;}fn discard<T:A>(x:<T as A>::Out){let _y=x;}fn main(){let mut n=0;discard::<&mut u16>(D(&mut n));check(n==1);}",
        true,
    ),
    (
        "forward-drop",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}struct D<'a>(&'a mut u16);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}impl<'a>A for &'a mut u16{type Out=D<'a>;}fn id<T:A>(x:<T as A>::Out)-><T as A>::Out{x}fn main(){let mut n=0;{let _x=id::<&mut u16>(D(&mut n));}check(n==1);}",
        true,
    ),
    (
        "all-inputs-static",
        "trait A<U>{type Out;}impl A<u16>for u8{type Out=u32;}fn require_static<X:'static>(_:X){}fn f<T:A<U>+'static,U:'static>(x:<T as A<U>>::Out){require_static(x);}fn main(){f::<u8,u16>(73);}",
        true,
    ),
    (
        "output-static-with-captured-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=u16;}fn require_static<X:'static>(_:X){}fn f<T:A>(_:T,x:<T as A>::Out)where<T as A>::Out:'static{require_static(x);}fn main(){let x=1;f(&x,73);}",
        true,
    ),
    (
        "opaque-captured-callable",
        "trait A{type Out;}fn f<'a,T:A>(p:for<'b>fn(&'b u8,&'a <T as A>::Out))->for<'c>fn(&'c u8,&'a <T as A>::Out){p}fn main(){}",
        true,
    ),
    (
        "opaque-parameter-equality",
        "trait A{type Out;}fn f<T:A<Out=<U as A>::Out>,U:A>(p:<T as A>::Out)-><U as A>::Out{p}fn main(){}",
        true,
    ),
    (
        "opaque-repeated-local-value",
        "trait A{type Out;}fn f<T:A>(p:<T as A>::Out)-><T as A>::Out{let q:<T as A>::Out=p;let r:<T as A>::Out=q;r}fn main(){}",
        true,
    ),
    (
        "written-trait-lifetime-value",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A<'a>{type Out;}fn f<'a,T:A<'a>>(x:<T as A<'a>>::Out)-><T as A<'a>>::Out{x}fn main(){}",
        true,
    ),
];

const CALLABLES: &[(&str, &str, bool)] = &[
    (
        "opaque-callable-identity",
        "trait A{type Out;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out)->&'a <T as A>::Out)->for<'b>fn(&'b <T as A>::Out)->&'b <T as A>::Out{p}fn main(){}",
        true,
    ),
    (
        "opaque-sibling-callables",
        "trait A{type Out;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out),q:for<'a>fn(&'a <T as A>::Out))->(for<'b>fn(&'b <T as A>::Out),for<'c>fn(&'c <T as A>::Out)){(q,p)}fn main(){}",
        true,
    ),
    (
        "opaque-nested-callable",
        "trait A{type Out;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out,for<'b>fn(&'b u8,&'a <T as A>::Out)))->for<'x>fn(&'x <T as A>::Out,for<'y>fn(&'y u8,&'x <T as A>::Out)){p}fn main(){}",
        true,
    ),
    (
        "opaque-callable-different-subjects",
        "trait A{type Out;}fn f<T:A,U:A>(p:for<'a>fn(&'a <T as A>::Out))->for<'b>fn(&'b <U as A>::Out){p}fn main(){}",
        false,
    ),
    (
        "opaque-callable-different-items",
        "trait A{type Out;type Other;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out))->for<'b>fn(&'b <T as A>::Other){p}fn main(){}",
        false,
    ),
    (
        "opaque-nested-different-subjects",
        "trait A{type Out;}fn f<T:A,U:A>(p:for<'a>fn(&'a <T as A>::Out,for<'b>fn(&'b u8,&'a <T as A>::Out)))->for<'x>fn(&'x <T as A>::Out,for<'y>fn(&'y u8,&'x <U as A>::Out)){p}fn main(){}",
        false,
    ),
    (
        "opaque-local-shared-normal-form",
        "trait A{type Out;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<'x,'y:'x,T:A>(p:fn(<T as B<'x,'y>>::Out))->fn(<T as A>::Out){let q:fn(<T as B<'x,'y>>::Out)=p;q}fn main(){}",
        true,
    ),
    (
        "opaque-sibling-valid-formation",
        "trait A{type Out;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,<T as B<'a,'static>>::Out))->for<'b>fn(&'b u8,<T as A>::Out){p}fn main(){}",
        true,
    ),
    (
        "opaque-sibling-invalid-formation",
        "trait A{type Out;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,<T as A>::Out))->(for<'a>fn(&'a u8,<T as B<'a,'static>>::Out),for<'a>fn(&'a u8,<T as B<'static,'a>>::Out)){(p,p)}fn main(){}",
        false,
    ),
    (
        "opaque-nested-valid-formation",
        "trait A{type Out;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,for<'b>fn(&'b u8,<T as A>::Out)))->for<'x>fn(&'x u8,for<'y>fn(&'y u8,<T as B<'y,'static>>::Out)){p}fn main(){}",
        true,
    ),
    (
        "opaque-nested-invalid-formation",
        "trait A{type Out;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,for<'b>fn(&'b u8,<T as A>::Out)))->for<'x>fn(&'x u8,for<'y>fn(&'y u8,<T as B<'x,'y>>::Out)){p}fn main(){}",
        false,
    ),
    (
        "opaque-result-equality-does-not-equate-inputs",
        "trait A{type Out;}fn f<T:A<Out=<U as A>::Out>,U:A>(p:T)->U{p}fn main(){}",
        false,
    ),
    (
        "opaque-distinct-trait-const",
        "trait A<const N:usize>{type Out;}fn f<T:A<1>+A<2>>(p:<T as A<1>>::Out)-><T as A<2>>::Out{p}fn main(){}",
        false,
    ),
    (
        "static-selection",
        "trait A{type Out;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(p:fn(<T as A>::Out))->fn(<T as A>::Out){p}fn take(_:u16){}fn main(){let p=f::<&'static u8>(take);p(73);}",
        true,
    ),
    (
        "invalid-captured-selection",
        "trait A{type Out;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(p:fn(<T as A>::Out))->fn(<T as A>::Out){p}fn take(_:u16){}fn g<'a>(_:&'a u8){let p=f::<&'a u8>(take);p(73);}fn main(){let x=1;g(&x);}",
        false,
    ),
];

#[test]
fn opaque_expression_types_preserve_result_identity() -> Result<(), Box<dyn std::error::Error>> {
    // Ownership and lifetime acceptance are intentionally outside this phase.
    // These cases must also pass the whole compiler before becoming course fixtures.
    for &(name, source, accepted) in VALUES {
        rustc::qualify(&format!("opaque-values-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn opaque_callable_results_retain_each_scopes_formation_debt()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CALLABLES {
        rustc::qualify(&format!("opaque-callables-{name}"), source, accepted)?;
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
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn opaque_callable_instances_execute_with_concrete_types() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "opaque-callables",
        &[
            (
                "identity",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out)->&'a <T as A>::Out)->for<'b>fn(&'b <T as A>::Out)->&'b <T as A>::Out{p}fn id(x:&u16)->&u16{x}fn main(){let p=f::<u8>(id);let x=73u16;check(*p(&x)==73);}",
                false,
            ),
            (
                "siblings",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out)->u16,q:for<'a>fn(&'a <T as A>::Out)->u16)->(for<'b>fn(&'b <T as A>::Out)->u16,for<'c>fn(&'c <T as A>::Out)->u16){(q,p)}fn id(x:&u16)->u16{*x}fn add(x:&u16)->u16{*x+1}fn main(){let p=f::<u8>(id,add);check((p.0)(&72)==73);check((p.1)(&73)==73);}",
                false,
            ),
            (
                "nested",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A>(p:for<'a>fn(&'a <T as A>::Out,for<'b>fn(&'b u8,&'a <T as A>::Out))->u16)->for<'x>fn(&'x <T as A>::Out,for<'y>fn(&'y u8,&'x <T as A>::Out))->u16{p}fn run<'a>(x:&'a u16,g:for<'b>fn(&'b u8,&'a u16))->u16{g(&1,x);*x}fn verify(_: &u8,x:&u16){check(*x==73)}fn main(){let p=f::<u8>(run);check(p(&73,verify)==73);}",
                false,
            ),
            (
                "erased-valid-formation",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,<T as B<'a,'static>>::Out))->for<'b>fn(&'b u8,<T as A>::Out){p}fn verify(_: &u8,x:u16){check(x==73)}fn main(){let p=f::<u8>(verify);p(&1,73);}",
                false,
            ),
            (
                "local-known-formation",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<'x,'y:'x,T:A>(p:fn(<T as B<'x,'y>>::Out))->fn(<T as A>::Out){let q:fn(<T as B<'x,'y>>::Out)=p;q}fn verify(x:u16){check(x==73)}fn main(){let p=f::<'static,'static,u8>(verify);p(73);}",
                false,
            ),
            (
                "nested-erased-formation",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait B<'a,'b>{type Out;}impl<'a,'b:'a,T:A>B<'a,'b>for T{type Out=<T as A>::Out;}fn f<T:A>(p:for<'a>fn(&'a u8,for<'b>fn(&'b u8,<T as A>::Out)))->for<'x>fn(&'x u8,for<'y>fn(&'y u8,<T as B<'y,'static>>::Out)){p}fn run(_: &u8,g:for<'b>fn(&'b u8,u16)){g(&1,73)}fn verify(_: &u8,x:u16){check(x==73)}fn main(){let p=f::<u8>(run);p(&1,verify);}",
                false,
            ),
        ],
    )
}
