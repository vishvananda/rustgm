//! Concrete field storage keeps variance and caller-owned formation obligations.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

#[test]
fn completed_body_demands_do_not_expand_discarded_constructor_templates()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [4, 16, 64, 96] {
        let mut source =
            String::from("trait A{type Out;}impl A for u8{type Out=u16;}struct S0<T:A>{v:T::Out}");
        for level in 1..=depth {
            write!(source, "struct S{level}<T:A>{{v:S{}<T>}}", level - 1)?;
        }
        source.push_str("fn main(){let w=");
        for level in (0..=depth).rev() {
            write!(source, "S{level}::<u8>{{v:")?;
        }
        source.push('7');
        source.extend(std::iter::repeat_n('}', depth + 1));
        source.push_str(";let _:u16=w");
        for _ in 0..=depth {
            source.push_str(".v");
        }
        source.push_str(";}");
        super::rustc::qualify(&format!("stored-field-depth-{depth}"), &source, true)?;
        let report = semantic::resolve(&source)?;
        let facts = rustgm::sema::declarations::analyze(&report)?;
        let facts = rustgm::sema::bodies::analyze(&report, facts)?;
        let steps = facts.aggregates.instantiation_steps();
        assert!(
            steps <= 128 * depth + 512,
            "depth {depth}: {steps} field steps"
        );
    }
    Ok(())
}

