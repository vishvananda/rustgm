//! A closure inherits lexical unsafe permission; nested item bodies reset it.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "block",
        "fn main(){let f=unsafe{|p:*const u8|*p};let x=7u8;let _=f(&raw const x);}",
        true,
    ),
    (
        "function",
        "unsafe fn run(){let _f=|p:*const u8|*p;}fn main(){}",
        true,
    ),
    (
        "nested",
        "fn main(){let f=unsafe{||{let g=|p:*const u8|*p;let x=7u8;g(&raw const x)}};let _=f();}",
        true,
    ),
    ("safe", "fn main(){let _f=|p:*const u8|*p;}", false),
    (
        "sibling",
        "fn main(){let _f=unsafe{||1};let _g=|p:*const u8|*p;}",
        false,
    ),
    (
        "item",
        "fn main(){let _f=unsafe{||{fn read(p:*const u8)->u8{*p}0u8}};}",
        false,
    ),
];

#[test]
fn inherited_permissions_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, expected) in CASES {
        rustc::qualify(&format!("closure-safety-{name}"), source, expected)?;
        let actual = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        assert_eq!(actual.is_ok(), expected, "{name}: {:?}", actual.err());
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn inherited_permissions_preserve_native_calls() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa24-source-bodies/safety",
        &[
            (
                "block",
                "fn main(){let f=unsafe{|p:*const u8|*p};let x=7u8;check(f(&raw const x)==7);}",
                false,
            ),
            (
                "nested",
                "fn main(){let f=unsafe{||{let g=|p:*const u8|*p;let x=7u8;g(&raw const x)}};check(f()==7);}",
                false,
            ),
        ],
    )
}
