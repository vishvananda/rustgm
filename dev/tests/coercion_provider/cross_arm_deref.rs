//! Reference joins preserve each arm's own dereference and loan.
use super::TestResult;

const DECLARATIONS: &str = r"
struct Wrap<T>(T);
impl<T> Deref for Wrap<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}
impl<T> DerefMut for Wrap<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}
enum Choice { Wrapped, Plain }
";

const VALID: &[(&str, &str)] = &[
    (
        "match",
        "match choice { Choice::Wrapped => wrapped, Choice::Plain => plain }",
    ),
    (
        "match-explicit",
        "match choice { Choice::Wrapped => &mut *wrapped, Choice::Plain => plain }",
    ),
    ("if", "if flag { wrapped } else { plain }"),
    ("direct", "wrapped"),
    (
        "open-referent",
        "let value: &mut _ = if flag { wrapped } else { wrapped }; value",
    ),
];

#[test]
fn reference_result_coerces_each_branch_with_its_own_loan() -> TestResult {
    let input = crate::resolve::fixture("pa31/tests/spec/source-field-deref/shadow.rs")?;
    let (roles, _) = input.split_once("pub struct Leaf").ok_or("Deref roles")?;
    for &(name, body) in VALID {
        let source = format!(
            r#"{roles}{DECLARATIONS}
            fn choose<'a, T>(flag: bool, choice: Choice, wrapped: &'a mut Wrap<T>, plain: &'a mut T)
                -> &'a mut T
            {{ {body} }}
            #[unsafe(no_mangle)] pub extern "C" fn observe(flag: bool) -> u8 {{
                let mut wrapped = Wrap(7u8);
                let mut plain = 8u8;
                *choose(flag, Choice::Wrapped, &mut wrapped, &mut plain)
            }}"#
        );
        let program = crate::ffi_program::exported(&source, "branch_deref")
            .map_err(|error| format!("{name}: {error}"))?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "observe" && symbol.body.is_some()),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn reference_result_rejects_shared_and_shorter_branch_loans() -> TestResult {
    let input = crate::resolve::fixture("pa31/tests/spec/source-field-deref/shadow.rs")?;
    let (roles, _) = input.split_once("pub struct Leaf").ok_or("Deref roles")?;
    let cases = [
        (
            "shared-arm",
            "fn choose<'a, T>(flag: bool, wrapped: &'a mut Wrap<T>, plain: &'a mut T) -> &'a mut T {
                if flag { &*wrapped } else { plain }
             }",
            "shared reference",
        ),
        (
            "short-arm",
            "fn choose<'a, 'b, T>(flag: bool, wrapped: &'a mut Wrap<T>, plain: &'b mut T)
                -> &'b mut T
             {
                if flag { wrapped } else { plain }
             }",
            "outlive",
        ),
    ];
    for (name, declaration, diagnostic) in cases {
        let source = format!(
            r#"{roles}{DECLARATIONS}{declaration}
            #[unsafe(no_mangle)] pub extern "C" fn observe(flag: bool) -> u8 {{
                let mut wrapped = Wrap(7u8);
                let mut plain = 8u8;
                *choose(flag, &mut wrapped, &mut plain)
            }}"#
        );
        let error = crate::ffi_program::exported(&source, "invalid_branch_deref")
            .err()
            .ok_or_else(|| format!("{name} accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn mutable_call_reborrows_refine_the_dereferenced_place() -> TestResult {
    let dispatch = crate::resolve::fixture("pa31/tests/spec/source-object-calls/shared.rs")?;
    let (dispatch, _) = dispatch.split_once("fn call(f:").ok_or("callable roles")?;
    let deref = crate::resolve::fixture("pa31/tests/spec/dereference-source/foundation.rs")?;
    let (_, deref) = deref.split_once("#[lang=\"deref\"]").ok_or("Deref roles")?;
    let cases = [
        (
            "direct",
            "fn(&mut u8) -> u8",
            "read",
            "mut guard",
            "*guard",
            true,
        ),
        (
            "dynamic",
            "&dyn Fn(&mut u8) -> u8",
            "&read",
            "mut guard",
            "*guard",
            true,
        ),
        (
            "explicit",
            "&dyn Fn(&mut u8) -> u8",
            "&read",
            "mut guard",
            "&mut **guard",
            true,
        ),
        (
            "immutable",
            "&dyn Fn(&mut u8) -> u8",
            "&read",
            "guard",
            "*guard",
            false,
        ),
    ];
    for (name, callable, argument, binding, operand, accepted) in cases {
        let source = format!(
            r#"{dispatch}#[lang="deref"]{deref}
            struct Guard<'a>(&'a mut u8);
            impl<'a> Deref for Guard<'a> {{
                type Target = &'a mut u8;
                fn deref(&self) -> &Self::Target {{ &self.0 }}
            }}
            impl<'a> DerefMut for Guard<'a> {{
                fn deref_mut(&mut self) -> &mut Self::Target {{ &mut self.0 }}
            }}
            fn dispatch(value: &mut u8, take: {callable}) -> u8 {{
                let {binding} = Guard(value);
                take({operand})
            }}
            fn read(value: &mut u8) -> u8 {{ *value }}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let mut value = 7u8;
                dispatch(&mut value, {argument})
            }}"#
        );
        let result = crate::ffi_program::exported(&source, "call_reborrow");
        if accepted {
            let program = result.map_err(|error| format!("{name}: {error}"))?;
            assert!(
                program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == "observe" && symbol.body.is_some()),
                "{name}"
            );
        } else {
            let error = result.err().ok_or("immutable guard was borrowed mutably")?;
            assert!(
                error.to_string().contains("immutable local"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}
