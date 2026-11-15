//! rustc derives from the configured item and clears every generic
//! parameter's attributes in the impl it writes (`create_derived_impl`), as
//! alloc's derives over `#[unstable(..)] A: Allocator = Global` need.
//!
//! Each case appends its items to the `probe` module of the
//! `derive-projection-bounds` source core. Every outcome was declared before
//! either compiler ran; rustc 1.98.0 is the oracle (`RUSTC_BOOTSTRAP=1 rustc
//! +1.98.0 --edition=2024 --crate-type=lib --crate-name minicore
//! --cap-lints=allow`). A rejection names the text whose last occurrence is
//! its site.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/derive-projection-bounds";
const SUFFIX: &str = "    #[derive(Clone, Debug)]
    pub struct Wrap<T> { value: T }
    pub fn again(value: &Wrap<u8>) -> Wrap<u8> { Clone::clone(value) }
}
";

/// One case: its items, appended to `probe`, and its declared rejection.
struct Case {
    name: &'static str,
    items: &'static str,
    rejected: Option<Rejection>,
}
/// A rejection's message prefix and the text whose last occurrence is its site.
struct Rejection {
    message: &'static str,
    site: &'static str,
}
/// A first `const N: usize` case was declared accepted, but both compilers
/// reject it: this core has no `ConstParamTy_` impl for `usize` (rustc E0741).
/// rustc: the three accepted cases compile; `bound-kept` fails with E0277
/// (`Plain: Clone` is not satisfied by the derived impl's bound) and
/// `removed-argument` with E0107 (`Gone` takes one generic argument, since
/// `cfg` removed the other). That case's message was pinned after its first
/// run; its rejection and site were declared before.
const CASES: [Case; 5] = [
    Case {
        name: "lint-attribute",
        items: "    #[derive(Clone, Debug)]
    pub struct Held<#[allow(unused)] T> { value: T }
    pub fn again(value: &Held<u8>) -> Held<u8> { Clone::clone(value) }
",
        rejected: None,
    },
    Case {
        name: "lifetime-attribute",
        items: "    pub struct Token;
    impl<'a> Clone for &'a Token { fn clone(&self) -> Self { loop {} } }
    #[derive(Clone)]
    pub struct Seen<#[allow(unused)] 'a, T> { value: T, token: &'a Token }
    pub fn again<'a>(value: &Seen<'a, u8>) -> Seen<'a, u8> { Clone::clone(value) }
",
        rejected: None,
    },
    Case {
        name: "configured-out",
        items: "    #[derive(Clone, Debug)]
    pub struct Gone<#[cfg(any())] U, T> { value: T }
    pub fn again(value: &Gone<u8>) -> Gone<u8> { Clone::clone(value) }
",
        rejected: None,
    },
    Case {
        name: "bound-kept",
        items: "    pub struct Plain;
    #[derive(Clone)]
    pub struct Held<#[allow(unused)] T> { value: T }
    pub fn again(value: &Held<Plain>) -> Held<Plain> { Clone::clone(value) }
",
        rejected: Some(Rejection {
            message: "trait obligation requires an unambiguous matching assumption or implementation",
            site: "Clone::clone(value)",
        }),
    },
    Case {
        name: "removed-argument",
        items: "    #[derive(Clone)]
    pub struct Gone<#[cfg(any())] U, T> { value: T }
    pub fn again(value: &Gone<u8, u8>) -> u8 { 0 }
",
        rejected: Some(Rejection {
            message: "too many generic arguments",
            site: "Gone<u8, u8>",
        }),
    },
];

/// The CLI's typing status and its stderr.
fn typed(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@derive_attributes",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "derive_attributes",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Attributes of a derived item's parameters never reach its impl, a
/// parameter that `cfg` removed is not one of its formals, and the derived
/// bound still applies to an attributed parameter.
#[test]
fn derived_impls_drop_parameter_attributes_like_rustc() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/sole-parameter.rs"))?;
    let prefix = source.strip_suffix(SUFFIX).ok_or("derive fixture suffix")?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/cleared-parameter-attributes")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for Case {
        name,
        items,
        rejected,
    } in CASES
    {
        let text = format!("{prefix}{items}}}\n");
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &text)?;
        let (types, stderr) = typed(&path)?;
        let matches = match rejected {
            None => types,
            Some(Rejection { message, site }) => {
                let offset = text.rfind(site).ok_or("derive attribute site text")?;
                !types && stderr.starts_with(&format!("ERROR: file 0: {offset}: {message}"))
            }
        };
        if !matches {
            failures.push(format!("{name}: types={types} {stderr}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
