//! Callable bounds wait for complete caller assumptions before projection formation.
pub const CASES: &[(&str, &str, bool)] = &[
    (
        "callable-where-function-unused",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T>()where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn main(){}",
        true,
    ),
    (
        "callable-inline-function-unused",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T:for<'a>A<'a>>()where (for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn main(){}",
        true,
    ),
    (
        "callable-reordered-function-unused",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T>()where (for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept,T:for<'a>A<'a>{}fn main(){}",
        true,
    ),
    (
        "callable-where-function",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T>()where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-inline-function",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T:for<'a>A<'a>>()where (for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-reordered-function",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T>()where (for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept,T:for<'a>A<'a>{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-argument-control",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T:for<'a>A<'a>>(g:for<'a>fn(&'a u8)-><T as A<'a>>::Out){}fn id(x:&u8)->&u8{x}fn main(){f::<u8>(id);}",
        true,
    ),
    (
        "nested-callable-input",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}trait Need{}impl<T>Need for T where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn need<T:Need>(){}fn main(){need::<u8>();}",
        true,
    ),
    (
        "nested-callable-input-wrong-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}trait Need{}impl<T>Need for T where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn need<T:Need>(){}fn main(){need::<u8>();}",
        false,
    ),
    (
        "callable-nested-capture",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8,for<'b>fn(&'b u8)->&'a u8)->u8{}fn f<T:for<'a>A<'a>>()where (for<'a>fn(&'a u8,for<'b>fn(&'b u8)-><T as A<'a>>::Out)->u8):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-empty-capture",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8,fn(u8)->&'a u8)->u8{}fn f<T:for<'a>A<'a>>()where (for<'a>fn(&'a u8,fn(u8)-><T as A<'a>>::Out)->u8):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-executed-identity",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}fn f<T>(g:for<'a>fn(&'a u8)-><T as A<'a>>::Out)->for<'a>fn(&'a u8)-><T as A<'a>>::Out where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{g}fn id(x:&u8)->&u8{x}fn main(){let n=7;let g=f::<u8>(id);if *g(&n)!=7{loop{}}}",
        true,
    ),
    (
        "callable-siblings-distinct-inputs",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->&'a u8{}impl Accept for for<'a>fn(&'a u16)->&'a u8{}fn f<T>()where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept,(for<'b>fn(&'b u16)-><T as A<'b>>::Out):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-tuple-siblings",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Both{}impl Both for (for<'a>fn(&'a u8)->&'a u8,for<'b>fn(&'b u16)->&'b u8){}fn f<T>()where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out,for<'b>fn(&'b u16)-><T as A<'b>>::Out):Both{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-scope-return",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait B<'a>{type Out;}impl<'a>B<'a>for u8{type Out=&'a u16;}trait Accept{}impl Accept for for<'a>fn(&'a u8,&'a u8,for<'b>fn(&'b u8)->&'b u8)->&'a u16{}fn f<T>()where T:for<'a>A<'a>,T:for<'b>B<'b>,(for<'a>fn(&'a u8,<T as A<'a>>::Out,for<'b>fn(&'b u8)-><T as A<'b>>::Out)-><T as B<'a>>::Out):Accept{}fn main(){f::<u8>();}",
        true,
    ),
    (
        "callable-scope-return-wrong-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait B<'a>{type Out;}impl<'a>B<'a>for u8{type Out=&'static u16;}trait Accept{}impl Accept for for<'a>fn(&'a u8,&'a u8,for<'b>fn(&'b u8)->&'b u8)->&'a u16{}fn f<T>()where T:for<'a>A<'a>,T:for<'b>B<'b>,(for<'a>fn(&'a u8,<T as A<'a>>::Out,for<'b>fn(&'b u8)-><T as A<'b>>::Out)-><T as B<'a>>::Out):Accept{}fn main(){f::<u8>();}",
        false,
    ),
    (
        "callable-result-introduces-binder",
        "trait B<'b>{type Out;}impl<'a,'b>B<'b>for &'a u8{type Out=(&'a u8,&'b u8);}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=for<'b>fn(&'b u8)-><&'a u8 as B<'b>>::Out;}trait Accept{}impl Accept for for<'a>fn(&'a u8)->for<'b>fn(&'b u8)->(&'a u8,&'b u8){}fn f<T>()where T:for<'a>A<'a>,(for<'a>fn(&'a u8)-><T as A<'a>>::Out):Accept{}fn main(){f::<u8>();}",
        true,
    ),
];
