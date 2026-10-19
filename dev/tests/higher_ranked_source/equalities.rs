//! Independently qualified source controls for quantified associated equalities.
pub const CASES: &[(&str, &str, bool)] = &[
    (
        "equality-identity",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where for<'a>T:A<'a,Out=&'a u8>{x}fn main(){let n=7;if *f::<u8>(&n)!=7{loop{}}}",
        true,
    ),
    (
        "equality-static-only",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}fn f<T>()where for<'a>T:A<'a,Out=&'a u8>{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "equality-independent-uses",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)->(<T as A<'x>>::Out,<T as A<'y>>::Out) where for<'a>T:A<'a,Out=&'a u8>{(x,y)}fn main(){let n=7;let m=9;let p=f::<u8>(&n,&m);if *p.0!=7||*p.1!=9{loop{}}}",
        true,
    ),
    (
        "equality-escape",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'static>>::Out where for<'a>T:A<'a,Out=&'a u8>{x}fn main(){}",
        false,
    ),
    (
        "equality-empty-binder",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where for<>T:A<'x,Out=&'x u8>{x}fn main(){let n=7;if *f::<u8>(&n)!=7{loop{}}}",
        true,
    ),
    (
        "equality-nested-callable",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=for<'b>fn(&'a u8,&'b u8)->&'a u8;}fn choose<'a,'b>(a:&'a u8,_:&'b u8)->&'a u8{a}fn f<'x,T>()-><T as A<'x>>::Out where for<'a>T:A<'a,Out=for<'b>fn(&'a u8,&'b u8)->&'a u8>{choose}fn main(){let n=7;let m=9;let g=f::<u8>();if *g(&n,&m)!=7{loop{}}}",
        true,
    ),
    (
        "equality-correlated",
        r"trait A<'a,'b>{type Out;}impl<'a,'b>A<'a,'b>for u8{type Out=(&'a u8,&'b u8);}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)-><T as A<'x,'y>>::Out where for<'a,'b>T:A<'a,'b,Out=(&'a u8,&'b u8)>{(x,y)}fn main(){let n=7;let m=9;let p=f::<u8>(&n,&m);if *p.0!=7||*p.1!=9{loop{}}}",
        true,
    ),
    (
        "equality-correlated-swapped",
        r"trait A<'a,'b>{type Out;}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)-><T as A<'x,'y>>::Out where for<'a,'b>T:A<'a,'b,Out=(&'a u8,&'b u8)>{(y,x)}fn main(){}",
        false,
    ),
    (
        "equality-captured-result",
        r"trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=(&'a u8,&'x u8);}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)-><T as A<'y>>::Out where for<'a>T:A<'a,Out=(&'a u8,&'x u8)>{(y,x)}fn main(){let n=7;let m=9;let p=f::<&u8>(&n,&m);if *p.0!=9||*p.1!=7{loop{}}}",
        true,
    ),
    (
        "equality-search-input",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Mark{}impl<'a>Mark for &'a u8{}fn need<U:Mark>(){}fn f<'x,T>()where for<'a>T:A<'a,Out=&'a u8>{need::<<T as A<'x>>::Out>();}fn main(){f::<u8>();}",
        true,
    ),
    (
        "equality-wrong-impl-result",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u16;}fn f<T>()where for<'a>T:A<'a,Out=&'a u8>{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "equality-unrelated-caller",
        r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where for<'a>T:A<'a,Out=&'a u8>{x}fn g<'x,T:A<'x>>(x:&'x u8)-><T as A<'x>>::Out{x}fn main(){}",
        false,
    ),
    (
        "alternative-same-arity",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where for<'a>T:A<'a,Out=&'a u8>,for<'renamed>T:A<'renamed,Out=&'renamed u8>{x}fn main(){let n=7;if *f::<u8>(&n)!=7{loop{}}}",
        true,
    ),
    (
        "alternative-unused-slot",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where for<'a>T:A<'a,Out=&'a u8>,for<'unused,'a>T:A<'a,Out=&'a u8>{x}fn main(){let n=7;if *f::<u8>(&n)!=7{loop{}}}",
        true,
    ),
    (
        "alternative-mixed",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'x>>::Out where T:A<'x,Out=&'x u8>,for<'a>T:A<'a,Out=&'a u8>{x}fn main(){let n=7;if *f::<u8>(&n)!=7{loop{}}}",
        false,
    ),
    (
        "alternative-reordered-slots",
        "trait A<'a,'b>{type Out;}impl<'a,'b>A<'a,'b>for u8{type Out=(&'a u8,&'b u8);}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)-><T as A<'x,'y>>::Out where for<'a,'b>T:A<'a,'b,Out=(&'a u8,&'b u8)>,for<'b,'a>T:A<'a,'b,Out=(&'a u8,&'b u8)>{(x,y)}fn main(){let n=7;let m=9;let p=f::<u8>(&n,&m);if *p.0!=7||*p.1!=9{loop{}}}",
        true,
    ),
];
