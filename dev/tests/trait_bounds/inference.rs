//! Caller assumptions constrain complete trait goals before ordinary inference completion.
const CASES: &[(&str, &str, bool)] = &[
    (
        "argument-only-type",
        "trait A<U>{}fn g<T:A<U>,U>(x:T){}fn f<T:A<u8>>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "inferred-lifetime",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a,T:A<'a>>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "equivalent-lifetime",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a:'b,'b:'a,T:A<'a>>(x:T){g::<'b,T>(x)}fn main(){}",
        true,
    ),
    (
        "ambiguous-argument",
        "trait A<U>{}fn g<T:A<U>,U>(x:T){}fn f<T:A<u8>+A<u16>>(x:T){g(x)}fn main(){}",
        false,
    ),
    (
        "inferred-constant",
        "trait A<const N:usize>{}fn g<const N:usize,T:A<N>>(x:T){}fn f<T:A<2>>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "duplicate-types-call",
        "trait A<U>{}fn g<T:A<U>,U>(x:T){}fn f<T:A<u8>+A<u8>>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "ordered-independent-goals",
        "trait A<U>{}trait B<U>{}fn g<T:A<U>,V:B<U>,U>(x:T,y:V){}fn f<T:A<u8>+A<u16>,V:B<u8>>(x:T,y:V){g(x,y)}fn main(){}",
        true,
    ),
    (
        "ordered-independent-goals-reversed",
        "trait A<U>{}trait B<U>{}fn g<T:A<U>,V:B<U>,U>(x:T,y:V){}fn f<T:A<u8>,V:B<u8>+B<u16>>(x:T,y:V){g(x,y)}fn main(){}",
        true,
    ),
    (
        "subject-only-inference",
        "trait A{}fn g<T:A>(){}fn f<T:A>(){g()}fn main(){}",
        false,
    ),
    (
        "parameter-subject-inference",
        "trait A<U>{}fn g<T:A<U>,U>(y:U){}fn f<T:A<u8>>(){g(7u8)}fn main(){}",
        false,
    ),
    (
        "wrong-lifetime",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a,'b,T:A<'a>>(x:T){g::<'b,T>(x)}fn main(){}",
        false,
    ),
    (
        "wrong-lifetime-one-direction",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a:'b,'b,T:A<'a>>(x:T){g::<'b,T>(x)}fn main(){}",
        false,
    ),
    (
        "transitive-argument",
        "trait A<U>:B<U>{}trait B<U>{}fn g<T:B<U>,U>(x:T){}fn f<T:A<u8>>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "rigid-argument",
        "trait A<U>{}fn g<T:A<V>,V>(x:T){}fn f<T:A<U>,U>(x:T){g(x)}fn main(){}",
        true,
    ),
    (
        "inferred-const-from-array",
        "trait A<const N:usize>{}fn g<const N:usize,T:A<N>>(x:T)->[u8;N]{[7;N]}fn f<T:A<2>>(x:T){let _: [u8;2]=g(x);}fn main(){}",
        true,
    ),
    (
        "duplicate-lifetimes-no-call",
        "trait A<'a>{}fn f<'a,'b,T:A<'a>+A<'b>>(x:T){}fn main(){}",
        false,
    ),
    (
        "duplicate-lifetimes-call",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a,'b,T:A<'a>+A<'b>>(x:T){g(x)}fn main(){}",
        false,
    ),
    (
        "equal-lifetimes-call",
        "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a:'b,'b:'a,T:A<'a>+A<'b>>(x:T){g(x)}fn main(){}",
        false,
    ),
    (
        "rigid-type-overlap",
        "trait A<U>{}fn f<T,S:A<T>+A<u8>>(x:S){}fn main(){}",
        true,
    ),
    (
        "rigid-const-overlap",
        "trait A<const N:usize>{}fn f<const N:usize,S:A<N>+A<2>>(x:S){}fn main(){}",
        true,
    ),
    (
        "reference-subject-overlap",
        "trait A{}fn f<'a,'b,T>()where &'a T:A,&'b T:A{}fn main(){}",
        false,
    ),
    (
        "duplicate-lifetime-super",
        "trait A<'a>{}trait B<'a,'b>:A<'a>+A<'b>{}fn main(){}",
        false,
    ),
    (
        "duplicate-exact-lifetime",
        "trait A<'a>{}fn f<'a,T:A<'a>+A<'a>>(x:T){}fn main(){}",
        true,
    ),
    (
        "distinct-types",
        "trait A<U>{}fn f<T:A<u8>+A<u16>>(x:T){}fn main(){}",
        true,
    ),
    (
        "different-borrow-mutability",
        "trait A{}fn f<'a,T>()where &'a T:A,&'a mut T:A{}fn main(){}",
        false,
    ),
    (
        "different-borrow-mutability-with-outlives",
        "trait A{}fn f<'a,T:'a>()where &'a T:A,&'a mut T:A{}fn main(){}",
        true,
    ),
    (
        "partial-self-tuple",
        "trait A{}fn g<T>()where(T,):A{}fn f<S>()where(S,):A{g()}fn main(){}",
        true,
    ),
    (
        "partial-self-array",
        "trait A{}fn g<T>()where[T;2]:A{}fn f<S>()where[S;2]:A{g()}fn main(){}",
        true,
    ),
    (
        "partial-self-reference",
        "trait A{}fn g<'a,T:'a>()where &'a T:A{}fn f<'a,S:'a>()where &'a S:A{g()}fn main(){}",
        true,
    ),
    (
        "partial-self-nominal",
        "trait A{}struct W<T>(T);fn g<T>()where W<T>:A{}fn f<S>()where W<S>:A{g()}fn main(){}",
        true,
    ),
    (
        "partial-self-repeated-cross-root",
        "trait A<U>{}fn g<T,U>(x:T)where T:A<(U,U)>{}fn f<T>(x:T)where T:A<(u8,u16)>{g(x)}fn main(){}",
        false,
    ),
    (
        "partial-self-partial-ambiguity",
        "trait A{}struct W<T>(T);fn g<T>()where W<T>:A{}fn f<S,U>()where W<S>:A,W<U>:A{g()}fn main(){}",
        false,
    ),
    (
        "impl-qualified-bound",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T:A>(x:S<T>){S::<T>::run(x)}fn main(){}",
        true,
    ),
    (
        "method-own-bound",
        "trait A{}struct S<T>(T);impl<T>S<T>{fn run(self)where T:A{}}fn f<T:A>(x:S<T>){x.run()}fn main(){}",
        true,
    ),
    (
        "method-missing-bound",
        "trait A{}struct S<T>(T);impl<T>S<T>{fn run(self)where T:A{}}fn f<T>(x:S<T>){x.run()}fn main(){}",
        false,
    ),
    (
        "bound-only-argument",
        "trait A<U>{}struct S<T>(T);impl<T>S<T>{fn run<U>(self)where T:A<U>{}}fn f<T:A<u8>>(x:S<T>){x.run()}fn main(){}",
        true,
    ),
];
#[test]
fn inferred_trait_arguments_preserve_actual_caller_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    super::cases(CASES)
}
