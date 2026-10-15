//! A bound-selected name retains its complete application through callable inference.
use super::cases;

#[test]
fn bound_applications_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "bound-argument-dot-priority",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u16{x.f(7u16)}fn main(){}",
            false,
        ),
        (
            "bound-argument-path-priority",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u16{T::f(x,7u16)}fn main(){}",
            false,
        ),
        (
            "bound-argument-explicit-trait",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u16{<T as A<u16>>::f(x,7u16)}fn main(){}",
            true,
        ),
        (
            "bound-argument-two-bounds",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>+A<u16>>(x:&T)->u16{x.f(7u16)}fn main(){}",
            true,
        ),
        (
            "bound-argument-matching-dot",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u8{x.f(7u8)}fn main(){}",
            true,
        ),
        (
            "bound-argument-matching-path",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u8{T::f(x,7u8)}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn bound_applications_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "bound-argument-function-value-reject",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u16{let f=T::f;f(x,7u16)}fn main(){}",
            false,
        ),
        (
            "bound-argument-function-value-accept",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u8{let f=T::f;f(x,7u8)}fn main(){}",
            true,
        ),
        (
            "bound-argument-two-bounds-path",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>+A<u16>>(x:&T)->u16{T::f(x,7u16)}fn main(){}",
            true,
        ),
        (
            "bound-argument-supertrait-dot-reject",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}trait B:A<u8>{}fn g<T:B>(x:&T)->u16{x.f(7u16)}fn main(){}",
            false,
        ),
        (
            "bound-argument-supertrait-path-reject",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}trait B:A<u8>{}fn g<T:B>(x:&T)->u16{T::f(x,7u16)}fn main(){}",
            false,
        ),
        (
            "bound-argument-supertrait-dot-accept",
            "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}trait B:A<u8>{}fn g<T:B>(x:&T)->u8{x.f(7u8)}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn bound_applications_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "bound-const-dot-reject",
            "trait A<const N:usize>{fn f(&self,x:[u8;N])->[u8;N];}impl<T>A<2>for T{fn f(&self,x:[u8;2])->[u8;2]{x}}fn g<T:A<1>>(x:&T)->[u8;2]{x.f([7,9])}fn main(){}",
            false,
        ),
        (
            "bound-const-path-reject",
            "trait A<const N:usize>{fn f(&self,x:[u8;N])->[u8;N];}impl<T>A<2>for T{fn f(&self,x:[u8;2])->[u8;2]{x}}fn g<T:A<1>>(x:&T)->[u8;2]{T::f(x,[7,9])}fn main(){}",
            false,
        ),
        (
            "bound-const-explicit-accept",
            "trait A<const N:usize>{fn f(&self,x:[u8;N])->[u8;N];}impl<T>A<2>for T{fn f(&self,x:[u8;2])->[u8;2]{x}}fn g<T:A<1>>(x:&T)->[u8;2]{<T as A<2>>::f(x,[7,9])}fn main(){}",
            true,
        ),
        (
            "bound-const-two-bounds-accept",
            "trait A<const N:usize>{fn f(&self,x:[u8;N])->[u8;N];}impl<T>A<2>for T{fn f(&self,x:[u8;2])->[u8;2]{x}}fn g<T:A<1>+A<2>>(x:&T)->[u8;2]{x.f([7,9])}fn main(){}",
            true,
        ),
        (
            "bound-method-other-subject",
            "trait A<U>{fn f<V:A<u16>>(&self,x:U,y:&V)->U;}impl<T>A<u16>for T{fn f<V:A<u16>>(&self,x:u16,y:&V)->u16{x}}fn g<T:A<u8>>(x:&T,y:&T)->u8{x.f(7u8,y)}fn main(){}",
            true,
        ),
        (
            "bound-method-other-application",
            "trait A<U>{fn f(&self,x:U)->U where Self:A<u16>;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}fn g<T:A<u8>>(x:&T)->u8{x.f(7u8)}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn bound_applications_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "bound-lifetime-dot-reject",
            "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<T>A<'static>for T{fn f(&self,x:&'static u8)->&'static u8{x}}fn g<'a,T:A<'a>>(x:&T,y:&'a u8)->&'static u8{x.f(&7)}fn main(){}",
            false,
        ),
        (
            "bound-lifetime-path-reject",
            "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<T>A<'static>for T{fn f(&self,x:&'static u8)->&'static u8{x}}fn g<'a,T:A<'a>>(x:&T,y:&'a u8)->&'static u8{T::f(x,&7)}fn main(){}",
            false,
        ),
        (
            "bound-lifetime-dot-accept",
            "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<T>A<'static>for T{fn f(&self,x:&'static u8)->&'static u8{x}}fn g<'a,T:A<'a>>(x:&T,y:&'a u8)->&'a u8{x.f(y)}fn main(){}",
            true,
        ),
        (
            "bound-lifetime-explicit-accept",
            "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<T>A<'static>for T{fn f(&self,x:&'static u8)->&'static u8{x}}fn g<'a,T:A<'a>>(x:&T,y:&'a u8)->&'static u8{<T as A<'static>>::f(x,&7)}fn main(){}",
            false,
        ),
    ])
}

const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "bound-dot-retains-u8",
        "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x+2}}impl A<u8>for u8{fn f(&self,x:u8)->u8{x+1}}fn g<T:A<u8>>(x:&T)->u8{x.f(7)}fn main(){check(g(&3u8)==8);}",
        false,
    ),
    (
        "bound-function-value-retains-u8",
        "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x+2}}impl A<u8>for u8{fn f(&self,x:u8)->u8{x+1}}fn g<T:A<u8>>(x:&T)->u8{let f=T::f;f(x,7)}fn main(){check(g(&3u8)==8);}",
        false,
    ),
    (
        "bound-dual-application-dispatch",
        "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x+2}}impl A<u8>for u8{fn f(&self,x:u8)->u8{x+1}}fn g<T:A<u8>+A<u16>>(x:&T)->u16{x.f(7u16)}fn main(){check(g(&3u8)==9);}",
        false,
    ),
    (
        "bound-const-dispatch",
        "trait A<const N:usize>{fn f(&self,x:[u8;N])->[u8;N];}impl<T>A<2>for T{fn f(&self,x:[u8;2])->[u8;2]{x}}impl A<1>for u8{fn f(&self,x:[u8;1])->[u8;1]{x}}fn g<T:A<1>+A<2>>(x:&T)->[u8;2]{x.f([7,9])}fn main(){let x=g(&3u8);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "bound-extra-self-requirement",
        "trait A<U>{fn f(&self,x:U)->U where Self:A<u16>;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x}}impl A<u8>for u8{fn f(&self,x:u8)->u8{x+1}}fn g<T:A<u8>>(x:&T)->u8{x.f(7)}fn main(){check(g(&3u8)==8);}",
        false,
    ),
    (
        "bound-supertrait-dispatch",
        "trait A<U>{fn f(&self,x:U)->U;}impl<T>A<u16>for T{fn f(&self,x:u16)->u16{x+2}}impl A<u8>for u8{fn f(&self,x:u8)->u8{x+1}}trait B:A<u8>{}impl B for u8{}fn g<T:B>(x:&T)->u8{T::f(x,7)}fn main(){check(g(&3u8)==8);}",
        false,
    ),
];
#[test]
fn bound_calls_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        super::cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn bound_calls_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("bound-receiver-calls", EXECUTIONS)
}
