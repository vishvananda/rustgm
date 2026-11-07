//! Builtin macro rows travel in tag-14 version two with their module placement
//! and expander identity; declarative rows there carry allowance lists, and
//! re-export rows name a definition row by its owner and key. Version three
//! adds builtin allowance lists and every definition row's `#[unstable]`
//! feature, which gates each call in the consumer. Version four adds
//! semiopaque `pub macro` rows and every definition row's
//! `allow_internal_unsafe` flag; edition-panic contracts travel as the
//! `edition_panic` member of a list.
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
    rejected(&forge(18, 3), "unknown exported macro row kind")?;
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

const REEXPORTS: &str = "pa29/tests/spec/macro-reexport-rows";

/// (placement, name, owner, target placement, target name) of one re-export row.
type Reexport = (String, String, u32, String, String);
fn reexport_rows(interface: &crates::Interface) -> Result<Vec<Reexport>> {
    Ok(interface
        .macros()
        .ok_or("missing macro table")?
        .reexports()
        .iter()
        .map(|row| {
            (
                row.placement().join("::"),
                row.name().to_owned(),
                row.owner(),
                row.target_placement().join("::"),
                row.target_name().to_owned(),
            )
        })
        .collect())
}

#[test]
fn reexport_rows_bind_placements_to_definition_identities_in_version_two() -> Result {
    let packet = Packet::publish(GROUP, REEXPORTS, "reexports", "exporter", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes[12..14], [14, 2]);
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let macros = interface.macros().ok_or("missing macro table")?;
    let names: Vec<_> = macros
        .rows()
        .iter()
        .map(crates::exported_macros::Macro::name)
        .collect();
    assert_eq!(names, ["forty_two"]);
    // The hidden `concat` definition sits at its least public re-export key.
    let builtins: Vec<_> = macros
        .builtins()
        .iter()
        .map(|row| (row.placement().join("::"), row.name(), row.identity()))
        .collect();
    assert_eq!(
        builtins,
        [
            (String::new(), "cfg", "cfg"),
            (String::new(), "column", "column"),
            (String::new(), "line", "line"),
            (String::new(), "stringify", "stringify"),
            ("alternate".into(), "file", "file"),
            ("builtin".into(), "concat", "concat"),
            ("builtin".into(), "file", "file"),
        ]
    );
    let row = |placement: &str, name: &str, target: &str, target_name: &str| {
        (
            placement.to_owned(),
            name.to_owned(),
            0,
            target.to_owned(),
            target_name.to_owned(),
        )
    };
    let mut expected = vec![
        row("", "file", "builtin", "file"),
        row("", "spelled", "", "stringify"),
    ];
    for prelude in ["prelude::rust_2024", "prelude::v1"] {
        for (name, target) in [
            ("cfg", ""),
            ("column", ""),
            ("concat", "builtin"),
            ("file", "builtin"),
            ("forty_two", ""),
            ("line", ""),
            ("stringify", ""),
        ] {
            expected.push(row(prelude, name, target, name));
        }
    }
    assert_eq!(reexport_rows(&interface)?, expected);
    // A crate without re-exports keeps its version-one bytes.
    let (_, plain) = compile(
        "#[macro_export] macro_rules! plain { () => { 1 }; }",
        "plain",
    )?;
    assert_eq!(crates::encode(&plain)?[12..14], [14, 1]);
    Ok(())
}

