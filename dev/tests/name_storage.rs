use rustgm::{
    expand::config::Config,
    literal::{
        Value,
        storage::{Bytes, Storage, StoredValue},
    },
    resolve::{names::Names, unit::Unit},
    support::source::SourceFile,
};
use std::borrow::Cow;

#[test]
fn nfc_raw_names_and_lifetimes_share_dense_identity() -> Result<(), String> {
    let mut names = Names::default();
    let id = names.intern("é")?;
    assert_eq!(id, names.intern("e\u{301}")?);
    assert_eq!(id, names.intern("r#é")?);
    assert_eq!(id, names.intern("'é")?);
    assert_eq!(id, names.intern("'r#e\u{301}")?);
    assert_eq!(
        names.intern("각")?,
        names.intern("\u{1100}\u{1161}\u{11a8}")?
    );
    assert_eq!(names.len(), 2);
    for index in 0..10_000 {
        names.intern(&format!("name_{index}"))?;
    }
    let bytes = names.byte_len();
    for index in 0..10_000 {
        assert_eq!(names.get(names_id(index)?), format!("name_{index}"));
    }
    for index in 0..10_000 {
        names.intern(&format!("name_{index}"))?;
    }
    assert_eq!(names.len(), 10_002);
    assert_eq!(names.byte_len(), bytes);
    Ok(())
}
fn names_id(index: u32) -> Result<rustgm::resolve::names::NameId, String> {
    Ok(rustgm::resolve::names::NameId(
        index.checked_add(2).ok_or("overflow")?,
    ))
}
#[test]
fn literal_storage_rejects_foreign_and_invalid_ranges() -> Result<(), String> {
    let source = String::from("sample");
    let foreign = String::from("sample");
    let mut storage = Storage::default();
    assert!(
        storage
            .store(&source, &Value::String(Cow::Borrowed(foreign.as_bytes())))
            .is_err()
    );
    let value = storage.store(
        &source,
        &Value::String(Cow::Borrowed(&source.as_bytes()[1..4])),
    )?;
    assert_eq!(
        value,
        StoredValue::String(Bytes::Source { start: 1, end: 4 })
    );
    assert_eq!(
        storage.view(&source, value)?,
        Value::String(Cow::Borrowed(b"amp"))
    );
    let escaped = storage.store(&source, &Value::ByteString(Cow::Owned(vec![0, 255])))?;
    assert_eq!(storage.decoded_bytes(), 2);
    assert_eq!(
        storage.view(&source, escaped)?,
        Value::ByteString(Cow::Borrowed(&[0, 255]))
    );
    assert!(
        storage
            .view(
                &source,
                StoredValue::String(Bytes::Source { start: 1, end: 100 })
            )
            .is_err()
    );
    Ok(())
}
#[test]
fn moving_sources_preserves_plain_and_decoded_literals() -> Result<(), String> {
    let mut units = Vec::new();
    for _ in 0..100 {
        let source = SourceFile::new(b"fn f(){\"plain\";\"line\\n\";1_000.5;}".to_vec())?;
        let mut names = rustgm::support::names::Names::default();
        units.push(
            Unit::parse(source, &Config::default(), &mut names).map_err(|e| format!("{e:?}"))?,
        );
    }
    for unit in units {
        assert_eq!(
            unit.literal(0).map_err(|e| format!("{e:?}"))?,
            Value::String(Cow::Borrowed(b"plain"))
        );
        assert_eq!(
            unit.literal(1).map_err(|e| format!("{e:?}"))?,
            Value::String(Cow::Borrowed(b"line\n"))
        );
        assert_eq!(unit.literals.decoded_bytes(), 11);
        assert!(matches!(unit.literal(2), Ok(Value::Float(_))));
    }
    Ok(())
}

#[test]
fn parsed_names_match_span_interning_for_raw_nfc_and_lifetimes() -> Result<(), String> {
    use rustgm::syntax::ast::Kind;
    // Composed and decomposed spellings, raw identifiers and lifetimes share
    // one identity whether parsing or resolution interns them.
    let source = "fn f<'a, 'r#b>(r#é: &'a u8, e\u{301}: &'r#b u8) -> u8 { let r#x = *r#é + *e\u{301}; S { x: r#x }.x }";
    let mut names = Names::default();
    let crate_name = names.intern("crate")?;
    let unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec()).map_err(|e| format!("{e:?}"))?,
        &Config::default(),
        &mut names,
    )
    .map_err(|e| format!("{e:?}"))?;
    let mut checked = 0;
    for node in &unit.configured.ast.nodes {
        let (span, parsed) = match node.kind {
            Kind::Name(span, parsed) => (span, parsed),
            Kind::Binding { name, symbol, .. } => (name, symbol),
            _ => continue,
        };
        let parsed = parsed.ok_or("parsed name was not interned")?;
        let spelling = unit.spelling(span).map_err(|e| format!("{e:?}"))?;
        assert_eq!(parsed, names.intern(spelling)?, "{spelling}");
        checked += 1;
    }
    assert!(checked >= 10);
    assert_eq!(names.intern("é")?, names.intern("e\u{301}")?);
    assert_eq!(names.get(crate_name), "crate");
    Ok(())
}
