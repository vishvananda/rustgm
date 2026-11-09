//! Root `#[macro_export]` definitions publish authenticated canonical token rows.
use super::exports::compile;
use rustgm::support::crates::{
    self,
    exported_macros::{Delimiter, Macro, Row, TokenClass},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa29/tests/spec/macro-metadata")
            .join(name),
    )?)
}

/// Byte offsets of the first macro row's fields in an encoded macro envelope.
struct Layout {
    edition: usize,
    flags: usize,
    image: usize,
    rows: usize,
    inner: usize,
}
fn field(bytes: &[u8], at: usize) -> Result<usize> {
    let word: [u8; 4] = bytes.get(at..at + 4).ok_or("truncated")?.try_into()?;
    Ok(u32::from_le_bytes(word) as usize)
}
fn layout(bytes: &[u8]) -> Result<Layout> {
    assert_eq!(bytes[12..14], [14, 1]);
    let count = field(bytes, 14)?;
    let mut at = 18;
    let mut first = None;
    for _ in 0..count {
        let name = field(bytes, at)?;
        let edition = at + 4 + name;
        let image = edition + 6;
        let rows = image + 4 + field(bytes, image)?;
        first.get_or_insert(Layout {
            edition,
            flags: edition + 1,
            image,
            rows,
            inner: 0,
        });
        at = rows + 4 + 5 * field(bytes, rows)?;
    }
    let mut first = first.ok_or("no macro rows")?;
    first.inner = at;
    Ok(first)
}
fn rejected(bytes: &[u8], message: &str) -> Result {
    let Err(error) = crates::decode(bytes) else {
        return Err(format!("forged macro metadata was accepted: {message}").into());
    };
    assert!(error.contains(message), "{error} != {message}");
    Ok(())
}

#[test]
fn root_macro_exports_publish_canonical_rows_and_macro_free_crates_keep_bytes() -> Result {
    // Wire codes are the declaration order of the closed vocabularies.
    for (index, class) in TokenClass::ALL.iter().enumerate() {
        assert_eq!(*class as usize, index);
    }
    for (index, delimiter) in Delimiter::ALL.iter().enumerate() {
        assert_eq!(*delimiter as usize, index);
    }
    let (_, interface) = compile(&fixture("provider.rs")?, "macros")?;
    let macros = interface.macros().ok_or("missing exported macros")?;
    let names: Vec<_> = macros.rows().iter().map(Macro::name).collect();
    assert_eq!(names, ["call_f", "nested_export", "select"]);
    let call = macros.get("call_f").ok_or("missing call_f")?;
    // Amended: the image now keeps the source adjacency of the definition
    // `macro_rules! call_f { () => { $crate::f() }; }`.
    assert_eq!(call.image(), "{ () => { $crate::f() }; }");
    assert_eq!(call.rules(), 1);
    assert!(!call.local_inner_macros());
    assert_eq!(
        call.rows()[..3],
        [
            Row::Group {
                delimiter: Delimiter::Brace,
                rows: 9
            },
            Row::Group {
                delimiter: Delimiter::Parenthesis,
                rows: 0
            },
            Row::Token {
                class: TokenClass::Punctuation,
                length: 2
            },
        ]
    );
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(decoded, interface);
    assert_eq!(crates::encode(&decoded)?, bytes);
    // An unexported macro_rules changes nothing in the published artifact.
    let (_, plain) = compile("pub fn f()->u8{40}", "plain")?;
    let (_, local) = compile("macro_rules! m{()=>{40}}pub fn f()->u8{m!()}", "plain")?;
    assert!(plain.macros().is_none() && local.macros().is_none());
    assert_eq!(crates::encode(&plain)?, crates::encode(&local)?);
    assert_ne!(crates::encode(&plain)?[12], 14);
    Ok(())
}

#[test]
fn forged_relabelled_and_nested_macro_rows_are_rejected_by_name() -> Result {
    let (_, interface) = compile(&fixture("provider.rs")?, "forged")?;
    let bytes = crates::encode(&interface)?;
    let at = layout(&bytes)?;
    let forge = |offset: usize, value: u8| {
        let mut copy = bytes.clone();
        copy[offset] = value;
        copy
    };
    // Amended for the version-two row table, then for versions three, four
    // and five (glob rows): versions 2 to 5 are now known, so the
    // unknown-version control uses 6, and v1 rows relabelled as any placed
    // version fail by name.
    rejected(&forge(13, 6), "unknown exported macro metadata version")?;
    rejected(&forge(13, 2), "unknown exported macro row kind")?;
    rejected(&forge(13, 3), "unknown exported macro row kind")?;
    rejected(&forge(13, 4), "unknown exported macro row kind")?;
    rejected(&forge(13, 5), "unknown exported macro row kind")?;
    rejected(&forge(at.edition, 4), "unknown exported macro edition")?;
    rejected(&forge(at.flags, 2), "unknown exported macro flags")?;
    rejected(
        &forge(at.rows + 4, 0x83),
        "unknown exported macro delimiter",
    )?;
    rejected(
        &forge(at.rows + 9, 15),
        "unknown exported macro token class",
    )?;
    // Relabel the empty matcher group as non-empty, and respell the root brace.
    rejected(
        &forge(at.rows + 10, 1),
        "exported macro rows disagree with their canonical image",
    )?;
    rejected(
        &forge(at.image + 4, b'('),
        "exported macro rows disagree with their canonical image",
    )?;
    rejected(&forge(at.flags + 1, 0), "exported macro has no rules")?;
    let mut trailing = bytes.clone();
    trailing.push(0);
    rejected(&trailing, "trailing or nested exported macro metadata")?;
    let mut nested = bytes[..at.inner].to_vec();
    nested.extend_from_slice(&u32::try_from(bytes.len())?.to_le_bytes());
    nested.extend_from_slice(&bytes);
    rejected(&nested, "trailing or nested exported macro metadata")?;
    let mut truncated = bytes.clone();
    truncated.truncate(at.inner);
    assert!(crates::decode(&truncated).is_err());
    Ok(())
}

/// The generated definition of `pa29/tests/macro-context-outcomes.json`'s
/// ordinary-path control, located at `macro_rules! made` inside `make!`'s body.
const GENERATED: &str =
    "macro_rules! make{()=>{#[macro_export] macro_rules! made{()=>{1}}}}make!{}";
#[test]
fn generated_documented_and_permissioned_macro_exports_fail_by_name() -> Result {
    let Err(error) = compile(GENERATED, "generated") else {
        return Err("a macro-generated exported macro was published".into());
    };
    assert_eq!(
        error.to_string(),
        "file 0: 39: macro-generated exported macros are outside the macro metadata profile\n  \
         in macro make at file 0: 67; defined at file 0: 0"
    );
    let text = "#[macro_export] macro_rules! documented{()=>{\n/// doc\npub fn f(){}}}";
    let message = "doc comments in exported macro bodies are outside the macro metadata profile";
    let Err(error) = compile(text, "rejected") else {
        return Err(format!("unsupported macro export was published: {text}").into());
    };
    assert!(error.to_string().contains(message), "{error}");
    Ok(())
}
