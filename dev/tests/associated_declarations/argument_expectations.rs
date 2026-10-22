//! Prefix context can coerce function items without borrowing later argument facts.
use super::{native, rustc, semantic};
const CASES: &[(&str, bool, &str)] = &[
    (
        "retained",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
",
    ),
    (
        "qualified",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(<&S as K<fn(&S)->&S>>::accepts(&&a,id));}
",
    ),
    (
        "explicit-pointer",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let f:fn(&S)->&S=id;check(K::accepts(&&a,f));}
",
    ),
    (
        "generic-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id<T>(x:&T)->&T{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
",
    ),
    (
        "ordinary-generic-free",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn apply<R>(s:&&S,r:R)->bool where for<'a> &'a S:K<R>{K::accepts(s,r)}fn main(){let a=S(3);check(apply(&&a,id));}
",
    ),
    (
        "competing-pointer",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}impl K<fn(u8)->u8>for&S{fn accepts(&self,_:fn(u8)->u8)->bool{false}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
",
    ),
    (
        "unsafe-item",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}unsafe fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
",
    ),
    (
        "wrong-convention",
        false,
        r#"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}extern "C" fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
"#,
    ),
    (
        "ordinary-identity",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(x:&S)->&S{x}fn identity<T>(x:T)->T{x}fn main(){let a=S(3);check(identity(id)(&a).0==3);}struct S(u8);
",
    ),
    (
        "weakened-safe-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<unsafe fn(&S)->&S>for&S{fn accepts(&self,r:unsafe fn(&S)->&S)->bool{unsafe{r(self)}.0==self.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id));}
",
    ),
    (
        "unconstrained-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn keep<T>(_:T){}fn id(x:u8)->u8{x}fn main(){keep(id);check(true);}
",
    ),
    (
        "competing-explicit-pointer",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}impl K<fn(u8)->u8>for&S{fn accepts(&self,_:fn(u8)->u8)->bool{false}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let f:fn(&S)->&S=id;check(K::accepts(&&a,f));}
",
    ),
    (
        "item-before-self",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(id,&&a));}
",
    ),
    (
        "unpaid-static-result",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S;impl K<fn(&S)->&'static S>for&S{fn accepts(&self,_:fn(&S)->&'static S)->bool{true}}fn id(x:&S)->&S{x}fn main(){let a=S;check(K::accepts(&&a,id));}
",
    ),
    (
        "crossed-output",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S;impl K<for<'a,'b>fn(&'a S,&'b S)->&'a S>for&S{fn accepts(&self,_:for<'a,'b>fn(&'a S,&'b S)->&'a S)->bool{true}}fn id<'a,'b>(_:&'a S,x:&'b S)->&'b S{x}fn main(){let a=S;check(K::accepts(&&a,id));}
",
    ),
    (
        "result-self-context",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn make(r:R)->Self;}struct S(u8);impl K<fn(&S)->&S>for S{fn make(r:fn(&S)->&S)->Self{S(r(&S(3)).0)}}fn id(x:&S)->&S{x}fn main(){let a:S=K::make(id);check(a.0==3);}
",
    ),
    (
        "result-rhs-context",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn choose<R>(r:R)->R where for<'a> &'a S:K<R>{r}fn main(){let a=S(3);let f:fn(&S)->&S=choose(id);check(f(&a).0==3);}
",
    ),
    (
        "two-items-with-prefix",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,a:R,b:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,a:fn(&S)->&S,b:fn(&S)->&S)->bool{a(self).0==3&&b(self).0==3}}fn id(x:&S)->&S{x}fn other(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id,other));}
",
    ),
    (
        "two-items-no-bound",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn other(x:u8)->u8{x}fn main(){same(id,other);}
",
    ),
    (
        "item-before-pointer",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn main(){let f:fn(u8)->u8=id;same(id,f);}
",
    ),
    (
        "pointer-before-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn main(){let f:fn(u8)->u8=id;same(f,id);check(f(3)==3);}
",
    ),
    (
        "indirect-call",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let f=K::accepts;check(f(&&a,id));}
",
    ),
    (
        "generic-free-first-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn apply<R>(r:R,s:&&S)->bool where for<'a> &'a S:K<R>{K::accepts(s,r)}fn main(){let a=S(3);check(apply(id,&&a));}
",
    ),
    (
        "competing-result-context",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}impl K<fn(u8)->u8>for&S{fn accepts(&self,_:fn(u8)->u8)->bool{false}}fn id(x:&S)->&S{x}fn choose<R>(r:R)->R where for<'a> &'a S:K<R>{r}fn main(){let a=S(3);let f:fn(&S)->&S=choose(id);check(f(&a).0==3);}
