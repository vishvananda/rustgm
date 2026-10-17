//! Associated result equations are available only with complete parent formation evidence.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "member-nested-equality-1",
        "trait A{type Out:B;}trait B:C<Out=u8>{}trait C{type Out;}impl A for u8{type Out=u16;}impl B for u16{}impl C for u16{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as C>::Out)->u8{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "member-nested-equality-2",
        "trait A{type Out:B;}trait B{type Out:C<Out=u8>;}trait C{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}impl C for u32{type Out=u8;}fn use_it<T:A>(x:<<<T as A>::Out as B>::Out as C>::Out)->u8{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "header-nested-equality-1",
        "trait A where Self::Out:B{type Out;}trait B:C<Out=u8>{}trait C{type Out;}impl A for u8{type Out=u16;}impl B for u16{}impl C for u16{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as C>::Out)->u8{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "header-nested-equality-2",
        "trait A where Self::Out:B{type Out;}trait B where Self::Out:C<Out=u8>{type Out;}trait C{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}impl C for u32{type Out=u8;}fn use_it<T:A>(x:<<<T as A>::Out as B>::Out as C>::Out)->u8{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "direct-member-binding",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}fn value<T:A>(x:<<T as A>::Out as B>::Out)->u8{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "redundant-explicit-binding",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}fn value<T:A>(x:<<T as A>::Out as B>::Out)->u8 where T::Out:B<Out=u8>{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "aliased-result-subject",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}type Output<T>=<T as A>::Out;fn value<T:A>(x:<Output<T> as B>::Out)->u8{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "equalities-not-inputs",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}fn wrong<T:A,U:A>(x:T)->U{x}fn main(){}",
        false,
    ),
    (
        "wrong-member-value",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}fn wrong<T:A>(x:<<T as A>::Out as B>::Out)->u16{x}fn main(){}",
        false,
    ),
    (
        "invalid-unused-definition",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn main(){}",
        false,
    ),
    (
        "missing-root-parent",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}fn wrong<T>(x:<<T as A>::Out as B>::Out)->u8{x}fn main(){}",
        false,
    ),
    (
        "unrelated-member-name",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}trait Other{type Out;}fn wrong<T:A>(x:<<T as A>::Out as Other>::Out)->u8 where T::Out:Other{x}fn main(){}",
        false,
    ),
    (
        "lifetime-result",
        "trait A<'a>{type Out:B<'a,Out=&'a u8>;}trait B<'a>{type Out:'a;}impl<'a>A<'a>for u8{type Out=u16;}impl<'a>B<'a>for u16{type Out=&'a u8;}fn value<'a,T:A<'a>>(x:<<T as A<'a>>::Out as B<'a>>::Out)->&'a u8{x}fn main(){let x=7u8;check(*value::<u8>(&x)==7);}",
        true,
    ),
    (
        "lifetime-escape",
        "trait A<'a>{type Out:B<'a,Out=&'a u8>;}trait B<'a>{type Out:'a;}impl<'a>A<'a>for u8{type Out=u16;}impl<'a>B<'a>for u16{type Out=&'a u8;}fn wrong<'a,T:A<'a>>(x:<<T as A<'a>>::Out as B<'a>>::Out)->&'static u8{x}fn main(){}",
        false,
    ),
    (
        "distinct-trait-arguments",
        "trait B<X>{type Out;}trait A{type Out:B<u8,Out=u32>+B<u16,Out=u64>;}impl B<u8>for u16{type Out=u32;}impl B<u16>for u16{type Out=u64;}impl A for u8{type Out=u16;}fn value<T:A>(x:<<T as A>::Out as B<u8>>::Out,y:<<T as A>::Out as B<u16>>::Out)->(u32,u64){(x,y)}fn main(){let p=value::<u8>(7,9);check(p.0==7&&p.1==9);}",
        true,
    ),
    (
        "wrong-trait-argument",
        "trait B<X>{type Out;}trait A{type Out:B<u8,Out=u32>+B<u16,Out=u64>;}impl B<u8>for u16{type Out=u32;}impl B<u16>for u16{type Out=u64;}impl A for u8{type Out=u16;}fn wrong<T:A>(x:<<T as A>::Out as B<u32>>::Out)->u32 where T::Out:B<u32>{x}fn main(){}",
        false,
    ),
    (
        "const-trait-argument",
        "trait B<const N:usize>{type Out;}trait A{type Out:B<2,Out=[u8;2]>;}impl B<2>for u16{type Out=[u8;2];}impl A for u8{type Out=u16;}fn value<T:A>(x:<<T as A>::Out as B<2>>::Out)->[u8;2]{x}fn main(){let p=value::<u8>([7,9]);check(p[0]==7&&p[1]==9);}",
        true,
    ),
    (
        "default-method-body",
        "trait A{type Out:B<Out=u8>;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}trait Use:A{fn take(x:<<Self as A>::Out as B>::Out)->u8{x}}impl Use for u8{}fn main(){check(<u8 as Use>::take(7)==7);}",
        true,
    ),
    (
        "direct-header-binding",
        "trait A where Self::Out:B<Out=u8>{type Out;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u8;}fn value<T:A>(x:<<T as A>::Out as B>::Out)->u8{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "header-invalid-unused-definition",
        "trait A where Self::Out:B<Out=u8>{type Out;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn main(){}",
        false,
    ),
    (
        "inherited-header-without-requirement",
        "trait C{type Out;}trait B{type Out:C;}trait A:B where Self::Out:C<Out=u8>{}impl C for u16{type Out=u8;}impl B for u8{type Out=u16;}impl A for u8{}fn value<T:A>(x:<<T as B>::Out as C>::Out)->u8{x}fn main(){check(value::<u8>(7)==7);}",
        false,
    ),
    (
        "inherited-header-explicit",
        "trait C{type Out;}trait B{type Out:C;}trait A:B where Self::Out:C<Out=u8>{}impl C for u16{type Out=u8;}impl B for u8{type Out=u16;}impl A for u8{}fn value<T:A>(x:<<T as B>::Out as C>::Out)->u8 where T::Out:C<Out=u8>{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "recursive-member-equality",
        "trait A{type Out:A<Out=Self::Out>;}impl A for u8{type Out=u16;}impl A for u16{type Out=u16;}fn value<T:A>(x:<<T as A>::Out as A>::Out)->T::Out{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "recursive-header-equality",
        "trait A where Self::Out:A<Out=Self::Out>{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u16;}fn value<T:A>(x:<<T as A>::Out as A>::Out)->T::Out{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "diamond-equality",
        "trait C{type Out;}trait Left:C<Out=u8>{}trait Right:C<Out=u8>{}trait B:Left+Right{}trait A{type Out:B;}impl C for u16{type Out=u8;}impl Left for u16{}impl Right for u16{}impl B for u16{}impl A for u8{type Out=u16;}fn value<T:A>(x:<<T as A>::Out as C>::Out)->u8{x}fn main(){check(value::<u8>(7)==7);}",
        true,
    ),
    (
        "diamond-wrong-value",
        "trait C{type Out;}trait Left:C<Out=u8>{}trait Right:C<Out=u8>{}trait B:Left+Right{}trait A{type Out:B;}impl C for u16{type Out=u8;}impl Left for u16{}impl Right for u16{}impl B for u16{}impl A for u8{type Out=u16;}fn wrong<T:A>(x:<<T as A>::Out as C>::Out)->u16{x}fn main(){}",
        false,
    ),
    (
        "member-missing-parent",
        "trait B{type Out;}trait A<T>{type Out:B<Out=<T as B>::Out>;}fn main(){}",
        false,
    ),
    (
        "member-formed",
        "trait B{type Out;}trait A<T:B>{type Out:B<Out=<T as B>::Out>;}impl B for u8{type Out=u16;}impl B for u32{type Out=u16;}impl A<u8>for u64{type Out=u32;}fn value<T:B,X:A<T>>(x:<<X as A<T>>::Out as B>::Out)->T::Out{x}fn main(){check(value::<u8,u64>(7)==7);}",
        true,
    ),
    (
        "header-missing-parent",
        "trait B{type Out;}trait A<T>where Self::Out:B<Out=<T as B>::Out>{type Out;}fn main(){}",
        false,
    ),
    (
        "header-formed",
        "trait B{type Out;}trait A<T:B>where Self::Out:B<Out=<T as B>::Out>{type Out;}impl B for u8{type Out=u16;}impl B for u32{type Out=u16;}impl A<u8>for u64{type Out=u32;}fn value<T:B,X:A<T>>(x:<<X as A<T>>::Out as B>::Out)->T::Out{x}fn main(){check(value::<u8,u64>(7)==7);}",
        true,
    ),
    (
        "member-noninjective",
        "trait A{type Out:A<Out=Self::Out>;}fn wrong<T:A>(x:T)->T::Out{x}fn main(){}",
        false,
    ),
    (
        "header-noninjective",
        "trait A where Self::Out:A<Out=Self::Out>{type Out;}fn wrong<T:A>(x:T)->T::Out{x}fn main(){}",
        false,
    ),
];

#[test]
fn associated_result_equalities_match_independent_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("result-equality-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_result_equalities_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("result-equality-projections", &cases)
}
