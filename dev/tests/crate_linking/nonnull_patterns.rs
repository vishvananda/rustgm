//! Fresh CLI processes re-prove version-thirteen pattern types and `NonNull`'s
//! diagnostic identity. A pattern row carries only its base and its
//! constraint, `!null` or a closed interval of raw bits; every consumer
//! re-proves it through its own interning and derives its own layout.
use super::fresh_packets::{
    Packet, Result, declaration, difference, inner, rewrapped, role_column, role_copies, spliced,
    unique, with_id,
};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/nonnull-patterns";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "nonnull-patterns";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/nonnull-pattern-outcomes.json`: each
/// accepted client whose `Opt` has a carrier, and its `o`'s size and
/// alignment. Each is rustc's, except holder's, a recorded divergence: rustc
/// places the niche through a non-transparent struct's field for 8x8, while
/// the local nullable-carrier walk descends only `repr(transparent)` structs
/// and gives the tagged 16x8. The pin fails when field-niche placement lands
/// and then flips to the declared 8x8.
const ACCEPTED: [(&str, u32, u32); 4] = [
    ("nonnull", 8, 8),
    ("nonnull-str", 16, 8),
    ("holder", 16, 8),
    ("nonzero-u8", 1, 1),
];
/// Clients rustc accepts that wait for the local niche ABI profile: only
/// `!null` pointers and zero-excluded unsigned ranges have nullable carriers.
/// Their fresh consumers and single-crate controls reject alike.
const PENDING: [&str; 4] = ["nanoseconds", "nonzero-char", "code-point", "percent"];
const NICHE: &str = "scalar pattern enum niches require their complete ABI profile";
/// Version-thirteen module-body export has no fn-item-to-fn-pointer
/// reification row, so a witness's single-crate control waits for it.
const REIFY: &str = "source rvalue requires a complete wider transport row";
/// Each range type and rustc's size and alignment of it, witnessed by the
/// by-value slot of a function whose address the entry takes.
const WITNESSES: [(&str, u32, u32); 5] = [
    ("Nanoseconds", 4, 4),
    ("NonZeroCharInner", 4, 4),
    ("CodePointInner", 4, 4),
    ("NonZeroU8", 1, 1),
    ("Percent", 1, 1),
];
const OPPONENTS: [(&str, &str); 2] = [
    ("private", "aggregate field is private"),
    ("mismatch", "incompatible types"),
];
/// Older profiles reject the provider's tuple structs before any pattern.
const UNIT: &str = "source declaration kind is outside the initial source profile";
const GATED: &str = "source NonNull diagnostic identity transport requires version 13";
const PARAMETER: &str = "source non-null pattern over a type parameter is not transported";
const NON_NULL: &str = "imported non-null pattern requires a raw pointer base";
const INVALID: &str = "scalar pattern endpoint is invalid for its actual base";
const UNSIGNED: &str = "scalar alternatives require a signed integer base";
const GATE: &str = "// Generated NonNull diagnostic gate.\n#[rustc_diagnostic_item = \"NonNull\"]\npub struct NonNull<T> { value: T }\n";
const FIELD: &str =
    "// Generated parameter base fence.\npub struct Field<T>(pattern_type!(T is !null));\n";
