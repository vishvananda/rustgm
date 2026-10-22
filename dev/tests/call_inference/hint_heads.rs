//! Optional argument/result hints cannot force an unresolved associated projection.
use crate::{native_references as native, qualify as rustc, semantic};
const LATER: &str = "trait C{type In;fn get(x:Self::In,s:Self)->u16;}impl C for u8{type In=u16;fn get(x:u16,s:u8)->u16{x+s as u16}}fn main(){let _=<_ as C>::get(7u16,3u8);}";
const WRONG: &str = "trait C{type In;fn get(x:Self::In,s:Self)->u16;}impl C for u8{type In=u16;fn get(x:u16,s:u8)->u16{x+s as u16}}fn main(){let _=<_ as C>::get(7u32,3u8);}";
const REFERENCES: &[(&str, bool, &str)] = &[
    (
        "mut-later-value",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let a=<_ as C>::take(r,*r);let b=*r;check(a==6&&b==6);}",
    ),
    (
        "explicit-mut-later-value",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let _=<_ as C>::take(&mut *r,*r);}",
    ),
    (
        "block-later-value",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let a=<_ as C>::take({r},1u8);let b=*r;check(a==4&&b==4);}",
    ),
    (
        "typed-block-later-value",
        true,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let a=<_ as C>::take({let s:&mut u8=r;s},1u8);let b=*r;check(a==4&&b==4);}",
    ),
    (
        "if-tail-later-value",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let a=<_ as C>::take(if true{r}else{r},1u8);let b=*r;check(a==4&&b==4);}",
    ),
    (
        "unknown-shared-from-mut",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a u8;fn take(x:&'a u8,s:u8)->u8{*x+s}}fn main(){let mut n=3;let r=&mut n;let _=<_ as C>::take(r,1u8);}",
    ),
    (
        "known-shared-from-mut",
        true,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a u8;fn take(x:&'a u8,s:u8)->u8{*x+s}}fn main(){let mut n=3;let r=&mut n;let a=<u8 as C>::take(r,1u8);let b=*r;check(a==4&&b==3);}",
    ),
    (
        "shared-later-write",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a u8;fn take(x:&'a u8,s:u8)->u8{*x+s}}fn main(){let mut n=3;let r=&mut n;let _=<_ as C>::take(&*r,{*r=4;1u8});}",
    ),
    (
        "unknown-mut-later-write",
        false,
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait C<'a>{type In;fn take(x:Self::In,s:Self)->u8;}impl<'a>C<'a>for u8{type In=&'a mut u8;fn take(x:&'a mut u8,s:u8)->u8{*x+=s;*x}}fn main(){let mut n=3;let r=&mut n;let _=<_ as C>::take(r,{*r=4;1u8});}",
    ),
];
#[test]
fn projected_argument_keeps_its_equation_until_later_self_is_known()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in [("later-self", LATER, true), ("wrong-element", WRONG, false)] {
        rustc::qualify(&format!("prefix-projection-{name}"), source, accepted)?;
        let result = checked(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
fn projected_reference_inputs_preserve_source_moves_and_known_reborrows()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in REFERENCES {
        rustc::qualify(&format!("prefix-reference-{name}"), source, accepted)?;
        let result = checked(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn deferred_projection_calls_execute_with_selected_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cases = vec![
        (
            "argument",
            "trait C{type In;fn get(x:Self::In,s:Self)->u16;}impl C for u8{type In=u16;fn get(x:u16,s:u8)->u16{x+s as u16}}fn main(){check(<_ as C>::get(7u16,3u8)==10);}",
            false,
        ),
        (
            "result",
            "trait C{type Out;fn get(x:Self)->Self::Out;}impl C for u8{type Out=u16;fn get(x:u8)->u16{x as u16}}fn main(){let r:u16=<_ as C>::get(73u8);check(r==73);}",
            false,
        ),
    ];
    for &(name, accepted, source) in REFERENCES {
        if accepted {
            cases.push((
                name,
                source
                    .strip_prefix(native::CHECK.trim_end())
                    .ok_or("missing native check prefix")?,
                false,
            ));
        }
    }
    native::check_cases("parallel/prefix-projection/native", &cases)
}

fn checked(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
