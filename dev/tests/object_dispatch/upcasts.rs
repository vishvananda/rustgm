//! Upcasts require intrinsic object bounds and use the selected target table.
const CASES: &[(&str, &str, bool)] = &[
    (
        "upcast-shared",
        "trait A{fn a(&self)->u8;}trait B:A{fn b(&self)->u8{9}}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn base(x:&dyn B)->&dyn A{x}fn main(){let v=7u8;let p:&dyn B=&v;check(base(p).a()==7);}",
        false,
    ),
    (
        "upcast-mutable",
        "trait A{fn a(&mut self);}trait B:A{}impl A for u8{fn a(&mut self){*self+=2;}}impl B for u8{}fn base(x:&mut dyn B)->&mut dyn A{x}fn main(){let mut v=7u8;let p:&mut dyn B=&mut v;base(p).a();check(v==9);}",
        false,
    ),
    (
        "upcast-constant",
        "trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}const P:&dyn B=&7u8;const Q:&dyn A=P;fn main(){check(Q.a()==7);}",
        false,
    ),
    (
        "upcast-literal",
        "trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn main(){let p:&dyn B=&7u8;let q:&dyn A=p;check(q.a()==7);}",
        false,
    ),
    (
        "upcast-diamond",
        "trait A{fn a(&self)->u8;}trait B:A{fn b(&self)->u8{2}}trait C:A{fn c(&self)->u8{3}}trait D:B+C{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}impl C for u8{}impl D for u8{}fn main(){let v=7u8;let d:&dyn D=&v;let b:&dyn B=d;let c:&dyn C=d;let a:&dyn A=c;check(b.b()==2&&c.c()==3&&a.a()==7);}",
        false,
    ),
    (
        "upcast-associated",
        "trait A{type V;fn a(&self)->Self::V;}trait B:A<V=u16>{}impl A for u8{type V=u16;fn a(&self)->u16{1000}}impl B for u8{}fn main(){let v=7u8;let p:&dyn B=&v;let q:&dyn A<V=u16>=p;check(q.a()==1000);}",
        false,
    ),
    (
        "upcast-generic-parent",
        "trait A<T>{fn a(&self,x:T)->T;}trait B:A<u16>{}impl A<u16>for u8{fn a(&self,x:u16)->u16{x+2}}impl B for u8{}fn main(){let v=7u8;let p:&dyn B=&v;let q:&dyn A<u16>=p;check(q.a(1000)==1002);}",
        false,
    ),
    (
        "upcast-higher-ranked",
        "trait A<'a>{fn a(&self,x:&'a u8)->&'a u8;}trait B:for<'a>A<'a>{}impl<'a>A<'a>for u8{fn a(&self,x:&'a u8)->&'a u8{x}}impl B for u8{}fn base(x:&dyn B)->&dyn for<'a>A<'a>{x}fn main(){let v=7u8;let x=9u8;check(*base(&v).a(&x)==9);}",
        false,
    ),
    (
        "upcast-quantifier-specialization",
        "trait A<'a>{fn a(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn a(&self,x:&'a u8)->&'a u8{x}}fn main(){let v=7u8;let p:&dyn for<'a>A<'a>=&v;let q:&dyn A<'static>=p;check(*q.a(&9u8)==9);}",
        false,
    ),
    (
        "upcast-raw",
        "trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn main(){let v=7u8;let p:&dyn B=&v;let raw=p as *const dyn B;let q:*const dyn A=raw;let _=q;}",
        false,
    ),
    (
        "upcast-two-concretes",
        "trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl A for u16{fn a(&self)->u8{(*self+2)as u8}}impl B for u8{}impl B for u16{}fn base(x:&dyn B)->&dyn A{x}fn main(){let a=7u8;let b=9u16;check(base(&a).a()==7&&base(&b).a()==11);}",
        false,
    ),
    (
        "upcast-discovered-before-table",
        "trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn base(x:&dyn B)->&dyn A{x}fn make(x:&u8)->&dyn B{x}fn main(){let f=base as for<'a>fn(&'a(dyn B+'a))->&'a(dyn A+'a);let v=7u8;check(f(make(&v)).a()==7);}",
        false,
    ),
];

const REJECTED: &[(&str, &str)] = &[
    (
        "upcast-unrelated-impl",
        "trait A{}trait B{}impl B for dyn A+'_{}fn bad(x:&dyn A)->&dyn B{x}fn main(){}",
    ),
    (
        "upcast-downcast",
        "trait A{}trait B:A{}fn bad(x:&dyn A)->&dyn B{x}fn main(){}",
    ),
    (
        "upcast-wrong-argument",
        "trait A<T>{}trait B:A<u8>{}fn bad(x:&dyn B)->&dyn A<u16>{x}fn main(){}",
    ),
    (
        "upcast-wrong-equality",
        "trait A{type V;}trait B:A<V=u8>{}fn bad(x:&dyn B)->&dyn A<V=u16>{x}fn main(){}",
    ),
    (
        "upcast-weakened-quantifier",
        "trait A<'a>{}fn bad<'s>(x:&'s dyn A<'static>)->&'s dyn for<'a>A<'a>{x}fn main(){}",
    ),
    (
        "upcast-strengthened-object-lifetime",
        "trait A{}trait B:A{}fn bad<'a>(x:&'a(dyn B+'a))->&'a(dyn A+'static){x}fn main(){}",
    ),
    (
        "upcast-shared-to-mutable",
        "trait A{}trait B:A{}fn bad(x:&dyn B)->&mut dyn A{x}fn main(){}",
    ),
];

#[test]
fn upcasts_lower() -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for &(name, source, _) in CASES {
        let source = format!("{}{source}", super::native::CHECK);
        super::rustc::qualify(name, &source, true)?;
        if let Err(error) = super::build(&source) {
            errors.push(format!("{name}: {error}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

#[test]
fn invalid_upcasts_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in REJECTED {
        super::rustc::qualify(name, source, false)?;
        assert!(super::build(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn upcasts_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("pa24-pa25-work/native-upcasts", CASES)
}