/// The diagnostic identity row: tag 254, the spelling's length and bytes,
/// then the declaration address.
const DIAGNOSTIC: &[u8] = b"\xfe\x07\x00\x00\x00NonNull";
const DIAGNOSTIC_ROW: usize = DIAGNOSTIC.len() + 8;
/// Each kind-1 row's provider type and its canonical interval.
const RANGES: [(&str, u128, u128); 5] = [
    ("Nanoseconds", 0, 999_999_999),
    ("NonZeroCharInner", 1, 0x0010_ffff),
    ("CodePointInner", 0, 0x0010_ffff),
    ("NonZeroU8", 1, 255),
    ("Percent", 0, 100),
];
fn source(packet: &Packet, name: &str, text: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}
fn export(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut command = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut command).map(drop)
}
/// The provider's lines before its authored section, and its `pattern_type`
/// macro declaration.
fn foundation(packet: &Packet) -> Result<(String, String)> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let (lines, authored) = fixture
        .split_once("// Authored pattern-type and NonNull transport controls.\n")
        .ok_or("pattern foundation")?;
    let (macro_lines, _) = authored
        .split_once("// Core's `NonNull` declaration")
        .ok_or("pattern macro")?;
    Ok((lines.to_owned(), macro_lines.to_owned()))
}
fn entry(packet: &Packet, name: &str, body: &str) -> Result<std::path::PathBuf> {
    source(
        packet,
        &format!("{name}.rs"),
        &format!(
            "#![feature(no_core)]\n#![no_core]\nextern crate base;\nenum Opt<T> {{ None, Some(T) }}\n#[unsafe(no_mangle)]\n{body}\n"
        ),
    )
}
/// The text of one `LowIR` function, through its closing brace.
fn function(lowir: &str, name: &str) -> Result<String> {
    let start = lowir
        .find(&format!("function @{name}("))
        .ok_or("LowIR function")?;
    let length = lowir[start..].find("\n}\n").ok_or("LowIR function end")?;
    Ok(lowir[start..start + length + 2].to_owned())
}
/// A layout witness entry: `hold` is lowered only through its address.
fn witness(name: &str) -> String {
    format!(
        "pub extern \"C\" fn entry() -> i32 {{ let _f: fn(base::{name}) -> i32 = hold; 42 }}\nfn hold(_v: base::{name}) -> i32 {{ 42 }}"
    )
}
/// A witness entry and the function whose address it stores, renamed `entry`
/// and `hold`, after checking that function's by-value slot.
fn held(lowir: &str, entry: &str, size: u32, align: u32) -> Result<String> {
    let text = function(lowir, entry)?.replacen(&format!("@{entry}("), "@entry(", 1);
    let target = text
        .split("addr @")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or("witness address")?;
    let hold = function(lowir, target)?;
    let slot = hold
        .lines()
        .find(|line| line.trim_start().starts_with("slot "))
        .ok_or("witness slot")?;
    assert!(
        slot.ends_with(&format!(": obj<{size}x{align}>")),
        "{entry}: {slot}"
    );
    Ok(format!("{text}{hold}").replace(target, "hold"))
}
/// The wrapped artifact's offset: the provider exports its `pattern_type`
/// macro, so its packet is a declarative macro envelope.
fn id_at(bytes: &[u8], offset: usize) -> Result<usize> {
    let value = bytes.get(offset..offset + 4).ok_or("recipe id")?;
    Ok(usize::try_from(u32::from_le_bytes(value.try_into()?))?)
}
/// One type row's length from its tag, for the rows this provider exports.
/// Function-pointer recipes and unknown tags end the parse.
fn type_row(bytes: &[u8], offset: usize) -> Option<usize> {
    Some(match *bytes.get(offset)? {
        0 => 2,
        10 | 11 => 1,
        1 | 4 | 5 | 7 | 8 | 13 => 5,
        12 => 6,
        9 => 9,
        2 => 10,
        6 => 17,
        16 => match *bytes.get(offset + 5)? {
            0 => 6,
            1 => 38,
            _ => return None,
        },
        _ => return None,
    })
}
/// The type rows' offsets, if a type column's count is at `offset`.
fn type_rows(bytes: &[u8], offset: usize) -> Option<Vec<usize>> {
    let count = u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    if !(1..=4096).contains(&count) {
        return None;
    }
    let mut rows = Vec::new();
    let mut cursor = offset + 4;
    for _ in 0..count {
        rows.push(cursor);
        cursor += type_row(bytes, cursor)?;
    }
    (cursor <= bytes.len()).then_some(rows)
}
/// A kind-1 row's offset, found by its interval.
fn range_row(bytes: &[u8], start: u128, end: u128) -> Result<usize> {
    let mut operands = vec![1];
    operands.extend(start.to_le_bytes());
    operands.extend(end.to_le_bytes());
    let found = unique(bytes, &operands)?;
    let row = found.checked_sub(5).ok_or("range row")?;
    assert_eq!(bytes[row], 16);
    Ok(row)
}
fn endpoints(bytes: &[u8], row: usize, start: u128, end: u128) -> Vec<u8> {
    spliced(
        &spliced(bytes, row + 6, &start.to_le_bytes()),
        row + 22,
        &end.to_le_bytes(),
    )
}
/// The honest packet's type column: scanning upward, the first count whose
/// parse passes through every range row starts the whole column.
struct Types {
    rows: Vec<usize>,
    ranges: [usize; 5],
    non_null: usize,
}
fn types(bytes: &[u8]) -> Result<Types> {
    let mut ranges = [0; 5];
    for (slot, (_, start, end)) in ranges.iter_mut().zip(RANGES) {
        *slot = range_row(bytes, start, end)?;
    }
    let first = ranges.iter().copied().min().ok_or("range rows")?;
    let rows = (first.saturating_sub(1 << 16)..first)
        .find_map(|offset| {
            type_rows(bytes, offset).filter(|rows| ranges.iter().all(|row| rows.contains(row)))
        })
        .ok_or("type column")?;
    let patterns: Vec<usize> = rows
        .iter()
        .copied()
        .filter(|&row| bytes[row] == 16)
        .collect();
    let [non_null] = *patterns
        .iter()
        .copied()
        .filter(|&row| bytes[row + 5] == 0)
        .collect::<Vec<_>>()
        .as_slice()
    else {
        return Err("one non-null pattern row".into());
    };
    assert_eq!(patterns.len(), 6);
    // Nanoseconds and CodePointInner share the u32 scalar row.
    let u32_row = rows[id_at(bytes, ranges[0] + 1)?];
    assert_eq!(bytes[u32_row..u32_row + 2], [0, 5]);
    assert_eq!(id_at(bytes, ranges[2] + 1)?, id_at(bytes, ranges[0] + 1)?);
    Ok(Types {
        rows,
        ranges,
        non_null,
    })
}
/// The offsets of a role column's count and rows, if one is at `offset`.
/// Compact rows are a tag and an address; spelled rows add the spelling.
/// The provider's lines and `entries` compiled in one source-core crate,
/// `base::` as `crate::`.
fn single_crate(packet: &Packet, label: &str, entries: &str) -> Result<std::process::Command> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let local = source(
        packet,
        &format!("{label}.rs"),
        &format!(
            "{fixture}// Generated single-crate layout control.\nenum Opt<T> {{ None, Some(T) }}\nstruct H(crate::NonNull<u8>);\n{}",
            entries.replace("base::", "crate::")
        ),
    )?;
    let metadata = packet.folder.join(format!("{label}.rgcrate"));
    Ok(packet.export(PROFILE, &local, &metadata))
}
/// A frozen client's entry, renamed by its index.
fn numbered(packet: &Packet, name: &str, index: usize) -> Result<String> {
    let client = std::fs::read_to_string(packet.fixture(name))?;
    let line = client.lines().last().ok_or("client entry")?;
    Ok(format!(
        "#[unsafe(no_mangle)]\n{}\n",
        line.replace("fn entry()", &format!("fn entry{index}()"))
    ))
}
fn lowir(packet: &Packet, label: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        packet.folder.join(format!("{label}.lowir")),
    )?)
}
#[test]
fn pattern_types_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes[12..14], [14, 2]);
    let inner = inner(&bytes)?;
    assert_eq!(bytes[inner + 13..inner + 15], [13, 0]);
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    types(&bytes)?;
    let diagnostic = unique(&bytes, DIAGNOSTIC)?;
    assert_eq!(bytes[diagnostic + DIAGNOSTIC.len()..][..4], [0; 4]);
    let mut entries = String::new();
    for (index, (name, _, _)) in ACCEPTED.iter().enumerate() {
        entries.push_str(&numbered(&packet, name, index)?);
    }
    let mut local = single_crate(&packet, "local", &entries)?;
    super::artifacts::checked("local", &packet.folder, &mut local)?;
    let local_lowir = lowir(&packet, "local")?;
    for (index, (name, size, align)) in ACCEPTED.into_iter().enumerate() {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let imported = function(&lowir(&packet, name)?, "entry")?;
        let single = function(&local_lowir, &format!("entry{index}"))?.replacen(
            &format!("@entry{index}("),
            "@entry(",
            1,
        );
        assert_eq!(imported, single, "{name}");
        // `o` is the entry's first local.
        let slot = imported
            .lines()
            .find(|line| line.trim_start().starts_with("slot "))
            .ok_or("slot holding o")?;
        assert!(
            slot.ends_with(&format!(": obj<{size}x{align}>")),
            "{name}: {slot}"
        );
    }
    for name in PENDING {
        packet.rejected(name, &mut packet.client(name), NICHE)?;
        let label = format!("local-{name}");
        let mut local = single_crate(&packet, &label, &numbered(&packet, name, 0)?)?;
        packet.rejected(&label, &mut local, NICHE)?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
#[test]
fn range_types_keep_their_layout_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "witness", "provider", PROFILE)?;
    let entry_line = format!("#[unsafe(no_mangle)]\n{}\n", witness(WITNESSES[0].0));
    let mut local = single_crate(&packet, "local", &entry_line)?;
    packet.rejected("local", &mut local, REIFY)?;
    let mut observed = Vec::new();
    for (name, size, align) in WITNESSES {
        let label = format!("witness-{name}");
        let client = entry(&packet, &label, &witness(name))?;
        let mut fresh = packet.consumer(&client, &label, &packet.metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        observed.push(held(&lowir(&packet, &label)?, "entry", size, align)?);
    }
    // Nanoseconds and CodePointInner share the u32 base.
    assert_eq!(observed[0], observed[2]);
    Ok(())
}
#[test]
fn older_profiles_diagnostic_identity_and_parameter_bases_fence_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let provider = packet.fixture("provider");
    for profile in OLDER {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut command = packet.export(profile, &provider, &metadata);
        packet.rejected(profile, &mut command, UNIT)?;
    }
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let gate = source(&packet, "gate.rs", &format!("{nominals}{GATE}"))?;
    for profile in OLDER {
        let metadata = packet.folder.join(format!("gate-{profile}.rgcrate"));
        let mut command = packet.export(profile, &gate, &metadata);
        packet.rejected(&format!("gate-{profile}"), &mut command, GATED)?;
    }
    let metadata = packet.folder.join("gate.rgcrate");
    export(&packet, &gate, &metadata)?;
    unique(&std::fs::read(&metadata)?, DIAGNOSTIC)?;
    let client = entry(
        &packet,
        "gate-client",
        "pub extern \"C\" fn entry() -> i32 { let o: Opt<base::NonNull<u8>> = Opt::None; match o { Opt::None => 42, Opt::Some(_) => 7 } }",
    )?;
    let mut fresh = packet.consumer(&client, "gate-client", &metadata, &packet.owner());
    super::artifacts::checked("gate-client", &packet.folder, &mut fresh)?;
    let (lines, pattern_macro) = foundation(&packet)?;
    let field = source(
        &packet,
        "field.rs",
        &format!("{lines}{pattern_macro}{FIELD}"),
    )?;
    let metadata = packet.folder.join("field.rgcrate");
    let mut command = packet.export(PROFILE, &field, &metadata);
    packet.rejected("field", &mut command, PARAMETER)
}
/// The honest packet with its diagnostic row deleted, or duplicated, and the
/// role column's count changed to match. The policy's role range covers only
/// the language rows, so it is unchanged.
fn diagnostic_rows(bytes: &[u8], row: usize, copies: usize) -> Result<Vec<u8>> {
    rewrapped(role_copies(bytes, row, DIAGNOSTIC_ROW, copies)?)
}
/// Packet's declaration: the representation byte that a `repr(C)` variant of
/// the provider changes ends its header row.
fn packet_declaration(packet: &Packet, bytes: &[u8]) -> Result<[u8; 4]> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let variant = source(
        packet,
        "packet-c.rs",
        &fixture.replacen(
            "pub struct Packet<T>",
            "#[repr(C)]\npub struct Packet<T>",
            1,
        ),
    )?;
    let metadata = packet.folder.join("packet-c.rgcrate");
    export(packet, &variant, &metadata)?;
    let offset = difference(bytes, &std::fs::read(&metadata)?)?;
    declaration(bytes, offset)
}
#[test]
fn forged_pattern_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "wire", "provider", PROFILE)?;
    let bytes = &std::fs::read(&packet.metadata)?;
    let rows = types(bytes)?;
    let [nanoseconds, nonzero_char, _, nonzero_u8, percent] = rows.ranges;
    let pointer = id_at(bytes, rows.non_null + 1)?;
    assert_eq!(bytes[rows.rows[pointer]], 12);
    let parameter = id_at(bytes, rows.rows[pointer] + 2)?;
    assert_eq!(bytes[rows.rows[parameter]], 1);
    let scalar = id_at(bytes, nanoseconds + 1)?;
    let reference = rows
        .rows
        .iter()
        .position(|&row| bytes[row] == 2)
        .ok_or("reference recipe")?;
    for (name, changed, diagnostic) in [
        (
            "kind",
            spliced(bytes, rows.non_null + 5, &[2]),
            "unknown source pattern constraint",
        ),
        (
            "non-null-scalar",
            with_id(bytes, rows.non_null + 1, u32::try_from(scalar)?),
            NON_NULL,
        ),
        (
            "non-null-reference",
            with_id(bytes, rows.non_null + 1, u32::try_from(reference)?),
            NON_NULL,
        ),
        (
            "non-null-parameter",
            with_id(bytes, rows.non_null + 1, u32::try_from(parameter)?),
            NON_NULL,
        ),
        (
            "range-pointer",
            with_id(bytes, nanoseconds + 1, u32::try_from(pointer)?),
            "scalar pattern requires an integer or char base",
        ),
        (
            "unsigned-wrapped",
            endpoints(bytes, nonzero_u8, 2, 1),
            UNSIGNED,
        ),
        (
            "char-reversed",
            endpoints(bytes, nonzero_char, 0x0010_ffff, 1),
            UNSIGNED,
        ),
        (
            "surrogate",
            endpoints(bytes, nonzero_char, 1, 0xd800),
            INVALID,
        ),
        (
            "beyond-width",
            endpoints(bytes, nonzero_u8, 1, 0x1ff),
            INVALID,
        ),
    ] {
        let mut consumer = packet.forged(name, changed, "nonnull")?;
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // A well-formed rewritten row, not an honest export: Percent as the bits
    // of `i8 is ..0 | 1..`, witnessed beside the honest packet.
    let client = entry(&packet, "percent-witness", &witness("Percent"))?;
    let mut honest = packet.consumer(&client, "honest", &packet.metadata, &packet.owner());
    super::artifacts::checked("honest", &packet.folder, &mut honest)?;
    let metadata = packet.folder.join("wrapped.rgcrate");
    std::fs::write(&metadata, endpoints(bytes, percent, 1, 0xff))?;
    let mut wrapped = packet.consumer(&client, "wrapped", &metadata, &packet.owner());
    super::artifacts::checked("wrapped", &packet.folder, &mut wrapped)?;
    assert_eq!(
        held(&lowir(&packet, "wrapped")?, "entry", 1, 1)?,
        held(&lowir(&packet, "honest")?, "entry", 1, 1)?
    );
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn forged_diagnostic_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "identity", "provider", PROFILE)?;
    let bytes = &std::fs::read(&packet.metadata)?;
    let diagnostic = unique(bytes, DIAGNOSTIC)?;
    let address = diagnostic + DIAGNOSTIC.len() + 4;
    let (_, roles) = role_column(bytes, diagnostic)?;
    let clone = roles
        .iter()
        .find(|&&row| bytes[row] == 4)
        .ok_or("Clone role row")?;
    let clone = bytes[clone + 5..clone + 9].to_vec();
    for (name, changed, diagnostic) in [
        (
            "spelling",
            spliced(bytes, diagnostic + 5, b"NonNulx"),
            "unknown source diagnostic identity spelling",
        ),
        (
            "clone",
            spliced(bytes, address, &clone),
            "imported NonNull diagnostic item requires its nominal declaration profile",
        ),
        (
            "duplicate",
            diagnostic_rows(bytes, diagnostic, 2)?,
            "duplicate source diagnostic identity",
        ),
    ] {
        let mut consumer = packet.forged(name, changed, "nonnull")?;
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Moving or deleting the identity is accepted until Location consumes it.
    let packet_struct = packet_declaration(&packet, bytes)?;
    for (name, changed) in [
        ("moved", spliced(bytes, address, &packet_struct)),
        ("deleted", diagnostic_rows(bytes, diagnostic, 0)?),
    ] {
        let mut consumer = packet.forged(name, changed, "nonnull")?;
        super::artifacts::checked(name, &packet.folder, &mut consumer)?;
    }
    Ok(())
}
#[test]
fn version_twelve_packets_reject_pattern_and_diagnostic_tags() -> Result {
    let packet = Packet::new(GROUP, SPEC, "twelve", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let mut exported = Vec::new();
    for (name, scalar) in [("w16", "u16"), ("wi16", "i16")] {
        let provider = source(
            &packet,
            &format!("{name}.rs"),
            &format!(
                "{nominals}// Generated version-twelve control.\npub struct W {{ pub w: {scalar} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut command = packet.export(OLDER[0], &provider, &metadata);
        super::artifacts::checked(name, &packet.folder, &mut command)?;
        exported.push(std::fs::read(&metadata)?);
    }
    let bytes = &exported[0];
    assert_eq!(bytes[13], 12);
    let recipe = difference(bytes, &exported[1])? - 1;
    assert_eq!(bytes[recipe], 0);
    // Version thirteen numbers the same declarations, and NonNull's identity
    // row anchors its role column, whose Clone row this packet repeats.
    let gate = source(&packet, "gate.rs", &format!("{nominals}{GATE}"))?;
    let metadata = packet.folder.join("gate.rgcrate");
    export(&packet, &gate, &metadata)?;
    let thirteen = std::fs::read(&metadata)?;
    let (_, roles) = role_column(&thirteen, unique(&thirteen, DIAGNOSTIC)?)?;
    let clone = roles
        .iter()
        .find(|&&row| thirteen[row] == 4)
        .ok_or("Clone role row")?;
    let role = unique(bytes, &thirteen[*clone..clone + 9])?;
    let client = entry(
        &packet,
        "client",
        "pub extern \"C\" fn entry() -> i32 { let o: Opt<base::W> = Opt::None; match o { Opt::None => 42, Opt::Some(_) => 7 } }",
    )?;
    let honest = packet.folder.join("w16.rgcrate");
    let mut consumer = packet.consumer(&client, "honest", &honest, &packet.owner());
    super::artifacts::checked("honest", &packet.folder, &mut consumer)?;
    for (name, forged, diagnostic) in [
        (
            "recipe",
            spliced(bytes, recipe, &[16]),
            "unknown source function type recipe",
        ),
        (
            "role",
            spliced(bytes, role, &[254]),
            "unknown source function role ID",
        ),
    ] {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, forged)?;
        let mut consumer = packet.consumer(&client, name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    Ok(())
}
#[test]
fn pattern_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "records")?;
    let (lines, pattern_macro) = foundation(&packet)?;
    let client = entry(&packet, "client", &witness("R005"))?;
    let mut observed = Vec::new();
    for width in [6, 24, 96] {
        let mut records = String::from("// Generated scaling control.\n");
        for index in 0..width {
            writeln!(
                records,
                "#[repr(transparent)]\npub struct R{index:03}(pattern_type!(u32 is 0..={}));",
                1000 + index
            )?;
        }
        let provider = source(
            &packet,
            &format!("records{width}.rs"),
            &format!("{lines}{pattern_macro}{records}"),
        )?;
        let metadata = packet.folder.join(format!("records{width}.rgcrate"));
        export(&packet, &provider, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let lowir = std::fs::metadata(packet.folder.join(format!("{label}.lowir")))?.len();
        observed.push((std::fs::metadata(&metadata)?.len(), lowir));
    }
    // Fixed-length names and endpoints: every record adds the same
    // declaration, header, field, constructor and pattern row bytes.
    let [(small, first), (middle, _), (large, third)] =
        <[(u64, u64); 3]>::try_from(observed.as_slice())?;
    assert_eq!(
        (middle - small) * 4,
        large - middle,
        "{small} {middle} {large}"
    );
    assert!(first.abs_diff(third) <= 64, "{first} {third}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn pattern_type_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for (name, _, _) in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