/// A source client of `packet`'s provider that re-exports its rows, published
/// as `bridge.rgcrate`.
fn bridge(packet: &Packet) -> Result<std::path::PathBuf> {
    let source = client(
        packet,
        "bridge",
        "pub use base::builtin::concat;\npub use base::prelude::v1::line as here;",
    )?;
    let metadata = packet.folder.join("bridge.rgcrate");
    let mut command = packet.consumer(&source, "bridge", &packet.metadata, &packet.owner());
    command.arg("--metadata-output").arg(&metadata);
    super::artifacts::checked("bridge", &packet.folder, &mut command)?;
    Ok(metadata)
}
/// A client that reaches the provider only through `bridge` and the inventory.
fn bridged(
    packet: &Packet,
    label: &str,
    bridge: &std::path::Path,
) -> Result<std::process::Command> {
    let source = packet.folder.join(format!("{label}.rs"));
    std::fs::write(
        &source,
        "#![feature(no_core)]\n#![no_core]\nextern crate bridge;\npub fn f() -> u32 { let _t: &str = bridge::concat!(\"a\"); bridge::here!() }\n",
    )?;
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-lowir", "--crate-type=lib", "--crate-name", "client"])
        .args(["--crate-disambiguator", "bridged", "--source-client"])
        .arg(packet.owner())
        .arg("--extern")
        .arg(format!("bridge={}", bridge.display()))
        .arg("--dependency-metadata")
        .arg(&packet.metadata)
        .arg("-o")
        .arg(packet.folder.join(format!("{label}.lowir")))
        .arg(source);
    Ok(command)
}

