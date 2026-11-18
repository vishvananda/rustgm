//! A wrapper's erased callable signature guides its stored closure.
use super::TestResult;

const DECLARATIONS: &str = r#"
#[rustc_diagnostic_item = "Send"] pub unsafe auto trait Send {}
#[lang = "sync"] pub unsafe auto trait Sync {}
struct Thread;
type Task = Box<dyn Send + FnOnce()>;
type Hook = Box<dyn Send + Sync + Fn(&Thread) -> Task>;
fn action() {}
fn init(_: &Thread) -> fn() { action }
"#;

const CASES: &[(&str, &str, &str, bool)] = &[
    ("inferred", "Sync", "Send", true),
    ("missing-sync", "", "Send", false),
    ("missing-result-send", "Sync", "", false),
];

#[test]
fn boxed_closure_uses_the_erased_signature_and_keeps_auto_trait_requirements() -> TestResult {
    let fixture =
        crate::resolve::fixture("pa31/tests/spec/explicit-dynamic-call-members/prefix.rs")?;
    let (roles, _) = fixture
        .split_once("impl Clone for u32")
        .ok_or("Box and callable roles")?;
    let mut failures = Vec::new();
    for &(name, sync, send, accepted) in CASES {
        let sync = if sync.is_empty() {
            String::new()
        } else {
            format!("{sync} + ")
        };
        let send = if send.is_empty() {
            String::new()
        } else {
            format!("{send} + ")
        };
        let source = format!(
            r#"{roles}{DECLARATIONS}
            fn erase<F, G>(hook: F) -> Hook
            where
                F: 'static + Send + {sync}Fn(&Thread) -> G,
                G: 'static + {send}FnOnce(),
            {{
                Box::new(move |thread| Box::new(hook(thread)))
            }}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let _hook = erase(init);
                7
            }}"#
        );
        let lowered = crate::ffi_program::exported(&source, "boxed_closures");
        match (accepted, lowered) {
            (true, Ok(program))
                if program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == "observe" && symbol.body.is_some()) => {}
            (false, Err(error))
                if ["trait obligation", "Unsize input formation"]
                    .iter()
                    .any(|message| error.to_string().contains(message)) => {}
            (_, result) => failures.push(format!("{name}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
