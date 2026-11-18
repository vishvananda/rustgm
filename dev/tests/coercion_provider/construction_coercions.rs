//! Object erasure converts the borrow of a fully constructed value.
use super::TestResult;

const DECLARATIONS: &str = r#"
#[lang = "dispatch_from_dyn"] pub trait DispatchFromDyn<T>: Sized {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
trait Payload { fn value(&self) -> u8; }
enum Slot<T> { Full(T), Empty }
struct Scoped<'a> { message: &'a u8, pending: Slot<u8> }
impl Payload for Scoped<'_> { fn value(&self) -> u8 { *self.message } }
struct Generic<T> { value: Slot<T> }
impl<T> Payload for Generic<T> { fn value(&self) -> u8 { 7 } }
struct Tuple<T>(T);
impl Payload for Tuple<u8> { fn value(&self) -> u8 { self.0 } }
impl Payload for (u8, u8) { fn value(&self) -> u8 { self.0 } }
impl Payload for [u8; 2] { fn value(&self) -> u8 { 7 } }
struct WithoutPayload { value: u8 }
fn invoke(value: &mut dyn Payload) -> u8 { value.value() }
fn static_object(value: &mut (dyn Payload + 'static)) -> u8 { value.value() }
"#;

const POSITIVE: &[(&str, &str)] = &[
    (
        "borrowed-fields",
        "invoke(&mut Scoped { message: &message, pending: Slot::Empty })",
    ),
    (
        "borrowed-generic-fields",
        "invoke(&mut Generic { value: Slot::Full(message) })",
    ),
    ("borrowed-tuple-constructor", "invoke(&mut Tuple(message))"),
    ("borrowed-tuple", "invoke(&mut (message, 8u8))"),
    ("borrowed-array", "invoke(&mut [message, 8u8])"),
    ("borrowed-repeat", "invoke(&mut [message; 2])"),
];

const NEGATIVE: &[(&str, &str, &str)] = &[
    (
        "unimplemented-trait",
        "invoke(&mut WithoutPayload { value: message })",
        "trait",
    ),
    (
        "object-by-value",
        "let _: dyn Payload = Generic { value: Slot::Full(message) }; 7",
        "incompatible types",
    ),
    (
        "escaping-borrow",
        "static_object(&mut Scoped { message: &message, pending: Slot::Empty })",
        "outlive",
    ),
];

#[test]
fn borrowed_constructors_form_the_object_after_inferring_its_operand() -> TestResult {
    let core = crate::resolve::fixture("pa31/tests/spec/unsizing-roles/object-goal.rs")?;
    let (roles, _) = core
        .split_once("fn require<T:")
        .ok_or("Unsize provider prefix")?;
    let compile = |body| {
        let source = format!(
            r#"{roles}{DECLARATIONS}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let message = 7u8;
                {body}
            }}"#
        );
        crate::ffi_program::exported(&source, "borrowed_constructors")
    };
    for &(name, body) in POSITIVE {
        let program = compile(body).map_err(|error| format!("{name}: {error}"))?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "observe" && symbol.body.is_some()),
            "{name}"
        );
    }
    for &(name, body, diagnostic) in NEGATIVE {
        let error = compile(body)
            .err()
            .ok_or_else(|| format!("{name} accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}
