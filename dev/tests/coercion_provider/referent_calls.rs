//! A call's concrete referent is inferred before its enclosing reference coercion.
use super::TestResult;

const DECLARATIONS: &str = r#"
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
#[lang = "dispatch_from_dyn"] pub trait DispatchFromDyn<T>: Sized {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
trait Probe { fn value(&self) -> u8; }
struct Descriptor;
impl Probe for Descriptor { fn value(&self) -> u8 { 7 } }
struct Other;
impl Probe for Other { fn value(&self) -> u8 { 8 } }
struct Socket(Descriptor, Other);
trait AsInner<Inner: ?Sized> { fn as_inner(&self) -> &Inner; }
impl AsInner<Descriptor> for Socket { fn as_inner(&self) -> &Descriptor { &self.0 } }
struct Builder(u8);
impl Builder {
    fn field(&mut self, _: &str, value: &dyn Probe) {
        self.0 = value.value();
    }
}
"#;

const ANOTHER_IMPL: &str = r"
impl AsInner<Other> for Socket { fn as_inner(&self) -> &Other { &self.1 } }
";

#[test]
fn unit_calls_infer_shared_inputs_before_later_reference_coercion() -> TestResult {
    let fixture =
        crate::resolve::fixture("pa31/tests/spec/specialization-fixpoint/core-extend.rs")?;
    let (roles, _) = fixture.split_once("pub trait Iter {").ok_or("core roles")?;
    for (name, annotation, call, accepted) in [
        ("method", "", "items.extend(Once(&path));", true),
        (
            "qualified",
            "",
            "Extend::extend(&mut items, Once(&path));",
            true,
        ),
        (
            "typed",
            ": Collection<&Path>",
            "items.extend(Once(&path));",
            true,
        ),
        (
            "wrong",
            ": Collection<Path>",
            "items.extend(Once(&path));",
            false,
        ),
    ] {
        let source = format!(
            r#"{roles}
            #[lang="drop"] pub const trait Drop {{ fn drop(&mut self); }}
            struct Path(u8);
            enum Slot<T> {{ Empty, Found(T) }}
            struct Collection<T>(Slot<T>);
            trait Source {{ type Item; fn take(self) -> Self::Item; }}
            struct Once<T>(T);
            impl<T> Source for Once<T> {{
                type Item = T;
                fn take(self) -> T {{ self.0 }}
            }}
            trait Extend<T> {{ fn extend<I: Source<Item=T>>(&mut self, item: I); }}
            impl<T> Extend<T> for Collection<T> {{
                fn extend<I: Source<Item=T>>(&mut self, item: I) {{ self.0 = Slot::Found(item.take()); }}
            }}
            impl<T> Collection<T> {{
                fn new() -> Self {{ Self(Slot::Empty) }}
                fn get(&self) -> &T {{
                    match &self.0 {{ Slot::Found(value) => value, Slot::Empty => loop {{}} }}
                }}
            }}
            fn read(path: &Path) -> u8 {{ path.0 }}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let path = Path(7);
                let mut items{annotation} = Collection::new();
                {call}
                read(items.get())
            }}"#
        );
        let result = crate::ffi_program::exported(&source, "unit_call_inputs");
        match (accepted, result) {
            (true, Ok(_)) => {}
            (false, Err(error)) if error.to_string().contains("trait obligation") => {}
            (_, result) => return Err(format!("{name}: {:?}", result.err()).into()),
        }
    }
    Ok(())
}

fn prefix() -> TestResult<String> {
    let core = crate::resolve::fixture("pa31/tests/spec/unsizing-roles/object-goal.rs")?;
    let (roles, _) = core
        .split_once("fn require<T:")
        .ok_or("Unsize provider prefix")?;
    Ok(format!("{roles}{DECLARATIONS}"))
}

#[test]
fn call_proofs_infer_referents_before_trait_object_and_slice_coercions() -> TestResult {
    let prefix = prefix()?;
    let cases = [
        (
            "nested",
            "",
            r#"let mut builder = Builder(0); builder.field("fd", socket.as_inner()); builder.0"#,
        ),
        (
            "binding",
            "",
            "let value: &dyn Probe = socket.as_inner(); value.value()",
        ),
        (
            "return",
            "fn erased(socket: &Socket) -> &dyn Probe { socket.as_inner() }",
            "erased(&socket).value()",
        ),
        (
            "slice",
            r"
             struct Bytes([u8; 1]);
             impl AsInner<[u8; 1]> for Bytes { fn as_inner(&self) -> &[u8; 1] { &self.0 } }
             fn use_slice(_: &[u8]) -> u8 { 7 }
            ",
            "let bytes = Bytes([7]); use_slice(bytes.as_inner())",
        ),
        (
            "disambiguated",
            ANOTHER_IMPL,
            "let value: &Descriptor = socket.as_inner(); value.value()",
        ),
    ];
    for (name, declarations, body) in cases {
        let source = format!(
            r#"{prefix}{declarations}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let socket = Socket(Descriptor, Other);
                {body}
            }}"#
        );
        let program = crate::ffi_program::exported(&source, "inferred_referent")
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
fn referent_inference_preserves_ambiguity_trait_and_lifetime_failures() -> TestResult {
    let prefix = prefix()?;
    let cases = [
        (
            "ambiguous",
            ANOTHER_IMPL,
            "let _: &dyn Probe = socket.as_inner();",
            "trait obligation",
        ),
        (
            "wrong-trait",
            "trait Missing {}",
            "let _: &dyn Missing = socket.as_inner();",
            "trait",
        ),
        (
            "escaping-reference",
            "fn erased(socket: &Socket) -> &'static dyn Probe { socket.as_inner() }",
            "let _ = erased(&socket);",
            "lifetime",
        ),
    ];
    for (name, declarations, body, diagnostic) in cases {
        let source = format!(
            r#"{prefix}{declarations}
            #[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 {{
                let socket = Socket(Descriptor, Other);
                {body}
                7
            }}"#
        );
        let error = crate::ffi_program::exported(&source, "invalid_inferred_referent")
            .err()
            .ok_or_else(|| format!("{name} accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}