#[test]
fn forged_and_relabelled_reexport_rows_are_rejected_by_name() -> Result {
    let packet = Packet::publish(GROUP, REEXPORTS, "forged-reexports", "exporter", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // The root `spelled` row: kind 2, no placement, owner 0, target `stringify`.
    let spelled =
        b"\x02\x00\x00\x07\x00\x00\x00spelled\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00stringify";
    unique(&bytes, spelled)?;
    let owned = b"spelled\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00stringify";
    rejected(
        &respelled(
            &bytes,
            owned,
            b"spelled\x01\x00\x00\x00\x00\x00\x09\x00\x00\x00stringify",
        )?,
        "exported macro re-export names an unknown owner",
    )?;
    rejected(
        &respelled(
            &bytes,
            owned,
            b"spelled\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00stringifz",
        )?,
        "exported macro re-export names no definition row of its crate",
    )?;
    // A target is a definition row, never another re-export.
    rejected(
        &respelled(
            &bytes,
            owned,
            b"spelled\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00file",
        )?,
        "exported macro re-export names no definition row of its crate",
    )?;
    let mut unknown = bytes.clone();
    unknown[unique(&bytes, spelled)?] = 3;
    rejected(&unknown, "unknown exported macro row kind")?;
    // A re-export renamed out of canonical order.
    rejected(
        &respelled(
            &bytes,
            b"\x02\x00\x00\x07\x00\x00\x00spelled",
            b"\x02\x00\x00\x07\x00\x00\x00stringz",
        )?,
        "exported macros must be unique and sorted by placement and name",
    )?;
    // A dependency's re-export row names its target by owner identity; a
    // target that the owner's table lacks fails in the consumer by name.
    let bridge = bridge(&packet)?;
    let published = std::fs::read(&bridge)?;
    assert_eq!(published[12..14], [14, 2]);
    let decoded = crates::decode(&published)?;
    assert_eq!(
        reexport_rows(&decoded)?,
        [
            (
                String::new(),
                "concat".into(),
                1,
                "builtin".into(),
                "concat".into()
            ),
            (
                String::new(),
                "here".into(),
                1,
                String::new(),
                "line".into()
            ),
        ]
    );
    super::artifacts::checked(
        "bridged",
        &packet.folder,
        &mut bridged(&packet, "bridged", &bridge)?,
    )?;
    let forged = packet.folder.join("bridge-forged.rgcrate");
    std::fs::write(
        &forged,
        respelled(
            &published,
            b"\x07\x00\x00\x00builtin\x06\x00\x00\x00concat",
            b"\x07\x00\x00\x00builtin\x06\x00\x00\x00concaz",
        )?,
    )?;
    packet.rejected(
        "bridged-forged",
        &mut bridged(&packet, "bridged-forged", &forged)?,
        "macro re-export names no imported definition row",
    )?;
    Ok(())
}

#[test]
fn reexport_row_metadata_grows_linearly_with_row_count() -> Result {
    let packet = Packet::new(GROUP, REEXPORTS, "reexport-scaling", "rows")?;
    let fixture = std::fs::read_to_string(packet.fixture("exporter"))?;
    let foundation = fixture
        .split("#[rustc_builtin_macro]")
        .next()
        .ok_or("missing provider foundation")?;
    let definition =
        "#[rustc_builtin_macro]\n#[macro_export]\nmacro_rules! stringify { ($($t:tt)*) => {} }\n";
    let (mut sizes, mut outputs) = (Vec::new(), Vec::new());
    for modules in [4, 16, 64] {
        let rows = (0..modules).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "pub mod m{index:03} {{ pub use crate::stringify; pub use crate::stringify as spelled; }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("rows{modules}.rs"));
        std::fs::write(&source, format!("{foundation}{definition}{rows}"))?;
        let metadata = packet.folder.join(format!("rows{modules}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let bytes = std::fs::read(&metadata)?;
        let interface = crates::decode(&bytes)?;
        let table = interface.macros().ok_or("missing macro table")?;
        assert_eq!(
            (table.builtins().len(), table.reexports().len()),
            (1, 2 * modules)
        );
        sizes.push(table_bytes(&bytes)?);
        let label = format!("client{modules}");
        let body = format!(
            "pub fn f() -> u32 {{ let _text: &str = base::m{:03}::spelled!(a); 1 }}",
            modules - 1
        );
        let source = client(&packet, &label, &body)?;
        let mut consumer = packet.consumer(&source, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        outputs.push(std::fs::read(packet.folder.join(format!("{label}.lowir")))?);
    }
    // Fixed-width module names: every re-export row adds the same table
    // bytes, and the consumer's output does not depend on the row count.
    let [small, middle, large] = <[usize; 3]>::try_from(sizes.as_slice())?;
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
    Ok(())
}

const STABILITY: &str = "pa29/tests/spec/macro-stability-rows";
const ANSWER_GATE: &str = "use of unstable library macro requires feature `gated_answer`";

/// A `no_core` client of `base` with extra crate features.
fn gated_client(
    packet: &Packet,
    label: &str,
    features: &str,
    body: &str,
) -> Result<std::path::PathBuf> {
    let source = packet.folder.join(format!("{label}.rs"));
    std::fs::write(
        &source,
        format!("#![feature(no_core{features})]\n#![no_core]\nextern crate base;\n{body}\n"),
    )?;
    Ok(source)
}

#[test]
fn stability_rows_and_builtin_lists_publish_in_version_three() -> Result {
    let packet = Packet::publish(GROUP, STABILITY, "stability", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes[12..14], [14, 3]);
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let macros = interface.macros().ok_or("missing macro table")?;
    let owned = |features: &[String]| features.join(",");
    let rows: Vec<_> = macros
        .rows()
        .iter()
        .map(|row| (row.name(), row.unstable(), owned(row.internal_unstable())))
        .collect();
    assert_eq!(
        rows,
        [
            ("answer", Some("gated_answer"), String::new()),
            ("checked", None, "cfg_ub_checks".into()),
            ("leaky", None, String::new()),
            ("line_wrapper", None, "gated_line".into()),
            ("wrapper", None, "gated_answer".into()),
        ]
    );
    let builtins: Vec<_> = macros
        .builtins()
        .iter()
        .map(|row| (row.name(), row.unstable(), owned(row.internal_unstable())))
        .collect();
    assert_eq!(
        builtins,
        [
            ("cfg", None, String::new()),
            ("line", Some("gated_line"), String::new()),
            ("stringify", None, "gated_answer".into()),
        ]
    );
    // The re-export names the unstable row; its own stability is not a row's.
    assert_eq!(
        reexport_rows(&interface)?,
        [(
            "reexported".into(),
            "answer".into(),
            0,
            String::new(),
            "answer".into()
        )]
    );
    // Calls through a path and a re-export need the feature; requesting it,
    // a library feature of the provider, admits both.
    let body = "pub fn f() -> u32 { base::reexported::answer!() }";
    let source = gated_client(&packet, "denied", "", body)?;
    let mut denied = packet.consumer(&source, "denied", &packet.metadata, &packet.owner());
    packet.rejected("denied", &mut denied, ANSWER_GATE)?;
    let source = gated_client(&packet, "requested", ", gated_answer", body)?;
    let mut requested = packet.consumer(&source, "requested", &packet.metadata, &packet.owner());
    super::artifacts::checked("requested", &packet.folder, &mut requested)?;
    // Version-two providers keep their exact bytes and version.
    let earlier = Packet::publish(GROUP, SPEC, "stability-v2", "provider", PROFILE)?;
    assert_eq!(std::fs::read(&earlier.metadata)?[12..14], [14, 2]);
    Ok(())
}

#[test]
fn forged_and_relabelled_stability_rows_are_rejected_by_name() -> Result {
    let packet = Packet::publish(GROUP, STABILITY, "forged-stability", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // The builtin `line` row: no helpers, an empty list, then tag 1 and its feature.
    let line = b"line\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x0a\x00\x00\x00gated_line";
    unique(&bytes, line)?;
    let tagged = b"\x00\x01\x0a\x00\x00\x00gated_line";
    rejected(
        &respelled(&bytes, tagged, b"\x00\x02\x0a\x00\x00\x00gated_line")?,
        "unknown exported macro stability",
    )?;
    rejected(
        &respelled(&bytes, tagged, b"\x00\x01\x0a\x00\x00\x00gated-line")?,
        "invalid exported macro unstable feature",
    )?;
    let listed = b"stringify\x00\x00\x00\x00\x00\x01\x00\x00\x00\x0c\x00\x00\x00gated-answer";
    rejected(
        &respelled(
            &bytes,
            b"stringify\x00\x00\x00\x00\x00\x01\x00\x00\x00\x0c\x00\x00\x00gated_answer",
            listed,
        )?,
        "invalid exported macro internal-unstable features",
    )?;
    // Without any version-three field the table must not claim version three.
    let mut plain = respelled(&bytes, tagged, b"\x00\x00")?;
    plain = respelled(
        &plain,
        b"stringify\x00\x00\x00\x00\x00\x01\x00\x00\x00\x0c\x00\x00\x00gated_answer",
        b"stringify\x00\x00\x00\x00\x00\x00\x00\x00\x00",
    )?;
    plain = respelled(
        &plain,
        b"\x00\x00\x00\x00\x01\x0c\x00\x00\x00gated_answer",
        b"\x00\x00\x00\x00\x00",
    )?;
    rejected(
        &plain,
        "version-three exported macro table has only version-two rows",
    )?;
    // A version-two reader of version-three rows misreads them, so a
    // relabelled table fails rather than dropping a gate.
    assert!(crates::decode(&super::fresh_packets::relabel(&bytes, 2)).is_err());
    // An expansion without a list does not admit the unstable row it calls,
    // and an unstable builtin row is gated like a declarative one.
    for (label, body, message) in [
        ("leaky", "pub fn f() -> u32 { base::leaky!() }", ANSWER_GATE),
        (
            "line",
            "pub fn f() -> u32 { base::line!() }",
            "use of unstable library macro requires feature `gated_line`",
        ),
    ] {
        let source = gated_client(&packet, label, "", body)?;
        let mut consumer = packet.consumer(&source, label, &packet.metadata, &packet.owner());
        packet.rejected(label, &mut consumer, message)?;
    }
    let body = "pub fn f() -> u32 { base::wrapper!() }\npub fn g() -> u32 { base::line_wrapper!() }\npub fn h() -> bool { base::checked!() }";
    let source = gated_client(&packet, "listed", "", body)?;
    let mut listed = packet.consumer(&source, "listed", &packet.metadata, &packet.owner());
    super::artifacts::checked("listed", &packet.folder, &mut listed)?;
    Ok(())
}

#[test]
fn stability_row_metadata_grows_linearly_with_row_count() -> Result {
    let packet = Packet::new(GROUP, STABILITY, "stability-scaling", "rows")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("#[rustc_builtin_macro]")
        .next()
        .ok_or("missing provider foundation")?;
    let (mut sizes, mut outputs) = (Vec::new(), Vec::new());
    for count in [8, 32, 128] {
        let rows = (0..count).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "#[macro_export]\n#[unstable(feature = \"g{index:03}\", issue = \"none\")]\nmacro_rules! u{index:03} {{ () => {{ 1 }} }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("rows{count}.rs"));
        std::fs::write(&source, format!("{foundation}{rows}"))?;
        let metadata = packet.folder.join(format!("rows{count}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let bytes = std::fs::read(&metadata)?;
        assert_eq!(bytes[12..14], [14, 3]);
        let interface = crates::decode(&bytes)?;
        let table = interface.macros().ok_or("missing macro table")?;
        assert_eq!(table.rows().len(), count);
        sizes.push(table_bytes(&bytes)?);
        let label = format!("client{count}");
        let last = count - 1;
        let features = format!(", g{last:03}");
        let body = format!("pub fn f() -> u32 {{ base::u{last:03}!() }}");
        let source = gated_client(&packet, &label, &features, &body)?;
        let mut consumer = packet.consumer(&source, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        outputs.push(std::fs::read(packet.folder.join(format!("{label}.lowir")))?);
    }
    // Fixed-width names and features: every row adds the same table bytes,
    // and the consumer's output does not depend on the row count.
    let [small, middle, large] = <[usize; 3]>::try_from(sizes.as_slice())?;
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
    Ok(())
}

const PANICS: &str = "pa29/tests/spec/imported-panic-rows";

/// The panic provider's crate features, lang items and nothing else.
fn panic_foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("#[rustc_builtin_macro]")
        .next()
        .ok_or("missing provider foundation")?
        .to_owned())
}

#[test]
fn panic_rows_publish_edition_lists_and_semiopaque_rows_in_version_four() -> Result {
    let packet = Packet::publish(GROUP, PANICS, "panic-rows", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes[12..14], [14, 4]);
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let macros = interface.macros().ok_or("missing macro table")?;
    let owned = |features: &[String]| features.join(",");
    let builtins: Vec<_> = macros
        .builtins()
        .iter()
        .map(|row| (row.name(), row.identity(), owned(row.internal_unstable())))
        .collect();
    assert_eq!(
        builtins,
        [
            ("assert", "assert", "edition_panic,panic_internals".into()),
            ("cfg", "cfg", String::new()),
            ("panic", "core_panic", "edition_panic".into()),
            ("unreachable", "unreachable", "edition_panic".into()),
        ]
    );
    let rows: Vec<_> = macros
        .rows()
        .iter()
        .map(|row| (row.name(), owned(row.internal_unstable())))
        .collect();
    assert_eq!(
        rows,
        [
            ("debug_assert", "edition_panic".into()),
            ("fail_with", "edition_panic".into()),
            ("plain_fail", String::new()),
        ]
    );
    let scoped: Vec<_> = macros
        .scoped()
        .iter()
        .map(|row| {
            let definition = row.definition();
            (
                row.placement().join("::"),
                row.name(),
                definition.unstable(),
                owned(definition.internal_unstable()),
                definition.rules(),
            )
        })
        .collect();
    let gated = Some("edition_panic");
    let listed = || "panic_internals".to_owned();
    assert_eq!(
        scoped,
        [
            ("panic".into(), "panic_2015", gated, listed(), 2),
            ("panic".into(), "panic_2021", gated, listed(), 2),
            ("panic".into(), "unreachable_2015", gated, listed(), 2),
            ("panic".into(), "unreachable_2021", gated, listed(), 2),
        ]
    );
    assert_eq!(
        macros.scoped()[1].definition().image(),
        "{ () => (41), ($($t:tt)+) => ($crate::panicking::formatted()), }"
    );
    let unsafe_rows = macros
        .rows()
        .iter()
        .map(crates::exported_macros::Macro::internal_unsafe);
    assert!(
        !unsafe_rows
            .chain(
                macros
                    .builtins()
                    .iter()
                    .map(crates::exported_macros::BuiltinMacro::internal_unsafe)
            )
            .any(|flag| flag)
    );
    // A fresh consumer runs each imported panic role, a skipped wrapper and
    // the semiopaque rule that names the provider's unstable function.
    let body = "pub fn a() -> u32 { base::panic!() }\npub fn b() -> u32 { base::unreachable!(\"x {}\", 1) }\npub fn c() -> u32 { base::fail_with!() }\npub fn d() -> u32 { base::panic!(\"x\") }";
    let source = client(&packet, "panics", body)?;
    let mut consumer = packet.consumer(&source, "panics", &packet.metadata, &packet.owner());
    super::artifacts::checked("panics", &packet.folder, &mut consumer)?;
    // rustc checks every single `use` path: an uncalled import of an unstable
    // row needs its feature, and a glob import gates only its module.
    let gate = "use of unstable library macro requires feature `edition_panic`";
    let source = client(&packet, "uncalled", "use base::panic::panic_2021;")?;
    let mut uncalled = packet.consumer(&source, "uncalled", &packet.metadata, &packet.owner());
    packet.rejected("uncalled", &mut uncalled, gate)?;
    let source = client(&packet, "globbed", "use base::panic::*;")?;
    let mut globbed = packet.consumer(&source, "globbed", &packet.metadata, &packet.owner());
    super::artifacts::checked("globbed", &packet.folder, &mut globbed)?;
    Ok(())
}

/// Core's `format_args` shape: a builtin row with `allow_internal_unsafe`
/// and a list.
const FORMAT_ARGS: &str = "#[rustc_builtin_macro]\n#[macro_export]\n#[allow_internal_unsafe]\n#[allow_internal_unstable(fmt_internals)]\n#[stable(feature = \"panics\", since = \"1.0.0\")]\nmacro_rules! format_args { ($fmt:expr) => {{}}; ($fmt:expr, $($args:tt)*) => {{}}; }\n";

#[test]
fn unsafe_rows_keep_their_flag_and_imported_format_args_stops_by_name() -> Result {
    let packet = Packet::publish(GROUP, PANICS, "unsafe-rows", "provider", PROFILE)?;
    // An `allow_internal_unsafe` row alone needs version four, and keeps its
    // flag, on a declarative and on a builtin row.
    let foundation =
        panic_foundation(&packet)?.replace("decl_macro", "decl_macro, allow_internal_unsafe");
    let unsafe_source = packet.folder.join("unsafe-provider.rs");
    std::fs::write(
        &unsafe_source,
        format!(
            "{foundation}#[macro_export]\n#[allow_internal_unsafe]\n#[stable(feature = \"panics\", since = \"1.0.0\")]\nmacro_rules! raw {{ () => {{ 1 }} }}\n{FORMAT_ARGS}"
        ),
    )?;
    let metadata = packet.folder.join("unsafe-provider.rgcrate");
    let mut export = packet.export(PROFILE, &unsafe_source, &metadata);
    super::artifacts::checked("unsafe-provider", &packet.folder, &mut export)?;
    let flagged = std::fs::read(&metadata)?;
    assert_eq!(flagged[12..14], [14, 4]);
    let flagged = crates::decode(&flagged)?;
    let table = flagged.macros().ok_or("missing macro table")?;
    assert!(table.rows()[0].internal_unsafe());
    let format = table
        .builtins()
        .iter()
        .find(|row| row.name() == "format_args")
        .ok_or("missing format_args row")?;
    assert!(format.internal_unsafe());
    assert_eq!(format.internal_unstable(), ["fmt_internals"]);
    // The imported row binds, but its expander builds typed fmt constructors
    // that need the provider's fmt items in this crate, so a call stops by name.
    let source = client(
        &packet,
        "format",
        "pub fn f() { base::format_args!(\"x\"); }",
    )?;
    let mut format = packet.consumer(&source, "format", &metadata, &packet.owner());
    packet.rejected(
        "format",
        &mut format,
        "imported builtin macro requires its provider's source declaration",
    )
}

#[test]
fn forged_and_relabelled_version_four_rows_are_rejected_by_name() -> Result {
    let packet = Packet::publish(GROUP, PANICS, "forged-panic-rows", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // The semiopaque `panic_2021` row: kind 3, placement `panic`, its name,
    // edition 2024 and no `local_inner_macros`.
    let head = b"\x03\x01\x00\x05\x00\x00\x00panic\x0a\x00\x00\x00panic_2021\x03\x00";
    let inner = b"\x03\x01\x00\x05\x00\x00\x00panic\x0a\x00\x00\x00panic_2021\x03\x01";
    rejected(
        &respelled(&bytes, head, inner)?,
        "semiopaque macro rows have no local_inner_macros",
    )?;
    // Its trailing flags byte follows its list and stability tag, just
    // before the next row, `unreachable_2015`.
    let next = b"\x03\x01\x00\x05\x00\x00\x00panic\x10\x00\x00\x00unreachable_2015";
    let tail = [&b"edition_panic\x00"[..], next].concat();
    let flags = [&b"edition_panic\x02"[..], next].concat();
    rejected(
        &respelled(&bytes, &tail, &flags)?,
        "unknown exported macro row flags",
    )?;
    // A version-three reader has no kind 3 and no flags; a relabelled table
    // fails rather than dropping rows.
    assert!(crates::decode(&super::fresh_packets::relabel(&bytes, 3)).is_err());
    // A placement respelled to another module is resolved by module identity
    // in a fresh consumer, never by a name, and fails by name.
    let moved = respelled(
        &bytes,
        b"\x05\x00\x00\x00panic\x10\x00\x00\x00unreachable_2021",
        b"\x05\x00\x00\x00panix\x10\x00\x00\x00unreachable_2021",
    )?;
    crates::decode(&moved)?;
    let forged = packet.folder.join("moved.rgcrate");
    std::fs::write(&forged, &moved)?;
    let source = client(&packet, "moved", "pub fn f() -> u32 { base::panic!() }")?;
    let mut consumer = packet.consumer(&source, "moved", &forged, &packet.owner());
    packet.rejected(
        "moved",
        &mut consumer,
        "exported macro placement has no public imported module",
    )?;
    // Without a version-four field the table must not claim version four.
    let foundation =
        panic_foundation(&packet)?.replace("decl_macro", "decl_macro, allow_internal_unsafe");
    let unsafe_source = packet.folder.join("unsafe-provider.rs");
    std::fs::write(
        &unsafe_source,
        format!(
            "{foundation}#[macro_export]\n#[allow_internal_unsafe]\n#[stable(feature = \"panics\", since = \"1.0.0\")]\nmacro_rules! raw {{ () => {{ 1 }} }}\n"
        ),
    )?;
    let metadata = packet.folder.join("unsafe-provider.rgcrate");
    let mut export = packet.export(PROFILE, &unsafe_source, &metadata);
    super::artifacts::checked("unsafe-provider", &packet.folder, &mut export)?;
    let flagged = std::fs::read(&metadata)?;
    // The sole row's flags byte is the last byte before the inner length.
    let mut cleared = flagged.clone();
    let at = table_bytes(&flagged)? + 18 - 1;
    assert_eq!(cleared[at], 1);
    cleared[at] = 0;
    rejected(
        &cleared,
        "version-four exported macro table has only version-three rows",
    )?;
    Ok(())
}

#[test]
fn semiopaque_row_metadata_grows_linearly_with_row_count() -> Result {
    let packet = Packet::new(GROUP, PANICS, "semiopaque-scaling", "rows")?;
    let foundation = panic_foundation(&packet)?;
    let (mut sizes, mut outputs) = (Vec::new(), Vec::new());
    for count in [8, 32, 128] {
        let rows = (0..count).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "#[stable(feature = \"panics\", since = \"1.0.0\")]\n#[rustc_macro_transparency = \"semiopaque\"]\npub macro s{index:03} {{ () => (1), }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("rows{count}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}#[stable(feature = \"panics\", since = \"1.0.0\")]\npub mod m {{\n{rows}}}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("rows{count}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let bytes = std::fs::read(&metadata)?;
        assert_eq!(bytes[12..14], [14, 4]);
        let interface = crates::decode(&bytes)?;
        let table = interface.macros().ok_or("missing macro table")?;
        assert_eq!(table.scoped().len(), count);
        sizes.push(table_bytes(&bytes)?);
        let label = format!("client{count}");
        let last = count - 1;
        let body = format!("pub fn f() -> u32 {{ base::m::s{last:03}!() }}");
        let source = client(&packet, &label, &body)?;
        let mut consumer = packet.consumer(&source, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        outputs.push(std::fs::read(packet.folder.join(format!("{label}.lowir")))?);
    }
    // Fixed-width names: every row adds the same table bytes, and the
    // consumer's output does not depend on the row count.
    let [small, middle, large] = <[usize; 3]>::try_from(sizes.as_slice())?;
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
    Ok(())
}
