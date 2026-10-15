//! Canonical nominal goals retain equality obligations in their owning region proof.

#[test]
fn nominal_trait_equality_requires_both_lifetime_directions()
-> Result<(), Box<dyn std::error::Error>> {
    super::cases(&[
        (
            "signature-equivalent",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(x:Wrap<'b,T>){}fn main(){}",
            true,
        ),
        (
            "signature-oneway",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b,T:Tr<'a>>(x:Wrap<'b,T>){}fn main(){}",
            false,
        ),
        (
            "signature-unrelated",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a,'b,T:Tr<'a>>(x:Wrap<'b,T>){}fn main(){}",
            false,
        ),
        (
            "fields-equivalent",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);struct Outer<'a:'b,'b:'a,T:Tr<'a>>{a:&'a (),b:Wrap<'b,T>}fn main(){}",
            true,
        ),
        (
            "fields-oneway",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);struct Outer<'a:'b,'b,T:Tr<'a>>{a:&'a (),b:Wrap<'b,T>}fn main(){}",
            false,
        ),
        (
            "body-equivalent",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(b:&'b (),t:T){let _:Wrap<'b,T>=Wrap(b,t);}fn main(){}",
            true,
        ),
        (
            "body-oneway",
            "trait Tr<'a>{} struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b,T:Tr<'a>>(b:&'b (),t:T){let _:Wrap<'b,T>=Wrap(b,t);}fn main(){}",
            false,
        ),
        (
            "subject-equivalent",
            "trait Flag{}struct Boxed<T:Flag>(T);fn f<'a:'b,'b:'a>(x:&'b u8)where &'a u8:Flag{let _:Boxed<&'b u8>=Boxed(x);}fn main(){}",
            true,
        ),
        (
            "subject-oneway",
            "trait Flag{}struct Boxed<T:Flag>(T);fn f<'a:'b,'b>(x:&'b u8)where &'a u8:Flag{let _:Boxed<&'b u8>=Boxed(x);}fn main(){}",
            false,
        ),
        (
            "argument-equivalent",
            "trait Tr<'a>{}trait Need<'b,U:Tr<'b>>{}fn f<'a:'b,'b:'a,T:Tr<'a>,V:Need<'b,T>>(){}fn main(){}",
            true,
        ),
        (
            "argument-oneway",
            "trait Tr<'a>{}trait Need<'b,U:Tr<'b>>{}fn f<'a:'b,'b,T:Tr<'a>,V:Need<'b,T>>(){}fn main(){}",
            false,
        ),
        (
            "wrong-type",
            "trait Tr<'a,U>{}struct Wrap<'a,T:Tr<'a,u16>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a,u8>>(x:Wrap<'b,T>){}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn trait_lifetime_permissions_stay_in_their_declaring_context()
-> Result<(), Box<dyn std::error::Error>> {
    super::cases(&[
        (
            "impl-captures",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);struct Host<'a,T>(&'a (),T);impl<'a:'b,'b:'a,T:Tr<'a>> Host<'a,T>{fn f(x:Wrap<'b,T>){}}fn main(){}",
            true,
        ),
        (
            "impl-captures-oneway",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);struct Host<'a,T>(&'a (),T);impl<'a:'b,'b,T:Tr<'a>> Host<'a,T>{fn f(x:Wrap<'b,T>){}}fn main(){}",
            false,
        ),
        (
            "trait-captures",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);trait Use<'a:'b,'b:'a,T:Tr<'a>>{fn f(x:Wrap<'b,T>);}fn main(){}",
            true,
        ),
        (
            "trait-captures-oneway",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);trait Use<'a:'b,'b,T:Tr<'a>>{fn f(x:Wrap<'b,T>);}fn main(){}",
            false,
        ),
        (
            "signature-implied",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a,'b:'a,T:Tr<'a>>(proof:&'b &'a (),x:Wrap<'b,T>){}fn main(){}",
            true,
        ),
        (
            "fields-implied",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);struct Outer<'a,'b:'a,T:Tr<'a>>{proof:&'b &'a (),x:Wrap<'b,T>}fn main(){}",
            true,
        ),
        (
            "signature-scope-isolation",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(x:Wrap<'b,T>){}fn g<'a:'b,'b,T:Tr<'a>>(x:Wrap<'b,T>){}fn main(){}",
            false,
        ),
        (
            "body-scope-isolation",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(b:&'b (),t:T){let _:Wrap<'b,T>=Wrap(b,t);}fn g<'a:'b,'b,T:Tr<'a>>(b:&'b (),t:T){let _:Wrap<'b,T>=Wrap(b,t);}fn main(){}",
            false,
        ),
        (
            "arguments-all-kinds",
            "trait Tr<'a,U,const N:usize>{}struct Wrap<'a,T:Tr<'a,u8,3>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a,u8,3>>(x:Wrap<'b,T>){}fn main(){}",
            true,
        ),
        (
            "wrong-const",
            "trait Tr<'a,U,const N:usize>{}struct Wrap<'a,T:Tr<'a,u8,3>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a,u8,4>>(x:Wrap<'b,T>){}fn main(){}",
            false,
        ),
        (
            "body-obligation-isolation",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b+'c,'b:'a,'c,T:Tr<'a>>(b:&'b (),c:&'c (),t:T,u:T){let _:Wrap<'b,T>=Wrap(b,t);let _:Wrap<'c,T>=Wrap(c,u);}fn main(){}",
            false,
        ),
        (
            "annotation-equivalent",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(){let _:Wrap<'b,T>=loop{};}fn main(){}",
            true,
        ),
        (
            "annotation-oneway",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b,T:Tr<'a>>(){let _:Wrap<'b,T>=loop{};}fn main(){}",
            false,
        ),
        (
            "annotation-subject-equivalent",
            "trait Flag{}struct Boxed<T:Flag>(T);fn f<'a:'b,'b:'a>()where &'a u8:Flag{let _:Boxed<&'b u8>=loop{};}fn main(){}",
            true,
        ),
        (
            "annotation-subject-oneway",
            "trait Flag{}struct Boxed<T:Flag>(T);fn f<'a:'b,'b>()where &'a u8:Flag{let _:Boxed<&'b u8>=loop{};}fn main(){}",
            false,
        ),
        (
            "annotation-scope-isolation",
            "trait Tr<'a>{}struct Wrap<'a,T:Tr<'a>>(&'a (),T);fn f<'a:'b,'b:'a,T:Tr<'a>>(){let _:Wrap<'b,T>=loop{};}fn g<'a:'b,'b,T:Tr<'a>>(){let _:Wrap<'b,T>=loop{};}fn main(){}",
            false,
        ),
    ])
}
