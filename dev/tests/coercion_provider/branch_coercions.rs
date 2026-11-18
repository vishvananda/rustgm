//! Each branch proves its own conversion to the function's wrapper result.
use super::TestResult;

const DECLARATIONS: &str = r#"
#[rustc_diagnostic_item = "Send"] pub unsafe auto trait Send {}
#[lang = "sync"] pub unsafe auto trait Sync {}
struct Notice<'a>(&'a u8);
fn default_hook(_: &Notice<'_>) {}
trait Probe {}
struct Value;
impl Probe for Value {}
struct LocalOnly;
impl Probe for LocalOnly {}
impl !Send for LocalOnly {}
"#;

const CASES: &[(&str, &str, &str, Option<&str>)] = &[
    (
        "callable-match",
        "dyn Fn(&Notice<'_>) + Send + Sync",
        "match hook { Hook::Default => Box::new(default_hook), Hook::Custom(hook) => hook }",
        None,
    ),
    (
        "reversed-arms",
        "dyn Fn(&Notice<'_>)",
        "match hook { Hook::Custom(hook) => hook, Hook::Default => Box::new(default_hook) }",
        None,
    ),
    (
        "if-let",
        "dyn Fn(&Notice<'_>)",
        "if let Hook::Custom(hook) = hook { hook } else { Box::new(default_hook) }",
        None,
    ),
    (
        "trait-object",
        "dyn Probe",
        "match hook { Hook::Default => Box::new(Value), Hook::Custom(hook) => hook }",
        None,
    ),
    (
        "auto-marker",
        "dyn Probe + Send",
        "match hook { Hook::Default => Box::new(Value), Hook::Custom(hook) => hook }",
        None,
    ),
    (
        "slice",
        "[u8]",
        "match hook { Hook::Default => Box::new([7u8; 2]), Hook::Custom(hook) => hook }",
        None,
    ),
    (
        "inferred-wrapper",
        "dyn Fn(&Notice<'_>)",
        r"
            let selected: Box<_> = if flag { Box::new(default_hook) } else { Box::new(default_hook) };
            selected
        ",
        None,
    ),
    (
        "diverging-arms",
        "dyn Fn(&Notice<'_>)",
        r"
            if flag { return Box::new(default_hook); }
            else { return Box::new(default_hook); }
        ",
        None,
    ),
    (
        "missing-trait",
        "dyn Probe",
        "match hook { Hook::Default => Box::new(default_hook), Hook::Custom(hook) => hook }",
        Some("trait obligation"),
    ),
    (
        "missing-auto-marker",
        "dyn Probe + Send",
        "match hook { Hook::Default => Box::new(LocalOnly), Hook::Custom(hook) => hook }",
        Some(
            "conditional Unsize input formation cannot publish an absence without paying its debts",
        ),
    ),
];

#[test]
fn wrapper_branches_coerce_to_the_known_result_with_arm_local_proofs() -> TestResult {
    let input = crate::resolve::fixture("pa31/tests/spec/explicit-dynamic-call-members/prefix.rs")?;
    let (roles, _) = input
        .split_once("impl Clone for u32")
        .ok_or("Box and Fn roles")?;
    let mut failures = Vec::new();
    for &(name, target, body, rejected) in CASES {
        let source = format!(
            r#"#![feature(negative_impls)]
            {roles}{DECLARATIONS}
            type Handler = Box<{target}>;
            enum Hook {{ Default, Custom(Handler) }}
            fn choose(flag: bool, hook: Hook) -> Handler {{ {body} }}
            #[unsafe(no_mangle)] pub extern "C" fn observe(flag: bool) -> u8 {{
                let _handler = choose(flag, Hook::Default);
                7
            }}"#
        );
        let result = crate::ffi_program::exported(&source, "wrapper_branches");
        match (rejected, result) {
            (None, Ok(program)) => assert!(
                program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == "observe" && symbol.body.is_some()),
                "{name}"
            ),
            (Some(message), Err(error)) if error.to_string().contains(message) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".to_owned())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
