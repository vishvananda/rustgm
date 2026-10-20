//! Runtime object calls use selected methods and preserve the ordinary Rust ABI.
#[path = "object_dispatch/auto_bounds.rs"]
mod auto_bounds;
#[path = "object_dispatch/containers.rs"]
mod containers;
#[path = "object_dispatch/erased_cleanup.rs"]
mod erased_cleanup;
#[path = "object_dispatch/hidden_dispatch.rs"]
mod hidden_dispatch;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "object_dispatch/tails.rs"]
mod tails;
#[path = "object_dispatch/upcasts.rs"]
mod upcasts;
use rustgm::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "object-static-trait-local-value",
        "trait A:'static{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn read(x:&dyn A)->u8{x.f()}fn main(){let v=7u8;check(read(&v)==7);}",
        false,
    ),
    (
        "object-shared",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self+2}}fn read(x:&dyn A)->u8{x.f()}fn main(){let v=7u8;check(read(&v)==9);}",
        false,
    ),
    (
        "object-mutable",
        "trait A{fn f(&mut self,n:u8)->u8;}impl A for u8{fn f(&mut self,n:u8)->u8{*self+=n;*self}}fn main(){let mut v=7u8;let p:&mut dyn A=&mut v;check(p.f(2)==9);check(v==9);}",
        false,
    ),
    (
        "object-two-implementations",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}impl A for u16{fn f(&self)->u8{(*self+2)as u8}}fn read(x:&dyn A)->u8{x.f()}fn main(){let a=7u8;let b=8u16;check(read(&a)==7&&read(&b)==10);}",
        false,
    ),
    (
        "object-default",
        "trait A{fn f(&self)->u8{self.g()+1}fn g(&self)->u8;}impl A for u8{fn g(&self)->u8{*self}}fn main(){let v=7u8;let p:&dyn A=&v;check(p.f()==8);}",
        false,
    ),
    (
        "object-generic-impl",
        "trait A{fn f(&self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(&self)->u8{let _=self.0;7}}fn main(){let v=W(9u16);let p:&dyn A=&v;check(p.f()==7);}",
        false,
    ),
    (
        "object-supertrait",
        "trait A{fn a(&self)->u8;}trait B:A{fn b(&self)->u8;}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{fn b(&self)->u8{*self+2}}fn main(){let v=7u8;let p:&dyn B=&v;check(p.a()==7&&p.b()==9);}",
        false,
    ),
    (
        "object-literal",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let p:&dyn A=&7u8;check(p.f()==7);}",
        false,
    ),
    (
        "object-constant",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}const P:&dyn A=&7u8;fn main(){check(P.f()==7);}",
        false,
    ),
    (
        "object-aggregate-abi",
        "trait A{fn f(&self,x:(u8,u16))->(u16,u8);}impl A for u8{fn f(&self,x:(u8,u16))->(u16,u8){(x.1,*self+x.0)}}fn main(){let v=7u8;let p:&dyn A=&v;let r=p.f((2,1000));check(r.0==1000&&r.1==9);}",
        false,
    ),
    (
        "object-associated-abi",
        "trait A{type V;fn f(&self,x:Self::V)->Self::V;}impl A for u8{type V=(u8,u16);fn f(&self,x:Self::V)->Self::V{(*self+x.0,x.1)}}fn main(){let v=7u8;let p:&dyn A<V=(u8,u16)>=&v;let r=p.f((2,1000));check(r.0==9&&r.1==1000);}",
        false,
    ),
    (
        "object-captured-reference",
        "trait A{fn f(&self)->&u8;}struct W<'a>(&'a u8);impl<'a>A for W<'a>{fn f(&self)->&u8{self.0}}fn main(){let v=7u8;let w=W(&v);let p:&dyn A=&w;check(*p.f()==7);}",
        false,
    ),
    (
        "object-late-method",
        "trait A{fn f<'a>(&self,x:&'a u8)->&'a u8;}impl A for u8{fn f<'a>(&self,x:&'a u8)->&'a u8{x}}fn main(){let v=7u8;let x=9u8;let p:&dyn A=&v;check(*p.f(&x)==9);}",
        false,
    ),
    (
        "object-higher-ranked-owner",
        "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(&self,x:&'a u8)->&'a u8{x}}fn main(){let v=7u8;let x=9u8;let p:&dyn for<'a>A<'a>=&v;check(*p.f(&x)==9);}",
        false,
    ),
    (
        "object-reified-method",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let v=7u8;let p:&(dyn A+'static)=&v;let f=<dyn A>::f as fn(&(dyn A+'static))->u8;check(f(p)==7);}",
        false,
    ),
    (
        "object-generic-consumer",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn read<T:A+?Sized>(p:&T)->u8{p.f()}fn main(){let v=7u8;let p:&dyn A=&v;check(read(p)==7);}",
        false,
    ),
    (
        "object-drop-table",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a mut u8);impl<'a>A for W<'a>{fn f(&self)->u8{*self.0}}impl<'a>Drop for W<'a>{fn drop(&mut self){*self.0+=2;}}fn read(x:&dyn A)->u8{x.f()}fn main(){let mut v=7u8;{let w=W(&mut v);check(read(&w)==7);}check(v==9);}",
        false,
    ),
    (
        "object-foreign-trait-impl",
        "trait A{fn f(&self)->u8;}trait B{fn g(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}impl B for dyn A+'_ {fn g(&self)->u8{self.f()+2}}fn main(){let v=7u8;let p:&dyn A=&v;check(p.g()==9);}",
        false,
    ),
    (
        "object-vacant-copy-method",
        "trait A{type V;fn g(&self)->u8{7}fn f(&self,x:Self::V)->u8 where Self::V:Copy{let a=x;let b=x;9}}struct Q(u8);impl A for u8{type V=Q;}fn main(){let v=2u8;let p:&dyn A<V=Q>=&v;check(p.g()==7);}",
        false,
    ),
    (
        "object-applicable-copy-method",
        "trait A{type V;fn f(&self,x:Self::V)->u8 where Self::V:Copy{let a=x;let b=x;9}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;check(p.f(3)==9);}",
        false,
    ),
    (
        "object-vacant-trait-method",
        "trait E{}trait A{type V;fn g(&self)->u8{7}fn f(&self)->u8 where Self::V:E{9}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;check(p.g()==7);}",
        false,
    ),
    (
        "object-parent-type-argument",
        "trait A<T>{fn f(&self,x:T)->T;}impl A<u16>for u8{fn f(&self,x:u16)->u16{x+2}}fn main(){let v=7u8;let p:&dyn A<u16>=&v;check(p.f(1000)==1002);}",
        false,
    ),
    (
        "object-parent-const-argument",
        "trait A<const N:usize>{fn f(&self,x:[u8;N])->u8;}impl A<2>for u8{fn f(&self,x:[u8;2])->u8{x[1]}}fn main(){let v=7u8;let p:&dyn A<2>=&v;check(p.f([2,9])==9);}",
        false,
    ),
    (
        "object-constant-bytes",
        "trait A{fn f(&self)->u8;}impl A for [u8;2]{fn f(&self)->u8{self[1]}}const P:&dyn A=b\"ab\";fn main(){check(P.f()==98);}",
        false,
    ),
    (
        "object-constant-raw-address",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}const P:*const dyn A=7usize as *const u8;fn main(){let _=P;}",
        false,
    ),
];

