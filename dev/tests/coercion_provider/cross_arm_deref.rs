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
