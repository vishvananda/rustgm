//! Quantified result contracts retain declaration ownership and caller lifetime limits.
pub const CASES: &[(&str, &str, bool)] = &[
    (
        "result-declared-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Family{type Item:for<'a>A<'a,Out=&'a u8>;}impl Family for u16{type Item=u8;}fn f<'x,T:Family>(x:&'x u8)-><<T as Family>::Item as A<'x>>::Out{x}fn main(){let n=7;if *f::<u16>(&n)!=7{loop{}}}\n",
        true,
    ),
    (
        "result-header-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Family where for<'a>Self::Item:A<'a,Out=&'a u8>{type Item;}impl Family for u16{type Item=u8;}fn f<'x,T:Family>(x:&'x u8)-><<T as Family>::Item as A<'x>>::Out{x}fn main(){let n=7;if *f::<u16>(&n)!=7{loop{}}}\n",
        true,
    ),
    (
        "result-unused-bad-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}trait Family{type Item:for<'a>A<'a,Out=&'a u8>;}impl Family for u16{type Item=u8;}fn main(){}\n",
        false,
    ),
    (
        "result-captured-owner",
        "trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=(&'a u8,&'x u8);}trait Family<'x>{type Item:for<'a>A<'a,Out=(&'a u8,&'x u8)>;}impl<'x>Family<'x>for &'x u8{type Item=&'x u8;}fn f<'x,'y,T:Family<'x>>(x:&'x u8,y:&'y u8)-><<T as Family<'x>>::Item as A<'y>>::Out{(y,x)}fn main(){let n=7;let m=9;let p=f::<&u8>(&n,&m);if *p.0!=9||*p.1!=7{loop{}}}\n",
        true,
    ),
    (
        "result-outlives-result",
        "trait A<'a>:'a{}impl<'a>A<'a>for u8{}trait Family{type Item:for<'a>A<'a>;}impl Family for u16{type Item=u8;}fn need<'a,X:'a>(){}fn f<T:Family>(){need::<'static,<T as Family>::Item>();}fn main(){f::<u16>();}\n",
        false,
    ),
    (
        "result-unused-short-result",
        "trait A<'a>:'a{}impl<'a,'x:'a>A<'a>for &'x u8{}trait Family<'x>{type Item:for<'a>A<'a>;}impl<'x>Family<'x>for &'x u8{type Item=&'x u8;}fn main(){}\n",
        false,
    ),
    (
        "result-nested-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Second{type Value:for<'a>A<'a,Out=&'a u8>;}impl Second for u32{type Value=u8;}trait Family{type Item:Second;}impl Family for u16{type Item=u32;}fn f<'x,T:Family>(x:&'x u8)-><<<T as Family>::Item as Second>::Value as A<'x>>::Out{x}fn main(){let n=7;if *f::<u16>(&n)!=7{loop{}}}\n",
        true,
    ),
    (
        "result-inherited-result",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Mid:for<'a>A<'a,Out=&'a u8>{}impl Mid for u8{}trait Family{type Item:Mid;}impl Family for u16{type Item=u8;}fn f<'x,T:Family>(x:&'x u8)-><<T as Family>::Item as A<'x>>::Out{x}fn main(){let n=7;if *f::<u16>(&n)!=7{loop{}}}\n",
        true,
    ),
    (
        "result-result-method",
        "trait A<'a>{fn read(x:&'a u8)->u8;}impl<'a>A<'a>for u8{fn read(x:&'a u8)->u8{*x}}trait Family{type Item:for<'a>A<'a>;}impl Family for u16{type Item=u8;}fn f<'x,T:Family>(x:&'x u8)->u8{<<T as Family>::Item as A<'x>>::read(x)}fn main(){let n=7;if f::<u16>(&n)!=7{loop{}}}\n",
        true,
    ),
    (
        "result-explicit-static-result",
        "trait A<'a>:'a{}impl<'a>A<'a>for u8{}trait Family{type Item:for<'a>A<'a>+'static;}impl Family for u16{type Item=u8;}fn need<'a,X:'a>(){}fn f<T:Family>(){need::<'static,<T as Family>::Item>();}fn main(){f::<u16>();}\n",
        true,
    ),
    (
        "result-explicit-quantified-outlives",
        "trait Family where for<'a>Self::Item:'a{type Item;}impl Family for u16{type Item=u8;}fn need<'a,X:'a>(){}fn f<T:Family>(){need::<'static,<T as Family>::Item>();}fn main(){f::<u16>();}\n",
        false,
    ),
];