fn build(source: &str) -> Result<rustgm::lowir::program::Program, Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let main = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Function && report.package.names.get(d.name) == "main")
        .ok_or("missing main")?;
    Ok(rustgm::lower::instances::build(
        &report,
        &mut facts,
        DefId(u32::try_from(main)?),
    )?)
}

#[test]
fn object_calls_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(name, &source, true)?;
        let output = build(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(
            output
                .globals
                .iter()
                .any(|g| g.name.starts_with("rg.vtable.")),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn object_method_pointer_keeps_object_lifetime() -> Result<(), Box<dyn std::error::Error>> {
    let source = "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let f=<dyn A>::f as fn(&dyn A)->u8;}";
    rustc::qualify("object-reified-object-lifetime", source, false)?;
    assert!(build(source).is_err());
    Ok(())
}

#[test]
fn vacant_methods_require_their_declared_conditions() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "object-vacant-copy-call",
            "trait A{type V;fn f(&self,x:Self::V)where Self::V:Copy{let a=x;let b=x;}}struct Q(u8);impl A for u8{type V=Q;}fn main(){let v=2u8;let p:&dyn A<V=Q>=&v;p.f(Q(1));}",
        ),
        (
            "object-vacant-copy-reification",
            "trait A{type V;fn f(&self,x:Self::V)where Self::V:Copy{let a=x;let b=x;}}struct Q(u8);impl A for u8{type V=Q;}fn main(){let f=<dyn A<V=Q>>::f;}",
        ),
        (
            "object-vacant-trait-call",
            "trait E{}trait A{type V;fn f(&self)where Self::V:E{}}impl A for u8{type V=u16;}fn main(){let v=2u8;let p:&dyn A<V=u16>=&v;p.f();}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(build(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn object_type_relations_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for (name, source, accepted) in [
        (
            "object-local-reference-elision",
            "trait A{}impl A for &u8{}fn f<'a>(x:&'a &'a u8){let p:&dyn A=x;let q:&'a(dyn A+'a)=p;}fn main(){}",
            true,
        ),
        (
            "object-redundant-inherited-equality",
            "trait A{type V;}trait B:A<V=u8>{}fn f(x:&dyn B)->&dyn B<V=u8>{x}fn main(){}",
            true,
        ),
        (
            "object-intrinsic-static",
            "trait A:'static{}fn prove<T:?Sized+'static>(_:&T){}fn f<'a>(x:&'a(dyn A+'a)){prove(x)}fn main(){}",
            true,
        ),
        (
            "object-intrinsic-outlives",
            "trait A<'a>:'a{}fn prove<'a,T:?Sized+'a>(_:&T){}fn f<'a:'b,'b>(x:&'b(dyn A<'a>+'b)){prove::<'a,_>(x)}fn main(){}",
            true,
        ),
        (
            "object-no-intrinsic-static",
            "trait A{}fn prove<T:?Sized+'static>(_:&T){}fn f<'a>(x:&'a(dyn A+'a)){prove(x)}fn main(){}",
            false,
        ),
        (
            "object-multiple-lifetime-promises",
            "trait A<'a,'b>:'a+'b{}fn prove<'a,T:?Sized+'a>(_:&T){}fn f<'a,'b,'c>(x:&'c(dyn A<'a,'b>+'c)){prove::<'a,_>(x);prove::<'b,_>(x)}fn main(){}",
            true,
        ),
        (
            "object-reference-formation-implies-input-lifetimes",
            "trait A<'a>{}fn prove<T:'static>(){}fn f<'a>(_:&'static(dyn A<'a>+'static)){prove::<&'a u8>();}fn main(){}",
            true,
        ),
        (
            "object-short-reference-does-not-imply-static-input",
            "trait A<'a>{}fn prove<T:'static>(){}fn f<'a,'b>(_:&'b(dyn A<'a>+'static)){prove::<&'a u8>();}fn main(){}",
            false,
        ),
        (
            "object-explicit-bound-constrains-input-lifetime",
            "trait A:'static{}fn f<'a>(x:&'a(dyn A+'a))->&'static dyn A{x}fn main(){}",
            true,
        ),
        (
            "object-static-trait-cannot-extend-local-borrow",
            "trait A:'static{}impl A for u8{}fn f<'a>(x:&'a(dyn A+'a))->&'static dyn A{x}fn main(){let v=7u8;let p=f(&v);let _=p;}",
            false,
        ),
        (
            "object-local-annotation-requires-principal-lifetime",
            "trait A:'static{}impl A for u8{}fn f<'a>(x:&'a u8){let p:&'a(dyn A+'a)=x;}fn main(){}",
            false,
        ),
        (
            "object-static-with-short-trait-input",
            "trait A<'a>{}fn f<'a>(x:&'static(dyn A<'a>+'static))->&'static(dyn A<'a>+'static){x}fn main(){}",
            true,
        ),
        (
            "object-local-principal-lifetime-does-not-imply-static",
            "trait A<'a>:'a{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn for<'a>A<'a>+'x)){prove(x)}fn main(){}",
            false,
        ),
        (
            "object-local-supertrait-lifetime-does-not-imply-static",
            "trait A<'a>:'a{}trait B:for<'a>A<'a>{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn B+'x)){prove(x)}fn main(){}",
            false,
        ),
        (
            "object-argument-promise-does-not-imply-static",
            "trait A<T>where T:'static{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn A<u8>+'x)){prove(x)}fn main(){}",
            false,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
        let result = build(source);
        if result.is_ok() != accepted {
            errors.push(format!(
                "{name}: expected {accepted}, {}",
                result.err().map_or("accepted".into(), |e| e.to_string())
            ));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn object_calls_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("pa24-pa25-work/native-object-dispatch", CASES)
}
