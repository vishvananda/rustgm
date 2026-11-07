//! Builtin macro rows travel in tag-14 version two with their module placement
//! and expander identity; declarative rows there carry allowance lists.
use super::{
    exports::compile,
    fresh_packets::{Packet, Result, unique},
};
use rustgm::support::crates::{
    self,
    exported_macros::{MacroKind, Syntax},
};
use std::fmt::Write as _;

const GROUP: &str = "pa29-builtin-rows";
const SPEC: &str = "pa29/tests/spec/builtin-macro-rows";
const PROFILE: &str = "module-bodies-v13";
const ALLOWED: &str = "#![feature(allow_internal_unstable)]\n#[macro_export]\n#[allow_internal_unstable(cfg_ub_checks)]\nmacro_rules! allowed { () => { 1 }; }\n";

fn rejected(bytes: &[u8], message: &str) -> Result {
    let Err(error) = crates::decode(bytes) else {
        return Err(format!("forged builtin rows were accepted: {message}").into());
    };
    assert!(error.contains(message), "{error} != {message}");
    Ok(())
}
/// The encoded bytes with the unique run `from` replaced by `to`.
fn respelled(bytes: &[u8], from: &[u8], to: &[u8]) -> Result<Vec<u8>> {
    let at = unique(bytes, from)?;
    let mut copy = bytes.to_vec();
    copy.splice(at..at + from.len(), to.iter().copied());
    Ok(copy)
}
fn client(packet: &Packet, label: &str, body: &str) -> Result<std::path::PathBuf> {
    let source = packet.folder.join(format!("{label}.rs"));
    std::fs::write(
        &source,
        format!("#![feature(no_core)]\n#![no_core]\nextern crate base;\n{body}\n"),
    )?;
    Ok(source)
}
/// Bytes of the version-two row table: after the 18-byte header and count,
/// up to the length prefix of the wrapped inner artifact.
fn table_bytes(bytes: &[u8]) -> Result<usize> {
    let inner = bytes
        .windows(8)
        .skip(8)
        .position(|window| window == b"RGMETA\r\n")
        .ok_or("missing wrapped artifact")?;
    Ok(inner + 8 - 4 - 18)
}

#[test]
fn builtin_rows_publish_placements_and_identities_in_version_two() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "publish", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes[12..14], [14, 2]);
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let macros = interface.macros().ok_or("missing macro table")?;
    assert!(macros.rows().is_empty());
    let rows: Vec<_> = macros
        .builtins()
        .iter()
        .map(|row| {
            assert!(row.helpers().is_empty());
            (
                row.placement().join("::"),
                row.name(),
                row.syntax(),
                row.identity(),
                row.kind(),
            )
        })
        .collect();
    let root = |name| {
        (
            String::new(),
            name,
            Syntax::MacroRules,
            name,
            MacroKind::Bang,
        )
    };
    let placed = |path: &str, name, kind| (path.to_owned(), name, Syntax::Macro, name, kind);
    assert_eq!(
        rows,
        [
            root("cfg"),
            root("column"),
            root("compile_error"),
            root("line"),
            root("stringify"),
            placed("builtin", "concat", MacroKind::Bang),
            placed("builtin", "file", MacroKind::Bang),
            placed("builtin::deeper", "module_path", MacroKind::Bang),
            placed("builtin::deeper", "test", MacroKind::Attribute),
        ]
    );
    // Declarative rows carry sorted, unique allowance lists, and only those
    // lists move an ordinary crate to version two.
    let internal = std::fs::read_to_string(packet.fixture("internal-provider"))?;
    let (_, ordinary) = compile(&internal, "internal")?;
    let encoded = crates::encode(&ordinary)?;
    assert_eq!(encoded[12..14], [14, 2]);
    assert_eq!(crates::decode(&encoded)?, ordinary);
    let lists: Vec<_> = ordinary
        .macros()
        .ok_or("missing ordinary macro table")?
        .rows()
        .iter()
        .map(|row| (row.name(), row.internal_unstable().to_vec()))
        .collect();
    assert_eq!(
        lists,
        [
            (
                "checked",
                vec!["cfg_target_has_atomic".to_owned(), "cfg_ub_checks".into()]
            ),
            ("partial", vec!["cfg_ub_checks".to_owned()]),
            ("unchecked", Vec::new()),
        ]
    );
    let (_, plain) = compile(
        "#[macro_export] macro_rules! plain { () => { 1 }; }",
        "plain",
    )?;
    assert_eq!(crates::encode(&plain)?[12..14], [14, 1]);
    Ok(())
}

