//! Early member shapes retain their lifetime contract until nominal facts are complete.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "equal-reference-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}struct S<'a>(&'a u8);trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for S<'a>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "equal-reference-late",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for S<'a>{const C:&'a u8=&3;}fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "stronger-reference-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}struct S<'a>(&'a u8);trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for S<'a>{const C:&'static u8=&3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "stronger-reference-late",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for S<'a>{const C:&'static u8=&3;}fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "shorter-reference-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}struct S<'a>(&'a u8);trait A<'a>{const C:&'static u8;}impl<'a>A<'a>for S<'a>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "shorter-reference-late",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);trait A<'a>{const C:&'static u8;}impl<'a>A<'a>for S<'a>{const C:&'a u8=&3;}fn f()->X<{*<S<'static>as A<'static>>::C as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "outlives",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static,'static>as A<'static,'static>>::C as usize}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait A<'a,'b>{const C:&'b u8;}impl<'a:'b,'b>A<'a,'b>for S<'a,'b>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "unrelated",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static,'static>as A<'static,'static>>::C as usize}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait A<'a,'b>{const C:&'b u8;}impl<'a,'b>A<'a,'b>for S<'a,'b>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "reversed",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static,'static>as A<'static,'static>>::C as usize}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait A<'a,'b>{const C:&'b u8;}impl<'a,'b:'a>A<'a,'b>for S<'a,'b>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "equal-regions",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static,'static>as A<'static,'static>>::C as usize}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait A<'a,'b>{const C:&'b u8;}impl<'a:'b,'b:'a>A<'a,'b>for S<'a,'b>{const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "projected-debt",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<S<'static,'static>as A<'static,'static>>::C}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait P<'b>{type Out;}impl<'a,'b>P<'b>for &'a u8 where 'a:'b{type Out=usize;}trait A<'a,'b>{const C:usize;}impl<'a,'b>A<'a,'b>for S<'a,'b> where 'a:'b{const C:<&'a u8 as P<'b>>::Out=3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "projected-unpaid",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<S<'static,'static>as A<'static,'static>>::C}>{X([7;3])}struct S<'a,'b>(&'a u8,&'b u8);trait P<'b>{type Out;}impl<'a,'b>P<'b>for &'a u8 where 'a:'b{type Out=usize;}trait A<'a,'b>{const C:usize;}impl<'a,'b>A<'a,'b>for S<'a,'b>{const C:<&'a u8 as P<'b>>::Out=3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "covariant-result",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}struct S<'a>(&'a u8);struct V<'a>(&'a u8);trait A<'a>{const C:V<'a>;}impl<'a>A<'a>for S<'a>{const C:V<'static>=V(&3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "invariant-result",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<S<'static>as A<'static>>::C.n}>{X([7;3])}struct S<'a>(&'a u8);struct V<'a>{n:usize,f:fn(&'a u8)->&'a u8}fn id(x:&u8)->&u8{x}trait A<'a>{const C:V<'a>;}impl<'a>A<'a>for S<'a>{const C:V<'static>=V{n:3,f:id};}fn main(){check(f().0[2]==7);}",
        false,
    ),
];

#[test]
fn early_member_contracts_preserve_lifetime_and_variance_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("member-lifetimes-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn early_member_contracts_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("member-lifetimes", &cases)
}
