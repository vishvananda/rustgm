//! Quantified conditions retain the original projection through normalized proof publication.
pub const CASES: &[(&str, &str, bool)] = &[
    (
        "projected-impl-condition",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait B<'a>{}impl<'a>B<'a>for &'a u8{}trait C{}impl<T>C for T where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:B<'a>{}fn need<T:C>(){}fn main(){need::<u8>();}",
        true,
    ),
    (
        "projected-impl-static-only",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}trait B<'a>{}impl<'a>B<'a>for &'a u8{}trait C{}impl<T>C for T where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:B<'a>{}fn need<T:C>(){}fn main(){need::<u8>();}",
        false,
    ),
    (
        "projected-outlives-static",
        "trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=&'x u8;}fn f<T>()where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:'a{}fn main(){f::<&'static u8>();}",
        true,
    ),
    (
        "projected-outlives-short",
        "trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=&'x u8;}fn f<T>()where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:'a{}fn run<'x>(x:&'x u8){f::<&'x u8>();}fn main(){let n=7;run(&n);}",
        false,
    ),
];
