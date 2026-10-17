//! Declaration bodies retain their inference graph while selected nominal fields become ready.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

const CASES: &[(&str, &str, bool)] = &[
    (
        "v1-trait-value",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{const C:V;}impl A for u8{const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-trait-reference",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{const C:&'static V;}impl A for u8{const C:&'static V=&V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-trait-reference-twice",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{const C:&'static &'static V;}impl A for u8{const C:&'static &'static V=&&V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-inherent-value",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{S::C.0 as usize}>{X([7;3])}struct S;impl S{const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-inherent-reference",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{S::C.0 as usize}>{X([7;3])}struct S;impl S{const C:&'static V=&V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-reference-cycle",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.n as usize}>{X([7;3])}struct R{next:OptionR,n:u8}enum OptionR{None,Some(&'static R)}trait A{const C:&'static R;}impl A for u8{const C:&'static R=&R{next:OptionR::None,n:3};}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-wrong-field",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.1 as usize}>{X([7;3])}trait A{const C:V;}impl A for u8{const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v1-wrong-type",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{const C:V;}impl A for u8{const C:u8=3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v2-nested-constant-during-fields",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.n}>{X([7;3])}trait B{const N:usize;}impl B for u16{const N:usize=2;}struct W{n:usize,v:X<{<u16 as B>::N}>}trait A{const C:&'static W;}impl A for u8{const C:&'static W=&W{n:3,v:X([5;2])};}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-array-of-reference",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C[0].0 as usize}>{X([7;3])}trait A{const C:[&'static V;1];}impl A for u8{const C:[&'static V;1]=[&V(3)];}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-ordinary-reference-constant",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{C.0 as usize}>{X([7;3])}const C:&V=&V(3);fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-projected-nominal-late",
        "struct X<const N:usize>([u8;N]);struct V(u8);trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(3);}fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-actual-referent-cycle",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C.n}>{X([7;3])}struct S{a:X<{<u8 as A>::C.n}>,n:usize}trait A{const C:&'static S;}impl A for u8{const C:&'static S=&S{a:X([7;3]),n:3};}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "initializer-finite-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}trait A{const C:usize;}trait B{const C:&'static V;}struct V{n:usize,pad:X<3>}impl A for u8{const C:usize=<u16 as B>::C.n;}impl B for u16{const C:&'static V=&V{n:3,pad:X([7;3])};}fn main(){check(f().0[0]==7);}",
        true,
    ),
    (
        "initializer-finite-late",
        "struct X<const N:usize>([u8;N]);trait A{const C:usize;}trait B{const C:&'static V;}struct V{n:usize,pad:X<3>}impl A for u8{const C:usize=<u16 as B>::C.n;}impl B for u16{const C:&'static V=&V{n:3,pad:X([7;3])};}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[0]==7);}",
        true,
    ),
    (
        "initializer-cycle-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}trait A{const C:usize;}trait B{const C:&'static V;}struct V{n:usize,pad:X<{<u8 as A>::C}>}impl A for u8{const C:usize=<u16 as B>::C.n;}impl B for u16{const C:&'static V=&V{n:3,pad:X([7;3])};}fn main(){check(f().0[0]==7);}",
        false,
    ),
    (
        "initializer-cycle-late",
        "struct X<const N:usize>([u8;N]);trait A{const C:usize;}trait B{const C:&'static V;}struct V{n:usize,pad:X<{<u8 as A>::C}>}impl A for u8{const C:usize=<u16 as B>::C.n;}impl B for u16{const C:&'static V=&V{n:3,pad:X([7;3])};}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[0]==7);}",
        false,
    ),
    (
        "formation-unused-referent-fields",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C.1}>{X([7;3])}enum O<T>{None,Some(T)}struct S{a:X<{<u8 as A>::C.1}>}trait A{const C:(O<&'static S>,usize);}impl A for u8{const C:(O<&'static S>,usize)=(O::None,3);}fn main(){check(f().0[2]==7);}\n",
        true,
    ),
    (
        "formation-paid-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.n as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const D:&'b u8;}impl<'b>B<'b>for R<'b>{const D:&'static u8=&3;}struct V<'a>{n:&'a u8,pad:X<{*<R<'static>as B<'static>>::D as usize}>}trait A<'a>{const C:&'a V<'a>;}impl<'a>A<'a>for S<'a>{const C:&'static V<'static>=&V{n:&3,pad:X([0;3])};}fn main(){check(f().0[0]==7);}\n",
        true,
    ),
    (
        "formation-paid-late",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const D:&'b u8;}impl<'b>B<'b>for R<'b>{const D:&'static u8=&3;}struct V<'a>{n:&'a u8,pad:X<{*<R<'static>as B<'static>>::D as usize}>}trait A<'a>{const C:&'a V<'a>;}impl<'a>A<'a>for S<'a>{const C:&'static V<'static>=&V{n:&3,pad:X([0;3])};}fn f()->X<{*<S<'static>as A<'static>>::C.n as usize}>{X([7;3])}fn main(){check(f().0[0]==7);}\n",
        true,
    ),
    (
        "formation-unpaid-early",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.n as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const D:&'b u8;}impl<'b>B<'b>for R<'b>{const D:&'static u8=&3;}struct V<'a>{n:&'a u8,pad:X<{*<R<'static>as B<'static>>::D as usize}>}trait A<'a>{const C:&'static V<'static>;}impl<'a>A<'a>for S<'a>{const C:&'a V<'a>=&V{n:&3,pad:X([0;3])};}fn main(){check(f().0[0]==7);}\n",
        false,
    ),
    (
        "formation-unpaid-late",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const D:&'b u8;}impl<'b>B<'b>for R<'b>{const D:&'static u8=&3;}struct V<'a>{n:&'a u8,pad:X<{*<R<'static>as B<'static>>::D as usize}>}trait A<'a>{const C:&'static V<'static>;}impl<'a>A<'a>for S<'a>{const C:&'a V<'a>=&V{n:&3,pad:X([0;3])};}fn f()->X<{*<S<'static>as A<'static>>::C.n as usize}>{X([7;3])}fn main(){check(f().0[0]==7);}\n",
        false,
    ),
];

#[test]
fn nominal_dependencies_preserve_selection_and_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("value-readiness-{name}"), &source, accepted)?;
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

fn program(family: &str, count: usize) -> Result<String, std::fmt::Error> {
    let depth = if family == "unrelated" { 8 } else { count };
    let expression = if family == "fields" {
        (0..depth)
            .map(|index| format!("<u8 as A{index}>::C.n"))
            .collect::<Vec<_>>()
            .join("+")
    } else {
        "<u8 as A0>::C.n".into()
    };
    let value = if family == "fields" { 3 * depth } else { 3 };
    let mut source =
        format!("struct X<const N:usize>([u8;N]);fn f()->X<{{{expression}}}>{{X([7;{value}])}}");
    for index in 0..depth {
        let (field, initializer) = if family == "nested" && index + 1 < depth {
            (
                format!("pad:X<{{<u8 as A{}>::C.n}}>,", index + 1),
                "pad:X([7;3]),",
            )
        } else {
            (String::new(), "")
        };
        write!(
            source,
            "struct V{index}{{n:usize,{field}}}trait A{index}{{const C:&'static V{index};}}impl A{index} for u8{{const C:&'static V{index}=&V{index}{{n:3,{initializer}}};}}"
        )?;
    }
    if family == "unrelated" {
        for index in 0..count * 32 {
            write!(source, "struct U{index}(u8,u16,usize);")?;
        }
    }
    source.push_str("fn main(){check(f().0[0]==7);}");
    Ok(source)
}

#[test]
fn suspended_body_work_and_storage_follow_demanded_nodes() -> Result<(), Box<dyn std::error::Error>>
{
    for family in ["fields", "nested", "unrelated"] {
        let mut prior = None;
        for count in [8, 24, 72] {
            let source = format!("{}{body}", native::CHECK, body = program(family, count)?);
            let (report, facts) = semantic::analyze(&source)?;
            let nodes: usize = report
                .package
                .units
                .iter()
                .map(|u| u.configured.ast.nodes.len())
                .sum();
            let work = &facts.body_work;
            println!(
                "{family}-{count}: nodes={nodes} steps={} suspensions={} peak_slots={}",
                work.steps, work.suspensions, work.peak_suspended_slots,
            );
            assert_eq!(work.suspended_slots, 0);
            assert!(
                work.steps < 4 * nodes,
                "{family}-{count}: {} steps for {nodes} nodes",
                work.steps
            );
            let needed = if family == "unrelated" { 1 } else { count };
            assert!(
                work.suspensions >= needed,
                "{family}-{count}: {} suspensions",
                work.suspensions
            );
            assert!(
                work.peak_suspended_slots < 128 * needed,
                "{family}-{count}: {} slots",
                work.peak_suspended_slots
            );
            if let Some((old_count, old_steps, old_slots)) = prior {
                if family == "unrelated" {
                    assert_eq!(work.peak_suspended_slots, old_slots);
                } else {
                    assert!(work.steps <= old_steps * count / old_count + 64);
                    assert!(work.peak_suspended_slots <= old_slots * count / old_count + 128);
                }
            }
            prior = Some((count, work.steps, work.peak_suspended_slots));
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn suspended_bodies_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, body, _)| (name, body.to_owned()))
        .collect::<Vec<_>>();
    for family in ["fields", "nested", "unrelated"] {
        cases.push((family, program(family, 8)?));
    }
    let borrowed = cases
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("value-readiness", &borrowed)
}
