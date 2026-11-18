//! `#[lang = "owned_box"]` makes its struct the built-in owning pointer, as
//! rustc 1.98.0 does for alloc's `Box<T, A>(Unique<T>, A)`. `*b` is a place of
//! its own behind the pointer the box's leading fields hold: a field access
//! steps through it without a `Deref` call, a move out of it leaves the box to
//! free its allocation without dropping the content, and the content can be
//! assigned again. Dropping a box drops whatever content remains, then runs
//! the box's own `Drop`, then drops its fields.
//!
//! The `owned-boxes` packet is a mini core and mini alloc whose `Box` keeps
//! alloc's field shape and allocates and frees through a C observer, which
//! prints every allocation, free and drop. The `pattern-*` programs hold the
//! content pointer as core does, in `*const T is !null`, which converts to a
//! trait object or slice pointer through its own `CoerceUnsized`. Outcomes
//! were declared before either compiler ran; rustc 1.98.0's are recorded in
//! its `outcomes.json`.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/owned-boxes";
/// Programs rustc accepts, with what rustc's build of each prints.
const ACCEPTED: [(&str, &str); 13] = [
    ("move-out", "alloc 1\ndrop 7\nfree\nvalue 7\n"),
    ("drop-whole", "alloc 1\ndrop 5\nfree\nvalue 5\n"),
    ("partial-move", "alloc 2\ndrop 1\ndrop 2\nfree\nvalue 1\n"),
    ("reinitialize", "alloc 1\ndrop 1\ndrop 2\nfree\nvalue 2\n"),
    (
        "nested",
        "alloc 24\nalloc 24\ndrop 1\ndrop 2\nfree\nfree\nvalue 1\n",
    ),
    ("method-move", "alloc 2\ndrop 2\nfree\ndrop 1\nvalue 1\n"),
    ("slice-content", "alloc 2\ndrop 3\ndrop 6\nfree\nvalue 9\n"),
    ("borrowed-content", "alloc 1\ndrop 8\nfree\nvalue 8\n"),
    ("dyn-content", "alloc 1\ndrop 4\nfree\nvalue 4\n"),
    ("pattern-move", "alloc 1\ndrop 7\nfree\nvalue 7\n"),
    ("pattern-dyn", "alloc 1\ndrop 4\nfree\nvalue 4\n"),
    ("pattern-slice", "alloc 2\ndrop 3\ndrop 6\nfree\nvalue 9\n"),
    (
        "box-in-box",
        "alloc 1\nalloc 8\ndrop 3\nfree\nfree\nvalue 3\n",
    ),
];
/// Programs rustc rejects, with the message `RustGM` rejects each with.
const REJECTED: [(&str, &str); 6] = [
    ("use-after-move", MOVED),
    (
        "move-from-borrowed-box",
        "cannot move a non-Copy value out of a reference",
    ),
    ("use-whole-after-partial", MOVED),
    (
        "move-from-drop-content",
        "cannot move a non-Copy field out of a type that implements Drop",
    ),
    ("immutable-box", "cannot assign through an immutable local"),
    (
        "unimplemented-dyn",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
];
const MOVED: &str = "local may be uninitialized or moved at this read";

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    owned_box_program(&crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?)
}

fn owned_box_program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(source, "owned_boxes", &config)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

