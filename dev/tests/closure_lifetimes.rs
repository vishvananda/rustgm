//! Closure input universality and inferred captured lifetimes have distinct contracts.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "annotated-read",
        "fn main(){let f=|x:&u8|*x;let a=7;let _=f(&a);}",
        true,
    ),
    (
        "annotated-return",
        "fn main(){let f=|x:&u8|x;let a=7;let _=f(&a);}",
        false,
    ),
    (
        "explicit-return",
        "fn main(){let f=|x:&u8|->&u8{x};let a=7;let _=f(&a);}",
        false,
    ),
    (
        "inferred-return",
        "fn main(){let f=|x|x;let a=7;let r=f(&a);let _=*r;}",
        true,
    ),
    (
        "annotated-higher",
        "fn apply<F:for<'a>Fn(&'a u8)->u8>(f:F,x:&u8)->u8{f(x)}fn main(){let a=7;let _=apply(|x:&u8|*x,&a);}",
        true,
    ),
    (
        "outer-named",
        "fn run<'a>(x:&'a u8)->u8{let f=|y:&'a u8|*y;f(x)}fn main(){}",
        true,
    ),
    (
        "disjoint-calls",
        "fn main(){let f=|x:&u8|*x;{let a=7;let _=f(&a);}{let b=9;let _=f(&b);}}",
        true,
    ),
];
#[test]
fn independently_qualified_lifetime_interfaces() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, expected) in CASES {
        rustc::qualify(&format!("closure-lifetimes-{name}"), source, expected)?;
    }
    Ok(())
}
#[test]
fn closure_interfaces_require_their_own_lifetime_proof() {
    for &(name, source, expected) in CASES {
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), expected, "{name}: {:?}", result.err());
    }
}
