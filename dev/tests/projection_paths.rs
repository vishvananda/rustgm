//! Qualified projected types are independently checked before their `LowIR` is executed.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use rustgm::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "impl-name-needs-self",
        "trait A{type Out;fn value()->u8;}impl A for u8{type Out=u8;fn value()->Out{7}}fn main(){}",
        false,
    ),
    (
        "associated-rhs-needs-self",
        "trait A{type Out;type Other;}impl A for u8{type Out=u8;type Other=Out;}fn main(){}",
        false,
    ),
    (
        "nested-ordinary-alias",
        "trait A{type Out;fn value()->u8;}impl A for u8{type Out=u16;fn value()->u8{type Out=u8;let x:Out=7;x}}fn main(){let _:u8=<u8 as A>::value();}",
        true,
    ),
    (
        "trait-name-does-not-supply-self",
        "trait A{type Out;}impl<T:?Sized>A for T{type Out=u8;}fn main(){let _: A::Out=7u8;}",
        false,
    ),
    (
        "associated-name-needs-self",
        "trait A{type Out;fn value()->Out;}impl<T:?Sized>A for T{type Out=u8;fn value()->u8{7}}fn main(){}",
        false,
    ),
    (
        "former-pa18-boundary",
        "trait Has{type Item;}struct S;impl Has for S{type Item=u8;}fn main(){let _: <S as Has>::Item=7;}\n",
        true,
    ),
    (
        "former-pa19-boundary",
        "trait A{type Out;}struct S;impl A for S{type Out=u8;}fn main(){let _: <S as A>::Out=7;}",
        true,
    ),
    (
        "static-result",
        "trait A{type Out;}impl A for u8{type Out=&'static u16;}fn f()-><u8 as A>::Out{&7}fn main(){let _: &'static u16=f();}",
        true,
    ),
    (
        "callable-result",
        "trait A{type Out;}impl A for u8{type Out=for<'a>fn(&'a u8)->&'a u8;}fn id(x:&u8)->&u8{x}fn main(){let _: <u8 as A>::Out=id;}",
        true,
    ),
    (
        "equal-results",
        "trait A{type Out;}impl A for u8{type Out=u32;}impl A for u16{type Out=u32;}fn main(){let x:<u8 as A>::Out=7;let _: <u16 as A>::Out=x;}",
        true,
    ),
    (
        "concrete",
        "trait A{type Out;}struct S;impl A for S{type Out=u8;}fn main(){let _: <S as A>::Out=7u8;}",
        true,
    ),
    (
        "signature",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()-><u8 as A>::Out{7}fn main(){let _:u16=f();}",
        true,
    ),
    (
        "generic-impl",
        "trait A{type Out;}struct S<T>(T);impl<T>A for S<T>{type Out=T;}fn main(){let _: <S<u8> as A>::Out=7u8;}",
        true,
    ),
    (
        "trait-arguments",
        "trait A<T>{type Out;}impl A<u8>for u8{type Out=u16;}impl A<u16>for u8{type Out=u32;}fn main(){let _: <u8 as A<u8>>::Out=7u16;let _: <u8 as A<u16>>::Out=9u32;}",
        true,
    ),
    (
        "const-arguments",
        "trait A<const N:usize>{type Out;}impl<const N:usize>A<N>for u8{type Out=[u8;N];}fn main(){let _: <u8 as A<2>>::Out=[7u8,9];}",
        true,
    ),
    (
        "nested-alias",
        "type P=(<u8 as A>::Out,[<u16 as A>::Out;2]);trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}fn main(){let _:P=(7u16,[8u32,9]);}",
        true,
    ),
    (
        "nested-projection",
        "trait A{type Out;}trait B{type Out;}impl A for u8{type Out=<u16 as B>::Out;}impl B for u16{type Out=u32;}fn main(){let _: <u8 as A>::Out=7u32;}",
        true,
    ),
    (
        "impl-obligation",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn main(){let _: <u16 as A>::Out=7u8;}",
        true,
    ),
    (
        "forward-alias",
        "trait A{type Out;}fn f()-><u8 as A>::Out{7}impl A for u8{type Out=Result;}type Result=u16;fn main(){let _:u16=f();}",
        true,
    ),
    (
        "trait-default",
        "trait A<T=u8>{type Out;}impl A for u8{type Out=u16;}fn main(){let _: <u8 as A>::Out=7u16;}",
        true,
    ),
    (
        "trait-alias-name",
        "trait A{type Out;}use A as B;impl A for u8{type Out=u16;}fn main(){let _: <u8 as B>::Out=7u16;}",
        true,
    ),
    (
        "unsized-result",
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u16];}fn main(){let x=[7u16,9];let _: &<u8 as A>::Out=&x;}",
        true,
    ),
    (
        "empty-arguments",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn main(){let _: <u8 as A>::Out<>=7u16;}",
        true,
    ),
    (
        "missing-impl",
        "trait A{type Out;}fn main(){let _: <u8 as A>::Out=7u8;}",
        false,
    ),
    (
        "wrong-type",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn main(){let _: <u8 as A>::Out=7u8;}",
        false,
    ),
    (
        "missing-bound",
        "struct N;trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}fn main(){let _: <N as A>::Out=7u8;}",
        false,
    ),
    (
        "cycle",
        "trait A{type Out;}impl A for u8{type Out=<u8 as A>::Out;}fn main(){}",
        false,
    ),
    (
        "mutual-cycle",
        "trait A{type Out;}impl A for u8{type Out=<u16 as A>::Out;}impl A for u16{type Out=<u8 as A>::Out;}fn main(){}",
        false,
    ),
    (
        "wrong-member",
        "trait A{fn out();}impl A for u8{fn out(){}}fn main(){let _: <u8 as A>::out=7;}",
        false,
    ),
    (
        "invalid-unused-impl",
        "trait A{type Out:Copy;}struct N;impl A for u8{type Out=u16;}impl A for u16{type Out=N;}fn main(){let _: <u8 as A>::Out=7u16;}",
        false,
    ),
];

#[test]
fn independently_qualified_paths() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("projection-path-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(report, mut facts)| {
            let main = report
                .resolution
                .definitions
                .iter()
                .position(|d| {
                    d.kind == DefKind::Function && report.package.names.get(d.name) == "main"
                })
                .ok_or("missing main")?;
            let main = DefId(u32::try_from(main).map_err(|e| e.to_string())?);
            rustgm::lower::instances::build(&report, &mut facts, main)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn normalized_types_execute_with_their_selected_layouts() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "projection-paths",
        &[
            (
                "scalar",
                "trait A{type Out;}struct S;impl A for S{type Out=u16;}fn f(x:<S as A>::Out)->u16{x+1}fn main(){check(f(299)==300)}",
                false,
            ),
            (
                "aggregate",
                "trait A<const N:usize>{type Out;}impl<T,const N:usize>A<N>for T{type Out=[T;N];}fn main(){let x:<u16 as A<2>>::Out=[299,400];check(x[0]+x[1]==699)}",
                false,
            ),
            (
                "wide",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u16];}fn main(){let x=[7u16,9];let y:&<u8 as A>::Out=&x;check(y[1]==9)}",
                false,
            ),
        ],
    )
}