/// Every accepted program lowers, and every control is rejected with its
/// own message.
#[test]
fn owned_boxes_lower_and_their_controls_reject() {
    let mut failures = Vec::new();
    for (name, _) in ACCEPTED {
        if let Err(error) = program(name) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, message) in REJECTED {
        match program(name) {
            Ok(_) => failures.push(format!("{name}: accepted")),
            Err(error) if !error.to_string().contains(message) => {
                failures.push(format!("{name}: {error}"));
            }
            Err(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const BOX_DEREF: &str = r#"
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
#[lang = "deref"] pub trait Deref: PointeeSized {
    #[lang = "deref_target"] type Target: PointeeSized;
    fn deref(&self) -> &Self::Target;
}
impl<T: ?Sized, A> Deref for Box<T, A> {
    type Target = T;
    fn deref(&self) -> &T { &**self }
}
struct Wrap(Box<Pair>);
impl Deref for Wrap {
    type Target = Box<Pair>;
    fn deref(&self) -> &Box<Pair> { &self.0 }
}
"#;

#[test]
fn method_receivers_preserve_owned_box_places_and_borrowed_access() -> Result {
    let fixture = crate::resolve::fixture(&format!("{PACKET}/box-in-box.rs"))?;
    let (roles, _) = fixture
        .split_once("#[unsafe(no_mangle)]")
        .ok_or("Box roles")?;
    let cases = [
        (
            "owned",
            BOX_DEREF,
            "let left = boxed.into_left(); left.0",
            true,
        ),
        (
            "builtin-only",
            "",
            "let left = boxed.into_left(); left.0",
            true,
        ),
        (
            "nested",
            BOX_DEREF,
            "let left = Box::new(boxed).into_left(); left.0",
            true,
        ),
        (
            "borrowed",
            BOX_DEREF,
            "let left = (&boxed).into_left(); left.0",
            false,
        ),
        (
            "box-before-overloaded",
            BOX_DEREF,
            "Box::new(Wrap(boxed)).value()",
            true,
        ),
        (
            "overloaded-before-box",
            BOX_DEREF,
            "Wrap(boxed).value()",
            true,
        ),
        (
            "box-overloaded-move",
            BOX_DEREF,
            "let left = Box::new(Wrap(boxed)).into_left(); left.0",
            false,
        ),
        (
            "shared-closure",
            BOX_DEREF,
            "let read = || boxed.value(); read()",
            true,
        ),
        (
            "owned-closure",
            BOX_DEREF,
            "let take = || boxed.into_left(); let left = take(); left.0",
            true,
        ),
        (
            "box-method-priority",
            "impl Box<Pair> { fn into_left(self) -> u8 { 77 } }",
            "boxed.into_left()",
            true,
        ),
    ];
    let mut failures = Vec::new();
    for (name, declarations, body, allowed) in cases {
        let text = format!(
            r#"#![feature(unboxed_closures)]
{roles}{declarations}
            impl Pair {{
                fn into_left(self) -> Noisy {{ self.left }}
                fn value(&self) -> u8 {{ self.left.0 }}
            }}
            #[unsafe(no_mangle)] pub extern "C" fn run() -> u8 {{
                let boxed = Box::new(Pair {{ left: Noisy(1), right: Noisy(2) }});
                {body}
            }}
        "#
        );
        match (allowed, owned_box_program(&text)) {
            (true, Ok(_)) => {}
            (false, Err(error))
                if error
                    .to_string()
                    .contains("cannot move a non-Copy value out of a reference") => {}
            (_, result) => failures.push(format!("{name}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Each accepted program prints exactly what rustc's build of it prints.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn owned_boxes_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (name, expected) in ACCEPTED {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/owned-boxes/native/{name}")),
            &program(name)?,
            &root.join(PACKET).join("observe.c"),
            expected.as_bytes(),
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

const DYNAMIC_MEMBERS: &str = "pa31/tests/spec/explicit-dynamic-call-members";
const DYNAMIC_MEMBER_TRACES: [(&str, &[u8]); 2] = [
    ("borrowed-shared-owned-tuple", b"drop 4\nvalue 7\n"),
    ("borrowed-mutable-owned-tuple", b"drop 3\ndrop 4\nvalue 2\n"),
];

#[test]
fn explicit_borrowed_dynamic_members_keep_exact_tuple_application() {
    let mut failures = Vec::new();
    for (name, _) in DYNAMIC_MEMBER_TRACES {
        match packet_program(DYNAMIC_MEMBERS, name, true) {
            Ok(output)
                if output
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name.starts_with("rg.callable.member.")) => {}
            Ok(_) => failures.push(format!(
                "{name}: explicit callable did not reach its virtual member"
            )),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    let wrong_tuple_cause = concat!(
        "file 0: 5389: ",
        "associated type requires an unambiguous validated implementation"
    );
    match packet_program(DYNAMIC_MEMBERS, "wrong-tuple-application", true) {
        Err(error) if error.to_string() == wrong_tuple_cause => {}
        Err(error) => failures.push(format!("wrong-tuple-application: {error}")),
        Ok(_) => failures.push("accepted a different dynamic call-trait application".into()),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn explicit_borrowed_dynamic_members_execute_exact_tuple_traces() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (name, expected) in DYNAMIC_MEMBER_TRACES {
        crate::observed_program::execute(
            &root.join(format!(
                "obj/parallel/explicit-dynamic-call-members/native/{name}"
            )),
            &packet_program(DYNAMIC_MEMBERS, name, true)?,
            &root.join(DYNAMIC_MEMBERS).join("observe.c"),
            expected,
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

const CASTS: &str = "pa31/tests/spec/coercion-casts";
fn coercion_program(name: &str, source_profile: bool) -> Result<rustgm::lowir::program::Program> {
    packet_program(CASTS, name, source_profile)
}
fn packet_program(
    packet: &str,
    name: &str,
    source_profile: bool,
) -> Result<rustgm::lowir::program::Program> {
    let text = crate::resolve::fixture(&format!("{packet}/{name}.rs"))?;
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let mut report = if source_profile {
        crate::resolve::source_provider_config(&text, "coercion_casts", &config)?
    } else {
        crate::resolve::resolve_config(&text, true, &config)?
    };
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(if source_profile {
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?
    } else {
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?
    })
}

/// Frozen profile/acceptance columns from `coercion-casts/outcomes.json`.
const COERCION_CASES: [(&str, bool, bool); 16] = [
    ("generic-array", true, true),
    ("method-result", true, true),
    ("caller-bound", true, true),
    ("slice-once", true, true),
    ("dyn", true, true),
    ("different-owner", true, false),
    ("wrong-element", true, false),
    ("reverse", true, false),
    ("changed-allocator", true, false),
    ("missing-implementation", true, false),
    ("stronger-mutability", true, false),
    ("unimplemented-dyn", true, false),
    ("moved-source", true, false),
    ("drop-opponent", true, false),
    ("ordinary-reference", false, true),
    ("ordinary-nominal", false, false),
];

#[test]
fn explicit_nominal_casts_keep_selected_proofs_and_ownership_opponents() {
    let mut failures = Vec::new();
    for (name, source, accepts) in COERCION_CASES {
        match coercion_program(name, source) {
            Ok(_) if accepts => {}
            Ok(_) => failures.push(format!("{name}: accepted a required rejection")),
            Err(error) if accepts => failures.push(format!("{name}: {error}")),
            Err(error) => {
                let error = error.to_string();
                let semantic = match name {
                    "moved-source" => error.contains("uninitialized"),
                    "drop-opponent" => error.contains(
                        "cannot move a non-Copy field out of a type that implements Drop",
                    ),
                    "ordinary-nominal" => true,
                    _ => ["cast", "coerc", "trait obligation", "incompatible types"]
                        .iter()
                        .any(|reason| error.contains(reason)),
                };
                let infrastructure = [
                    "budget",
                    "exhaust",
                    "panicked",
                    "not implemented",
                    "unsupported",
                    "coercion cast changed",
                    "coercion cast lost",
                    "coercion cast requires",
                    "missing unadjusted",
                ]
                .iter()
                .any(|reason| error.contains(reason));
                if !semantic || source && infrastructure {
                    failures.push(format!("{name}: wrong rejection cause: {error}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn explicit_nominal_casts_execute_once_with_slice_and_object_cleanup() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (name, expected) in [
        ("slice-once", "alloc 2\ndrop 3\ndrop 6\nfree\nvalue 9\n"),
        ("dyn", "alloc 1\ndrop 4\nfree\nvalue 4\n"),
    ] {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/coercion-casts/native/{name}")),
            &coercion_program(name, true)?,
            &root.join(CASTS).join("observe.c"),
            expected.as_bytes(),
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
