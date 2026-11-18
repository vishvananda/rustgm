//! A generic body's transmute reads a parameterized nominal's projected
//! fields, as core's `AtomicPtr::new` does for `Atomic<*mut T>`. They take a
//! premise-free normal form. A field that needs a caller premise, or a body
//! premise that could keep the projection rigid, stays at the closed boundary.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/parameterized-storage";
const CLOSED: &str = "constant storage layout requires a closed normalized type";
const REJECTED: [(&str, &str); 6] = [
    ("param-storage.rs", CLOSED),
    (
        "size-mismatch.rs",
        "source transmute requires equal source and destination sizes",
    ),
    ("shadowed.rs", CLOSED),
    (
        "shadowed-value.rs",
        "body types require a provable associated-result equality",
    ),
    ("sibling-shadowed.rs", CLOSED),
    ("nested-shadowed.rs", CLOSED),
];
const NEW: &str = "impl<T> AtomicPtr<T> { pub const fn new(p: *mut T) -> AtomicPtr<T> { unsafe { transmute(p) } } }\n";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("parameterized storage outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid parameterized storage outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 15);
    Ok(cases)
}

#[test]
fn parameterized_projected_fields_size_transmutes_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, accepted) in cases()? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@parameterized_storage",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "parameterized_storage",
            ])
            .arg(root.join(&path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        let expected = if accepted { Some(0) } else { Some(1) };
        if output.status.code() != expected
            || accepted != reason.is_none()
            || reason.is_some_and(|reason| !stderr.contains(reason))
        {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn transmute_of_thin_boxes_uses_the_sizing_contract_of_projected_pointees() -> Result {
    let source =
        crate::resolve::fixture("pa31/tests/spec/explicit-dynamic-call-members/prefix.rs")?;
    let (prefix, _) = source
        .split_once("pub struct Noisy")
        .ok_or("owned Box prefix")?;
    let prefix = prefix.replace(
        "pub const fn size_of<T>()->usize;}",
        concat!(
            "pub const fn size_of<T>()->usize;\n",
            "#[rustc_intrinsic] pub const unsafe fn transmute<Src, Dst>(src: Src) -> Dst;}",
        ),
    );
    let declarations = r"
        pub trait HasOutput { type Output; }
        pub trait HasWideOutput { type Output: ?Sized; }
        pub struct Payload<T>(T);
        pub struct ProjectionField<R: HasOutput>(R::Output);
    ";
    for (bound, from, to, accepted) in [
        (
            "HasOutput",
            "Box<Payload<T>>",
            "Box<Payload<R::Output>>",
            true,
        ),
        ("HasOutput", "*const T", "*const R::Output", true),
        ("HasWideOutput", "Box<T>", "Box<R::Output>", false),
        ("HasOutput", "T", "R::Output", false),
    ] {
        let text = format!(
            "{prefix}{declarations}
             pub unsafe fn convert<T, R: {bound}>(value: {from}) -> {to} {{
                 unsafe {{ intrinsics::transmute(value) }}
             }}"
        );
        let report = crate::resolve::source_provider(&text, "projected_pointee_storage")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if accepted {
            result?;
        } else {
            let failure = result
                .err()
                .ok_or("accepted a dependently sized transmute")?;
            assert_eq!(failure.message, CLOSED, "{from} to {to}");
        }
    }
    Ok(())
}

/// `calls` generic transmutes into their own parameterized `Atomic`, then
/// unrelated nominals with their own projected fields that no body reads.
fn width(prefix: &str, calls: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub const fn n{index}<T>(p: *mut T) -> AtomicPtr<T> {{ unsafe {{ transmute(p) }} }}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "#[repr(C)] pub struct U{index}<T: AtomicPrimitive> {{ v: T::Storage }}"
        )?;
    }
    Ok(text)
}

#[test]
fn parameterized_storage_tracks_transmutes_without_scanning_unrelated_nominals() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/ptr-new.rs"))?;
    let prefix = source.strip_suffix(NEW).ok_or("AtomicPtr::new")?;
    let mut previous = None;
    let mut unrelated = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report = crate::resolve::source_provider(
                &width(prefix, calls, noise)?,
                "parameterized_storage",
            )?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("parameterized storage calls={calls} noise={noise} work={work:?}");
            if noise {
                // The unrelated nominals publish their own templates once; no
                // transmute normalizes or lays them out.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 512,
                            "calls={calls}: {before:?}->{work:?}"
                        );
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
