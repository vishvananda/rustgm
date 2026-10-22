//! A wildcard retains type liveness without observing initialization or consuming a value.
use crate::qualify as rustc;
use crate::semantic;

#[test]
fn wildcard_lifetimes_match_pinned_rust() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in [
        (
            "reference",
            "fn f<'a>(x:&'a u8)->&'a u8{x}fn main(){let x;{let a=7;x=f(&a);}let _=x;}",
            false,
        ),
        (
            "opaque",
            "fn f<'a>(x:&'a u8)->impl Copy{x}fn main(){let x;{let a=7;x=f(&a);}let _=x;}",
            false,
        ),
        (
            "captured-unit",
            "fn f<'a>(_:&'a u8)->impl Copy{()}fn main(){let x;{let a=7;x=f(&a);}let _=x;}",
            false,
        ),
        (
            "unused-reference",
            "fn f<'a>(x:&'a u8)->&'a u8{x}fn main(){let x;{let a=7;x=f(&a);}}",
            true,
        ),
        (
            "unused-opaque",
            "fn f<'a>(x:&'a u8)->impl Copy{x}fn main(){let x;{let a=7;x=f(&a);}}",
            true,
        ),
        ("uninitialized", "fn main(){let x:&u8;let _=x;}", true),
        (
            "moved",
            "struct S<'a>(&'a u8);fn take<T>(_:T){}fn main(){let a=7;let x=S(&a);take(x);let _=x;}",
            true,
        ),
        (
            "precise-empty",
            "fn f<'a>(_:&'a u8)->impl Copy+use<>{()}fn main(){let x;{let a=7;x=f(&a);}let _=x;}",
            true,
        ),
        (
            "static-promise",
            "fn f<'a>(_:&'a u8)->impl Copy+'static{()}fn main(){let x;{let a=7;x=f(&a);}let _=x;}",
            true,
        ),
    ] {
        rustc::qualify(&format!("wildcard-liveness-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