",
    ),
    (
        "two-items-result-context",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn choose<T>(a:T,_:T)->T{a}fn id(x:u8)->u8{x}fn other(x:u8)->u8{x+1}fn main(){let f:fn(u8)->u8=choose(id,other);check(f(3)==3);}
",
    ),
    (
        "item-before-block-self",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(id,{&&a}));}
",
    ),
    (
        "item-before-wrapper-self",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);trait K<R>:Sized{fn accepts(r:R,s:Wrap<Self>)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:Wrap<Self>)->bool{r(s.0).0==s.0.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(id,Wrap(&a)));}
",
    ),
    (
        "block-self-before-item",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(s:&Self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(s:&&S,r:fn(&S)->&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts({&&a},id));}
",
    ),
    (
        "item-before-pointer-block",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn main(){same(id,{let f:fn(u8)->u8=id;f});}
",
    ),
    (
        "pointer-block-before-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn main(){same({let f:fn(u8)->u8=id;f},id);check(true);}
",
    ),
    (
        "qualified-item-before-block",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(<&S as K<fn(&S)->&S>>::accepts(id,{&&a}));}
",
    ),
    (
        "item-before-stable-block-self",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let r=&a;check(K::accepts(id,{&r}));}
",
    ),
    (
        "stable-block-self-before-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(s:&Self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(s:&&S,r:fn(&S)->&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let r=&a;check(K::accepts({&r},id));}
",
    ),
    (
        "qualified-item-before-stable-block",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);let r=&a;check(<&S as K<fn(&S)->&S>>::accepts(id,{&r}));}
",
    ),
    (
        "later-unpaid-obligation",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}trait Missing{}fn requires<T:Missing>(_:T){}fn main(){let a=S(3);check(K::accepts(&&a,id));requires(0u8);}
",
    ),
    (
        "numeric-fallback-after-prefix",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}trait Number{fn value()->u8;}impl Number for i32{fn value()->u8{3}}impl Number for u8{fn value()->u8{5}}fn number<T:Number>(_:T)->u8{T::value()}fn main(){let a=S(3);check(K::accepts(&&a,id));check(number(0)==3);}
",
    ),
    (
        "never-fallback-after-prefix",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn plain<T>(_:T){}fn main(){let a=S(3);check(K::accepts(&&a,id));if false{plain(loop{});}check(true);}
",
    ),
    (
        "unused-invalid-body",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn unused(){let a=S(3);let _=K::accepts(&&a,id);let _:u8=true;}fn main(){}
",
    ),
    (
        "conditional-pointee-unpaid",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Marker{}trait K<R>{fn accepts(&self,r:R)->bool;}struct S<T>(T);impl<T:Marker>K<fn(&S<T>)->&S<T>>for&S<T>{fn accepts(&self,_:fn(&S<T>)->&S<T>)->bool{true}}fn id<T>(x:&S<T>)->&S<T>{x}fn main(){let a=S(3u8);check(K::accepts(&&a,id));}
",
    ),
    (
        "forward-constant-prefix",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=accept(id);trait K<R>{}struct S;impl K<fn(u8)->u8> for S{}const fn accept<R:Copy>(_:R)->u8 where S:K<R>{3}fn id(x:u8)->u8{x}fn main(){check(N==3);}
",
    ),
    (
        "header-suspension",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize;}struct X<const N:usize>;trait K<R>{}impl K<fn(u8)->u8> for X<{<S0 as A>::C}>{} const N:u8=accept(id);const fn accept<R:Copy>(_:R)->u8 where X<3>:K<R>{3} fn id(x:u8)->u8{x} struct S0;impl A for S0{const C:usize=3;}fn main(){check(N==3);}
",
    ),
];
#[test]
fn function_inputs_match_qualified_prefix_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, source) in CASES {
        rustc::qualify(
            &format!("parallel/pa26/function-inputs/source/{name}"),
            source,
            accepted,
        )?;
        let result = (|| {
            let mut report = semantic::resolve(source)?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            if matches!(name, "forward-constant-prefix" | "header-suspension") {
                eprintln!("prefix {name} suspensions={}", facts.body_work.suspensions);
            }
            if name == "header-suspension" {
                assert!(facts.body_work.suspensions > 0);
            }
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })();
        let implemented = accepted;
        if result.is_ok() != implemented {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn function_inputs_preserve_native_calls() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|c| c.1)
        .map(|&(name, _, source)| {
            Ok((
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing native check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/function-inputs/native", &cases)
}
