//! Callable ABI and safety remain distinct through ordinary lowering and proof.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::{
    expand::config::Config,
    resolve::{self, load::InputFile},
    sema,
    support::source::SourceFile,
};
const CASES: &[(&str, &str, bool)] = &[
    (
        "rust-to-c",
        "fn f(x:i32)->i32{x}fn main(){let _:extern \"C\" fn(i32)->i32=f;}",
        false,
    ),
    (
        "c-to-rust",
        "extern \"C\" fn f(x:i32)->i32{x}fn main(){let _:fn(i32)->i32=f;}",
        false,
    ),
    (
        "c-item",
        "extern \"C\" fn f(x:i32)->i32{x}fn main(){let p:extern \"C\" fn(i32)->i32=f;let _=p(3);}",
        true,
    ),
    (
        "c-default",
        "extern fn f(x:i32)->i32{x}fn main(){let p:extern \"C\" fn(i32)->i32=f;let _=p(3);}",
        true,
    ),
    (
        "pointer-default",
        "extern \"C\" fn f(x:i32)->i32{x}fn main(){let p:extern fn(i32)->i32=f;let _=p(3);}",
        true,
    ),
    (
        "safe-to-unsafe",
        "extern \"C\" fn f(x:i32)->i32{x}fn main(){let p:unsafe extern \"C\" fn(i32)->i32=f;let _=unsafe{p(3)};}",
        true,
    ),
    (
        "unsafe-pointer",
        "extern \"C\" fn f(x:i32)->i32{x}fn main(){let p:extern \"C\" fn(i32)->i32=f;let q:unsafe extern \"C\" fn(i32)->i32=p;let _=unsafe{q(3)};}",
        true,
    ),
    (
        "unsafe-to-safe",
        "unsafe extern \"C\" fn f(x:i32)->i32{x}fn main(){let _:extern \"C\" fn(i32)->i32=f;}",
        false,
    ),
    (
        "closure-to-c",
        "fn main(){let _:extern \"C\" fn(i32)->i32=|x|x;}",
        false,
    ),
    (
        "c-call-trait",
        "extern \"C\" fn f(x:i32)->i32{x}fn take<F:Fn(i32)->i32>(f:F){let _=f(3);}fn main(){take(f);}",
        false,
    ),
    (
        "c-pointer-call-trait",
        "extern \"C\" fn f(x:i32)->i32{x}fn take<F:Fn(i32)->i32>(f:F){let _=f(3);}fn main(){let p:extern \"C\" fn(i32)->i32=f;take(p);}",
        false,
    ),
    (
        "const-c",
        "const extern \"C\" fn f(x:i32)->i32{x}const X:i32=f(3);fn main(){let _=X;}",
        true,
    ),
    (
        "unsafe-closure",
        "fn main(){let f:unsafe fn(&u8)->&u8=|x|x;let a=7;let _=unsafe{f(&a)};}",
        true,
    ),
    (
        "nested-safety",
        "fn f(){}fn main(){let p:fn()=f;let _: &unsafe fn()=&p;}",
        false,
    ),
    (
        "nested-abi",
        "extern \"C\" fn f(){}fn main(){let p:extern \"C\" fn()=f;let _: &fn()=&p;}",
        false,
    ),
    (
        "captured-unsafe",
        "fn main(){let a=7;let _:unsafe fn()->i32=||a;}",
        false,
    ),
    (
        "zero-repeat",
        "fn same(x:&u8)->&u8{x}fn main(){let _:[unsafe fn(&u8)->&u8;0]=[same;0];}",
        true,
    ),
];
fn checked(source: &str) -> Result<(), String> {
    let input = InputFile {
        identity: "lib.rs".into(),
        path: "lib.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Config::default())
        .map_err(|e| e.to_string())?;
    let mut report = resolve::analyze_expanded(expanded).map_err(|e| e.to_string())?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    let entry = sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
#[test]
fn conventions_survive_substitution_and_inference() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{
        Kind, Primitive, RegionId, Store, TypeId,
        callable::{Abi, Convention},
        infer::{Constructor, Context},
        relations::Constraints,
        substitute::Bound,
        variance::Variances,
    };
    let mut types = Store::default();
    let reference = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let parameters = types.intern_list(&[reference])?;
    let mut signatures = Vec::new();
    for convention in [
        Convention::RUST,
        Convention::C,
        Convention {
            abi: Abi::C,
            variadic: true,
        },
    ] {
        let signature = types.intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result: reference,
            unsafe_: false,
            convention,
        })?;
        let transformed = Bound::new(vec![RegionId::STATIC]).apply(&mut types, signature)?;
        let Some(Kind::Function {
            parameters,
            result,
            convention: actual,
            ..
        }) = types.kind(transformed)
        else {
            return Err("missing transformed signature".into());
        };
        assert_eq!(actual, convention);
        assert_ne!(signature, transformed);
        let parameter = types.list(parameters).ok_or("missing parameter")?[0];
        let mut context = Context::default();
        let input = context.known(&types, parameter)?;
        let output = context.known(&types, result)?;
        let term = context.construct(
            Constructor::Function {
                bound_regions: 1,
                parameters: 1,
                unsafe_: false,
                convention,
            },
            &[input, output],
        )?;
        let solved = context.finish(&mut types)?;
        assert_eq!(solved.types[term.0 as usize], transformed);
        signatures.push(transformed);
    }
    for other in &signatures[1..] {
        let variances = Variances::default();
        let mut proof = Constraints::new(&types, &variances, 0, 10_000)?;
        assert!(proof.coerce(signatures[0], *other, false, 0).is_err());
    }
    assert!(
        types
            .intern(Kind::Function {
                bound_regions: 0,
                parameters,
                result: TypeId::UNIT,
                unsafe_: false,
                convention: Convention {
                    abi: Abi::Rust,
                    variadic: true
                },
            })
            .is_err()
    );
    Ok(())
}
#[test]
fn conventions_preserve_independent_abi_and_safety() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("foreign-convention-{name}"), source, accepted)?;
        let result = checked(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn conventions_execute_selected_scalar_calls() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/foreign-conventions/cases",
        &[
            (
                "constant-c",
                "const extern \"C\" fn f(x:i32)->i32{x+3}const X:i32=f(39);fn main(){check(X==42);}",
                false,
            ),
            (
                "c-pointer",
                "extern \"C\" fn f(x:i32)->i32{x+3}fn main(){let p:extern \"C\" fn(i32)->i32=f;check(p(39)==42);}",
                false,
            ),
            (
                "unsafe-pointer",
                "extern \"C\" fn f(x:i32)->i32{x+3}fn main(){let p:extern \"C\" fn(i32)->i32=f;let q:unsafe extern \"C\" fn(i32)->i32=p;check(unsafe{q(39)==42});}",
                false,
            ),
            (
                "unsafe-closure",
                "fn main(){let f:unsafe fn(&u8)->&u8=|x|x;let a=7;check(unsafe{*f(&a)==7});}",
                false,
            ),
            (
                "c-lifetime",
                "extern \"C\" fn same(x:&u8)->&u8{x}fn main(){let f:unsafe extern \"C\" fn(&u8)->&u8=same;let a=7;check(unsafe{*f(&a)==7});}",
                false,
            ),
        ],
    )
}
