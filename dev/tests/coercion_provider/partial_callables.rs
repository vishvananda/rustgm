//! Source coercions keep written nominal arguments inside their callable's binder.
use super::TestResult;

const CALLABLES: &str = r#"
#[lang = "tuple_trait"] #[rustc_deny_explicit_impl] pub trait Tuple {}
#[lang = "fn_once"] pub const trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"] type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
#[lang = "fn_mut"] pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
#[lang = "fn"] pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
enum Choice<T> { Some(T), None }
struct Input;
fn choose(input: &Input) -> Choice<(u8, &Input)> { Choice::Some((3, input)) }
"#;

const CASES: &[(&str, bool, &str)] = &[
    (
        "mixed-callables",
        true,
        r"
            let _table: &[fn(&Input) -> Choice<(_, &Input)>] = &[
                |input| Choice::Some((1u8, input)),
                |input| Choice::Some((2u8, input)),
                choose,
            ];
        ",
    ),
    (
        "function-item",
        true,
        "let _table: &[for<'a> fn(&'a Input) -> Choice<(_, &'a Input)>] = &[choose];",
    ),
    (
        "single-pointer",
        true,
        "let _pick: fn(&Input) -> Choice<(_, &Input)> = choose;",
    ),
    (
        "fixed-array",
        true,
        "let _table: [fn(&Input) -> Choice<(_, &Input)>; 1] = [choose];",
    ),
    (
        "wrong-return-component",
        false,
        r"
            let _table: &[fn(&Input) -> Choice<(_, &Input)>] = &[
                |input| Choice::Some((true, input)),
                choose,
            ];
        ",
    ),
    (
        "escaping-inferred-lifetime",
        false,
        "let _pick: for<'a> fn(&'a Input) -> Choice<_> = choose;",
    ),
];

#[test]
fn partial_callable_annotations_preserve_nominal_argument_binders() -> TestResult {
    let core = crate::resolve::fixture("pa31/tests/spec/unsizing-roles/object-goal.rs")?;
    let (roles, _) = core
        .split_once("fn require<T:")
        .ok_or("Unsize provider prefix")?;
    for (name, expected, body) in CASES {
        let source = format!(
            r#"#![feature(unboxed_closures)]
            {roles}{CALLABLES}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                {body}
                7
            }}"#
        );
        let result = crate::ffi_program::exported(&source, "partial_callable");
        match (expected, result) {
            (true, Ok(program)) => assert!(
                program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == "observe" && symbol.body.is_some()),
                "{name}"
            ),
            (false, Err(error))
                if error.to_string().contains("incompatible types")
                    || error.to_string().contains("lifetime") => {}
            (_, result) => {
                return Err(format!(
                    "{name}: {}",
                    result.map_or_else(|error| error.to_string(), |_| "accepted".to_owned())
                )
                .into());
            }
        }
    }
    Ok(())
}
