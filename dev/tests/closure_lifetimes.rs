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

#[path = "harness/native_references.rs"]
mod native;
const CONTEXTUAL_CASES: &[(&str, &str, bool)] = &[
    (
        "shared-inputs",
        "fn main(){let f:for<'a>fn(&'a u8,&'a u8)->&'a u8=|_x,y|y;let a=7;let b=9;check(*f(&a,&b)==9);}",
        true,
    ),
    (
        "nested-callable",
        "fn id(x:&u8)->&u8{x}fn main(){let f:fn(fn(&u8)->&u8)->fn(&u8)->&u8=|g|g;let a=7;check(*f(id)(&a)==7);}",
        true,
    ),
    (
        "identity",
        "fn main(){let f:fn(&u8)->&u8=|x|x;let a=7;check(*f(&a)==7);}",
        true,
    ),
    (
        "annotated",
        "fn main(){let f:fn(&u8)->&u8=|x:&u8|x;let a=7;check(*f(&a)==7);}",
        true,
    ),
    (
        "explicit-result",
        "fn main(){let f:fn(&u8)->&u8=|x:&u8|->&u8{x};let a=7;check(*f(&a)==7);}",
        true,
    ),
    (
        "separate-inputs",
        "fn main(){let f:for<'a,'b>fn(&'a u8,&'b u8)->&'a u8=|x,_y|x;let a=7;{let b=9;check(*f(&a,&b)==7);}}",
        true,
    ),
    (
        "argument",
        "fn run(f:fn(&u8)->&u8)->u8{let x=7;*f(&x)}fn main(){check(run(|x|x)==7);}",
        true,
    ),
    (
        "return",
        "fn make()->fn(&u8)->&u8{|x|x}fn main(){let a=7;check(*make()(&a)==7);}",
        true,
    ),
    (
        "nested",
        "fn main(){let f=||{let g:fn(&u8)->&u8=|x|x;let a=7;*g(&a)};check(f()==7);}",
        true,
    ),
    (
        "outer-lifetime",
        "fn run<'a>(a:&'a u8)->&'a u8{let f:fn(&'a u8)->&'a u8=|x|x;f(a)}fn main(){let a=7;check(*run(&a)==7);}",
        true,
    ),
    (
        "mutable",
        "fn main(){let f:fn(&mut u8)->&mut u8=|x|x;let mut a=7;*f(&mut a)=9;check(a==9);}",
        true,
    ),
    (
        "crossed-output",
        "fn main(){let _f:for<'a,'b>fn(&'a u8,&'b u8)->&'a u8=|_x,y|y;}",
        false,
    ),
    (
        "local-output",
        "fn main(){let _f:fn(&u8)->&u8=|_x|{let a=7;&a};}",
        false,
    ),
    (
        "static-output",
        "fn main(){let _f:fn(&u8)->&'static u8=|x|x;}",
        false,
    ),
    (
        "escaped-call",
        "fn main(){let f:fn(&u8)->&u8=|x|x;let r;{let a=7;r=f(&a);}let _=*r;}",
        false,
    ),
    (
        "shared-to-mutable",
        "fn main(){let _f:fn(&u8)->&mut u8=|x|x;}",
        false,
    ),
];

#[test]
fn contextual_contracts_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CONTEXTUAL_CASES {
        let text = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("closure-context-{name}"), &text, accepted)?;
        let actual = semantic::analyze(&text)
            .map_err(|e| format!("semantics: {e}"))
            .and_then(|(mut report, mut facts)| {
                let entry =
                    rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
                rustgm::lower::instances::build(&report, &mut facts, entry)
                    .map_err(|e| format!("lowering: {e}"))
            });
        assert_eq!(actual.is_ok(), accepted, "{name}: {:?}", actual.err());
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn contextual_closures_execute_their_proved_signatures() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CONTEXTUAL_CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("parallel/closure-contexts/cases", &cases)
}