#[test]
fn forged_and_relabelled_builtin_rows_are_rejected_by_name() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "forged", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // Row one: builtin kind, empty placement, `cfg`, macro_rules, `cfg`, bang.
    assert_eq!(bytes[18..25], [1, 0, 0, 3, 0, 0, 0]);
    assert_eq!(
        bytes[25..41],
        *b"cfg\x00\x03\x00\x00\x00cfg\x00\x00\x00\x00\x00"
    );
    let forge = |offset: usize, value: u8| {
        let mut copy = bytes.clone();
        copy[offset] = value;
        copy
    };
    rejected(&forge(18, 2), "unknown exported macro row kind")?;
    rejected(&forge(28, 2), "unknown exported builtin macro syntax")?;
    rejected(&forge(36, 3), "unknown exported builtin macro kind")?;
    rejected(
        &respelled(&bytes, b"column\x00\x06", b"aolumn\x00\x06")?,
        "exported macros must be unique and sorted by placement and name",
    )?;
    rejected(
        &respelled(&bytes, b"concat\x01", b"concat\x00")?,
        "exported builtin macro_rules rows belong to the crate root",
    )?;
    rejected(
        &respelled(
            &bytes,
            b"cfg\x00\x00\x00\x00\x00",
            b"cfg\x00\x01\x00\x00\x00\x01\x00\x00\x00x",
        )?,
        "only builtin derive rows declare helper attributes",
    )?;
    // Declarative rows stay at the root, and a version-two table must need it.
    let (_, allowed) = compile(ALLOWED, "allowed")?;
    let allowed = crates::encode(&allowed)?;
    assert_eq!(allowed[18..21], [0, 0, 0]);
    let mut placed = allowed.clone();
    placed.splice(19..21, [1, 0, 1, 0, 0, 0, b'm']);
    rejected(
        &placed,
        "declarative exported macros belong to the crate root",
    )?;
    rejected(
        &respelled(
            &allowed,
            b"\x01\x00\x00\x00\x0d\x00\x00\x00cfg_ub_checks",
            b"\x00\x00\x00\x00",
        )?,
        "version-two exported macro table has only version-one rows",
    )?;
    rejected(
        &respelled(&allowed, b"cfg_ub_checks", b"cfg-ub-checks")?,
        "invalid exported macro internal-unstable features",
    )?;
    // Valid rows naming an unknown expander or another kind fail in the consumer.
    let source = client(&packet, "line", "pub fn f() -> u32 { base::line!() }")?;
    for (label, from, to, message) in [
        (
            "unknown",
            &b"line\x00\x04\x00\x00\x00line"[..],
            &b"line\x00\x04\x00\x00\x00lime"[..],
            "imported builtin macro `line` names unknown expander `lime`",
        ),
        (
            "relabelled",
            &b"line\x00\x04\x00\x00\x00line\x00"[..],
            &b"line\x00\x04\x00\x00\x00line\x01"[..],
            "imported builtin macro `line` changed the macro kind of expander `line`",
        ),
    ] {
        let forged = packet.folder.join(format!("{label}.rgcrate"));
        std::fs::write(&forged, respelled(&bytes, from, to)?)?;
        let mut consumer = packet.consumer(&source, label, &forged, &packet.owner());
        packet.rejected(label, &mut consumer, message)?;
    }
    let mut accepted = packet.consumer(&source, "accepted", &packet.metadata, &packet.owner());
    super::artifacts::checked("accepted", &packet.folder, &mut accepted)?;
    Ok(())
}

#[test]
fn builtin_row_metadata_grows_linearly_with_row_count() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "rows")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("#[rustc_builtin_macro]")
        .next()
        .ok_or("missing provider foundation")?;
    let (mut sizes, mut outputs) = (Vec::new(), Vec::new());
    for modules in [4, 16, 64] {
        let rows = (0..modules).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "pub mod m{index:03} {{ #[rustc_builtin_macro] pub macro concat($($t:tt)*) {{}} #[rustc_builtin_macro] pub macro stringify($($t:tt)*) {{}} }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("rows{modules}.rs"));
        std::fs::write(&source, format!("{foundation}{rows}"))?;
        let metadata = packet.folder.join(format!("rows{modules}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let bytes = std::fs::read(&metadata)?;
        let interface = crates::decode(&bytes)?;
        let table = interface.macros().ok_or("missing macro table")?;
        assert_eq!(table.builtins().len(), 2 * modules);
        sizes.push(table_bytes(&bytes)?);
        // The last placement binds through its own module identity.
        let label = format!("client{modules}");
        let body = format!(
            "pub fn f() -> u32 {{ let _text: &str = base::m{:03}::concat!(\"a\"); 1 }}",
            modules - 1
        );
        let source = client(&packet, &label, &body)?;
        let mut consumer = packet.consumer(&source, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        outputs.push(std::fs::read(packet.folder.join(format!("{label}.lowir")))?);
    }
    // Fixed-width module names: every row adds the same table bytes, and the
    // consumer's output does not depend on how many rows it imported.
    let [small, middle, large] = <[usize; 3]>::try_from(sizes.as_slice())?;
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
    Ok(())
}
