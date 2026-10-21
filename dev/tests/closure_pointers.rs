//! Noncapturing closures reify their checked source body through the ordinary call ABI.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "binding",
        "fn main(){let f:fn(u8)->u8=|x|x+1;check(f(7)==8);}",
        true,
    ),
    (
        "argument",
        "fn run(f:fn(u8)->u8)->u8{f(7)}fn main(){check(run(|x|x+2)==9);}",
        true,
    ),
    (
        "result",
        "fn make()->fn(u8)->u8{|x|x+3}fn main(){check(make()(7)==10);}",
        true,
    ),
    (
        "array",
        "fn main(){let f:[fn(u8)->u8;2]=[|x|x+1,|x|x+2];check(f[0](7)==8);check(f[1](7)==9);}",
        true,
    ),
    (
        "join",
        "fn main(){let f=if true{|x:u8|x+1}else{|x:u8|x+2};check(f(7)==8);}",
        true,
    ),
    (
        "mixed-join",
        "fn add(x:u8)->u8{x+2}fn main(){let f=if true{|x:u8|x+1}else{add};check(f(7)==8);}",
        true,
    ),
    (
        "empty-move",
        "fn main(){let f:fn(u8)->u8=move|x|x+1;check(f(7)==8);}",
        true,
    ),
    (
        "existing",
        "fn main(){let f=|x:u8|x+1;let g:fn(u8)->u8=f;check(g(7)==8);check(f(8)==9);}",
        true,
    ),
    (
        "cast",
        "fn main(){let f=(|x:u8|x+1) as fn(u8)->u8;check(f(7)==8);}",
        true,
    ),
    (
        "nested",
        "fn main(){let f=||{let g:fn(u8)->u8=|x|x+1;g(7)};check(f()==8);}",
        true,
    ),
    (
        "generic-dependency",
        "fn id<T>(x:T)->T{x}fn main(){let f:fn(u8)->u8=|x|id(x);check(f(7)==7);}",
        true,
    ),
    (
        "input",
        "fn main(){let _f:fn(u8)->u8=|x:bool|if x{1}else{2};}",
        false,
    ),
    ("output", "fn main(){let _f:fn(u8)->bool=|x|x+1;}", false),
    ("arity", "fn main(){let _f:fn(u8)->u8=||1;}", false),
];
#[test]
fn source_coercions_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, expected) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("closure-pointer-{name}"), &source, expected)?;
        let actual = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
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
fn function_pointers_execute_the_checked_closure() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("pa24-source-bodies/pointers", &cases)
}