const CASES: &[(&str, &str, bool)] = &[
    (
        "mixed-variance-still-infers-local-lifetime",
        "trait A{type Out;}impl A for u8{type Out=u8;}struct W<'b,T:A>(T::Out,&'b u8);trait Q{const C:W<'static,u8>;}impl Q for u8{const C:W<'static,u8>=W(3,&9);}fn main(){let x=7;let mut w=<u8 as Q>::C;w=W::<u8>(4,&x);check(w.0==4&&*w.1==7);}",
        true,
    ),
    (
        "invariant-erased-input-cannot-change-on-assignment",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=u8;}struct W<T:A>(T::Out);fn make<'a>(_:&'a u8)->W<&'a u8>{W(1)}const C:W<&'static u8>=W(3);fn bad<'a>(x:&'a u8)->W<&'static u8>{let mut w=C;w=make(x);w}fn main(){}",
        false,
    ),
    (
        "generic-lifetime-omitted",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}fn make<'a,T:A<'a>>(x:T::Out)->W<'a,T>{W{v:x}}fn main(){let mut n=0u8;{let _w=make::<u8>(D(&mut n));}check(n==1);}",
        true,
    ),
    (
        "generic-lifetime-explicit",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}fn make<'a,T:A<'a>>(x:T::Out)->W<'a,T>{W::<'a,T>{v:x}}fn main(){let mut n=0u8;{let _w=make::<u8>(D(&mut n));}check(n==1);}",
        true,
    ),
    (
        "generic-lifetime-annotated-local",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}fn make<'a,T:A<'a>>(x:T::Out)->W<'a,T>{let w:W<'a,T>=W{v:x};w}fn main(){let mut n=0u8;{let _w=make::<u8>(D(&mut n));}check(n==1);}",
        true,
    ),
    (
        "generic-lifetime-nested-context",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}struct Outer<'a,T:A<'a>>{w:W<'a,T>}fn make<'a,T:A<'a>>(x:T::Out)->Outer<'a,T>{Outer{w:W{v:x}}}fn main(){let mut n=0u8;{let _w=make::<u8>(D(&mut n));}check(n==1);}",
        true,
    ),
    (
        "generic-lifetime-incompatible-owner",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}struct Other<'a,T:A<'a>>{v:T::Out}fn make<'a,T:A<'a>>(x:T::Out)->Other<'a,T>{W::<'a,T>{v:x}}fn main(){let mut n=0u8;{let _w=make::<u8>(D(&mut n));}check(n==1);}",
        false,
    ),
    (
        "record",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn main(){let w=W::<u8>{v:7};check(w.v==7);}",
        true,
    ),
    (
        "tuple",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>(T::Out);fn main(){let w=W::<u8>(7);check(w.0==7);}",
        true,
    ),
    (
        "enum",
        "trait A{type Out;}impl A for u8{type Out=u16;}enum W<T:A>{Empty,Value(T::Out)}fn main(){let w=W::<u8>::Value(7);check(match w{W::Value(x)=>x==7,W::Empty=>false});}",
        true,
    ),
    (
        "nested",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}struct Outer<T:A>{w:W<T>}fn main(){let w=Outer::<u8>{w:W{v:7}};check(w.w.v==7);}",
        true,
    ),
    (
        "alias",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}type Alias<T>=W<T>;fn main(){let w=Alias::<u8>{v:7};check(w.v==7);}",
        true,
    ),
    (
        "generic-body",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn id<T:A>(w:W<T>)->W<T>{w}fn main(){check(id(W::<u8>{v:7}).v==7);}",
        true,
    ),
    (
        "different-instances",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}impl A for u16{type Out=(u8,u32);}fn main(){let a=W::<u8>{v:7};let b=W::<u16>{v:(9,11)};check(a.v==7);check(b.v.0==9);check(b.v.1==11);}",
        true,
    ),
    (
        "borrowed-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}struct W<'a,T:A<'a>>{v:T::Out}fn main(){let x=7u8;let w=W::<u8>{v:&x};check(*w.v==7);}",
        true,
    ),
    (
        "conditional-lifetimes",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,T:A<'a>>{v:T::Out}fn make<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,&'b u8>{v:7};w.v}fn main(){let x=0;check(make(&x,&x)==7);}",
        true,
    ),
    (
        "function-pointer",
        "trait A{type Out;}impl A for u8{type Out=fn(u8)->u8;}struct W<T:A>{v:T::Out}fn id(x:u8)->u8{x}fn main(){let w=W::<u8>{v:id};check((w.v)(7)==7);}",
        true,
    ),
    (
        "array-result",
        "trait A{type Out;}impl A for u8{type Out=[u16;2];}struct W<T:A>{v:T::Out}fn main(){let w=W::<u8>{v:[7,9]};check(w.v[0]==7);check(w.v[1]==9);}",
        true,
    ),
    (
        "array-field",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:[T::Out;2]}fn main(){let w=W::<u8>{v:[7,9]};check(w.v[0]==7);check(w.v[1]==9);}",
        true,
    ),
    (
        "drop-field",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:T::Out}fn main(){let mut n=0u8;{let _w=W::<u8>{v:D(&mut n)};}check(n==1);}",
        true,
    ),
    (
        "drop-zero-array",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:[T::Out;0]}fn main(){let _w=W::<u8>{v:[]};check(true);}",
        true,
    ),
    (
        "copy-result",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}#[derive(Clone,Copy)]struct W<T:A>{v:T::Out}fn main(){let w=W::<u8>{v:7};let a=w;let b=w;check(a.v==7&&b.v==7);}",
        true,
    ),
    (
        "transparent",
        "trait A{type Out;}impl A for u8{type Out=u16;}#[repr(transparent)]struct W<T:A>{v:T::Out}fn main(){let w=W::<u8>{v:7};check(w.v==7);}",
        true,
    ),
    (
        "transparent-side-field",
        "trait A{type Out;}impl A for u8{type Out=u16;}#[repr(transparent)]struct W<T:A>{v:T::Out,z:()}fn main(){let w=W::<u8>{v:7,z:()};check(w.v==7);}",
        true,
    ),
    (
        "wrong-value",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn main(){let _w=W::<u8>{v:7u32};}",
        false,
    ),
    (
        "missing-implementation",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn main(){let _w=W::<u32>{v:7u16};}",
        false,
    ),
    (
        "missing-outlives",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,T:A<'a>>{v:T::Out}fn make<'a,'b>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,&'b u8>{v:7};w.v}fn main(){let x=0;check(make(&x,&x)==7);}",
        false,
    ),
    (
        "projection-invariance",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=u8;}struct W<T:A>{v:T::Out}fn widen<'a>(w:W<&'a u8>)->W<&'static u8>{w}fn main(){}",
        false,
    ),
    (
        "move-twice",
        "struct D;impl Drop for D{fn drop(&mut self){}}trait A{type Out;}impl A for u8{type Out=D;}struct W<T:A>{v:T::Out}fn main(){let w=W::<u8>{v:D};let _a=w.v;let _b=w.v;}",
        false,
    ),
    (
        "lifetime-only",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,'b> where &'b u8:A<'a>{v:<&'b u8 as A<'a>>::Out}fn make<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,'b>{v:7};w.v}fn main(){let x=0;check(make(&x,&x)==7);}",
        true,
    ),
    (
        "lifetime-only-missing",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,'b> where &'b u8:A<'a>{v:<&'b u8 as A<'a>>::Out}fn make<'a,'b>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,'b>{v:7};w.v}fn main(){let x=0;check(make(&x,&x)==7);}",
        false,
    ),
    (
        "unused-generic-projected-storage",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn id<T:A>(x:W<T>)->W<T>{x}fn main(){check(true);}",
        true,
    ),
    (
        "instantiated-projected-storage",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn f<T:A>(x:T::Out)->W<T>{W{v:x}}fn main(){check(f::<u8>(7).v==7);}",
        true,
    ),
    (
        "projected-reference-field",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<'a,T:A>{v:&'a T::Out}fn main(){let x=7u16;let w=W::<u8>{v:&x};check(*w.v==7);}",
        true,
    ),
    (
        "multiple-caller-proofs",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,T:A<'a>>{v:T::Out}fn good<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,&'b u8>{v:7};w.v}fn bad<'a,'b>(_:&'a u8,_:&'b u8)->u16{let w=W::<'a,&'b u8>{v:7};w.v}fn main(){let x=0;check(good(&x,&x)==7);}",
        false,
    ),
    (
        "nested-drop-zero",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}struct W<'a,T:A<'a>>{v:[T::Out;0]}struct X<'a,T:A<'a>>{w:W<'a,T>}fn main(){let _x=X::<u8>{w:W{v:[]}};check(true);}",
        true,
    ),
    (
        "caller-equality-field",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct W<T:A>{v:T::Out}fn f<T:A<Out=u16>>(w:W<T>)->u16{w.v}fn main(){check(f(W::<u8>{v:7})==7);}",
        true,
    ),
    (
        "transparent-two-projected",
        "trait A{type Out;}impl A for u8{type Out=u16;}#[repr(transparent)]struct W<T:A>{v:T::Out,w:T::Out}fn main(){}",
        false,
    ),
    (
        "transparent-nested-side",
        "trait A{type Out;}impl A for u8{type Out=u16;}#[repr(transparent)]struct W<T:A>{v:T::Out,z:((),[();0])}fn main(){let w=W::<u8>{v:7,z:((),[])};check(w.v==7);}",
        true,
    ),
];

#[test]
fn projected_nominal_fields_match_independent_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("field-normalization-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn normalized_nominal_fields_preserve_native_storage() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("normalized-nominal-fields", &cases)
}
