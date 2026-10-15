//! Impl bounds filter receiver candidates; selected calls retain complete evidence.
const CASES: &[(&str, &str, bool)] = &[
    (
        "unknown-subject-from-argument",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self,x:T){}}fn f<T:A>(x:T){let s:S<_>=loop{};s.run(x)}fn main(){}",
        true,
    ),
    (
        "unresolved-subject",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T:A>(){let s:S<_>=loop{};s.run()}fn main(){}",
        false,
    ),
    (
        "inferred-prefix-argument",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self){}}fn f<T:A<u8>>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "ambiguous-prefix-argument",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self){}}fn f<T:A<u8>+A<u16>>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        false,
    ),
    (
        "later-call-argument",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self,x:U){}}fn f<T:A<u8>+A<u16>>(){let s:S<T,_>=loop{};s.run(7u8)}fn main(){}",
        true,
    ),
    (
        "later-result",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self)->U{self.1}}fn f<T:A<u8>+A<u16>>(){let s:S<T,_>=loop{};let _:u8=s.run();}fn main(){}",
        true,
    ),
    (
        "independent-goals",
        "trait A<U>{}trait B<U>{}struct S<T,U,V>(T,U,V);impl<T:A<U>,U,V:B<U>>S<T,U,V>{fn run(self){}}fn f<T:A<u8>+A<u16>,V:B<u8>>(){let s:S<T,_,V>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "wrong-impl-lifetime",
        "trait A<'a>{}struct S<'a,T>(&'a T);impl<'a,T:A<'a>>S<'a,T>{fn run(self){}}fn f<'a,'b,T:A<'b>>(x:S<'a,T>){x.run()}fn main(){}",
        false,
    ),
    (
        "equivalent-impl-lifetime",
        "trait A<'a>{}struct S<'a,T>(&'a T);impl<'a,T:A<'a>>S<'a,T>{fn run(self){}}fn f<'a:'b,'b:'a,T:A<'b>>(x:S<'a,T>){x.run()}fn main(){}",
        true,
    ),
    (
        "missing-impl-bound",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T>(x:S<T>){x.run()}fn main(){}",
        false,
    ),
    (
        "disjoint-headers",
        "trait A{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:A>S<T,u16>{fn run(self){}}fn f<T:A>(x:S<T,u8>){x.run()}fn main(){}",
        true,
    ),
    (
        "overlap-headers",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}impl<T>S<T>{fn run(self){}}fn main(){}",
        false,
    ),
    (
        "impl-dot-bound",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T:A>(x:S<T>){x.run()}fn main(){}",
        true,
    ),
    (
        "impl-qualified-bound",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T:A>(x:S<T>){S::<T>::run(x)}fn main(){}",
        true,
    ),
    (
        "impl-missing-bound",
        "trait A{}struct S<T>(T);impl<T:A>S<T>{fn run(self){}}fn f<T>(x:S<T>){x.run()}fn main(){}",
        false,
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
        "impl-type-argument",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self){}}fn f<T:A<u8>>(x:S<T,u8>){x.run()}fn main(){}",
        true,
    ),
    (
        "impl-type-argument-wrong",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<U>,U>S<T,U>{fn run(self){}}fn f<T:A<u16>>(x:S<T,u8>){x.run()}fn main(){}",
        false,
    ),
    (
        "impl-lifetime",
        "trait A<'a>{}struct S<'a,T>(&'a T);impl<'a,T:A<'a>>S<'a,T>{fn run(self){}}fn f<'a,T:A<'a>>(x:S<'a,T>){x.run()}fn main(){}",
        true,
    ),
    (
        "impl-lifetime-wrong",
        "trait A<'a>{}struct S<'a,T>(&'a T);impl<'a,T:A<'a>>S<'a,T>{fn run(self){}}fn f<'a,'b,T:A<'b>>(x:S<'a,T>){x.run()}fn main(){}",
        false,
    ),
    (
        "bound-only-argument",
        "trait A<U>{}struct S<T>(T);impl<T>S<T>{fn run<U>(self)where T:A<U>{}}fn f<T:A<u8>>(x:S<T>){x.run()}fn main(){}",
        true,
    ),
    (
        "impl-bounds-before-uniqueness",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T:A>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "qualified-impl-bounds",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T:A>(){S::<T,_>::run(loop{})}fn main(){}",
        true,
    ),
    (
        "both-impl-bounds",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T:A+B>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        false,
    ),
    (
        "neither-impl-bound",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        false,
    ),
    (
        "method-own-after-uniqueness",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T>S<T,u8>{fn run(self)where T:A{}}impl<T>S<T,u16>{fn run(self)where T:B{}}fn f<T:A>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        false,
    ),
    (
        "impl-trait-argument-filter",
        "trait A<U>{}struct S<T,U>(T,U);impl<T:A<u8>>S<T,u8>{fn run(self){}}impl<T:A<u16>>S<T,u16>{fn run(self){}}fn f<T:A<u8>>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "const-bound-prefix-inference",
        "trait A<const N:usize>{}struct S<T,const N:usize>(T);impl<T:A<N>,const N:usize>S<T,N>{fn run(self){}}fn f<T:A<2>>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "generic-lifetime-value",
        "struct S<'a,T>(&'a T);impl<'a,T>S<'a,T>{fn run(self){}}fn f<'a,T>(x:S<'a,T>){x.run()}fn main(){}",
        true,
    ),
    (
        "generic-copy-dereference",
        "#[derive(Copy,Clone)]struct S<'a,T>(&'a T);impl<'a,T>S<'a,T>{fn run(self){}}fn f<'a,T:Copy>(x:&S<'a,T>){x.run()}fn main(){}",
        true,
    ),
    (
        "generic-move-dereference",
        "struct S<'a,T>(&'a T);impl<'a,T>S<'a,T>{fn run(self){}}fn f<'a,T>(x:&S<'a,T>){x.run()}fn main(){}",
        false,
    ),
    (
        "failed-first-impl",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T:B>(){let s:S<T,_>=loop{};s.run()}fn main(){}",
        true,
    ),
    (
        "failed-first-qualified",
        "trait A{}trait B{}struct S<T,U>(T,U);impl<T:A>S<T,u8>{fn run(self){}}impl<T:B>S<T,u16>{fn run(self){}}fn f<T:B>(){S::<T,_>::run(loop{})}fn main(){}",
        true,
    ),
    (
        "builtin-after-trait-inference",
        "trait A<U>{}struct N;struct S<T,U,V>(T,U,V);impl<T:A<U>,U:Copy>S<T,U,u8>{fn run(self){}}impl<T:A<U>,U>S<T,U,u16>{fn run(self){}}fn f<T:A<N>>(){let s:S<T,_,_>=loop{};s.run()}fn main(){}",
        true,
    ),
];
#[test]
fn caller_bounds_select_impl_methods_without_guessing_or_erasing_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    super::cases(CASES)
}
