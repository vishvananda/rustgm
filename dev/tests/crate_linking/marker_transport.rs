//! Fresh CLI processes re-prove version-thirteen auto traits: each exact empty
//! auto declaration, each role-bound marker's local flags and `Send`'s
//! diagnostic identity, which fills every consumer's own `Send` role slot.
use super::fresh_packets::{Packet, Result, declaration, difference, role_copies, spliced, unique};
use rustgm::support::crates;
use std::path::Path;
const SPEC: &str = "pa29/tests/spec/auto-declarations";
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/auto-declaration-outcomes.json`.
const ACCEPTED: [&str; 7] = [
    "send-structural",
    "sync-static",
    "local-static",
    "pin-markers",
    "bounded-param",
    "local-holder",
    "answer",
];
const REJECTED: [(&str, &str); 3] = [
    (
        "unbounded-param",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
    (
        "duplicate-lang",
        "compiler bindings require an explicit source-core provider",
    ),
    (
        "duplicate-send",
        "diagnostic identity requires its rustc_attrs source provider",
    ),
];
/// `Send`'s diagnostic identity row: tag 254, the spelling's length and bytes,
/// then the declaration address.
const SEND: &[u8] = b"\xfe\x04\x00\x00\x00Send";
const SEND_ROW: usize = SEND.len() + 8;
const SEND_TEXT: &str = "imported Send diagnostic item requires its unsafe auto trait declaration";
/// The trait header's flag bits.
const UNSAFE: u16 = 1;
const AUTO: u16 = 4;
/// A trait header row's flags follow its declaration, scope, Self
/// declaration and Self type ids; its fixed-width ranges and counts follow.
const FLAGS: usize = 16;
const TRAIT_ROW: usize = 66;
/// The provider's sized family, `LegacyReceiver`, `Clone`, `Copy` and
/// `Freeze`: the lines that versions 12 and 10 export.
const FOUNDATION: usize = 57;
/// Each older-profile declaration and its verbatim diagnostic on main.
const GATES: [(&str, &str, &str); 6] = [
    (
        "custom",
        "pub auto trait Custom {}\n",
        "source const/auto/unsafe traits require a wider explicit profile",
    ),
    (
        "unsafe-custom",
        "pub unsafe auto trait Custom {}\n",
        "source const/auto/unsafe traits require a wider explicit profile",
    ),
    (
        "send",
        "#[rustc_diagnostic_item = \"Send\"]\npub unsafe auto trait Send {}\n",
        "source Send diagnostic contract transport is not implemented",
    ),
    (
        "unsafe-unpin",
        "#[lang = \"unsafe_unpin\"]\npub unsafe auto trait UnsafeUnpin {}\n",
        "source UnsafeUnpin contract transport is not implemented",
    ),
    (
        "sync",
        "#[lang = \"sync\"]\npub unsafe auto trait Sync {}\n",
        "source role requires a wider explicit source profile",
    ),
    (
        "unpin",
        "#[lang = \"unpin\"]\npub auto trait Unpin {}\n",
        "source role requires a wider explicit source profile",
    ),
];
pub(super) fn source(packet: &Packet, name: &str, text: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}
pub(super) fn export(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut command = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut command).map(drop)
}
/// The provider exported with one textual change, and the single byte that
/// change moves.
pub(super) fn variant(
    packet: &Packet,
    bytes: &[u8],
    name: &str,
    from: &str,
    to: &str,
) -> Result<usize> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let changed = source(
        packet,
        &format!("{name}.rs"),
        &fixture.replacen(from, to, 1),
    )?;
    let metadata = packet.folder.join(format!("{name}.rgcrate"));
    export(packet, &changed, &metadata)?;
    difference(bytes, &std::fs::read(&metadata)?)
}
/// The declaration a spelled language role row names.
fn role_declaration(bytes: &[u8], spelling: &str) -> Result<[u8; 4]> {
    let mut row = vec![255];
    row.extend_from_slice(&u32::try_from(spelling.len())?.to_le_bytes());
    row.extend_from_slice(spelling.as_bytes());
    let address = unique(bytes, &row)? + row.len();
    Ok(<[u8; 4]>::try_from(&bytes[address + 4..address + 8])?)
}
/// The trait header row of `UnwindSafe`, found as the one byte that its
/// unsafety moves: it anchors the fixed-width trait header table.
fn calibrated(packet: &Packet, bytes: &[u8]) -> Result<usize> {
    let custom = variant(
        packet,
        bytes,
        "unsafe-unwind",
        "pub auto trait UnwindSafe {}",
        "pub unsafe auto trait UnwindSafe {}",
    )?;
    let row = custom.checked_sub(FLAGS).ok_or("trait header row")?;
    assert_eq!(bytes[row..row + 4], bytes[row + 4..row + 8]);
    assert_eq!(bytes[custom..custom + 2], AUTO.to_le_bytes());
    Ok(row)
}
/// The flags of the one row of the trait header table at `table` that
/// `declaration` heads as its own scope.
pub(super) fn trait_flags(bytes: &[u8], table: usize, declaration: [u8; 4]) -> Result<usize> {
    let pattern = [declaration, declaration].concat();
    let mut rows = (0..bytes.len()).filter(|&offset| {
        offset.abs_diff(table) % TRAIT_ROW == 0 && bytes[offset..].starts_with(&pattern)
    });
    let row = rows.next().ok_or("trait header row is absent")?;
    if rows.next().is_some() {
        return Err("trait header row is ambiguous".into());
    }
    Ok(row + FLAGS)
}
pub(super) fn with_flags(bytes: &[u8], offset: usize, change: impl Fn(u16) -> u16) -> Vec<u8> {
    let flags = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
    spliced(bytes, offset, &change(flags).to_le_bytes())
}
#[test]
fn auto_declarations_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let send = unique(&bytes, SEND)?;
    assert_eq!(bytes[send + SEND.len()..][..4], [0; 4]);
    let table = calibrated(&packet, &bytes)?;
    for (spelling, flags) in [
        ("sync", UNSAFE | AUTO),
        ("unsafe_unpin", UNSAFE | AUTO),
        ("unpin", AUTO),
    ] {
        let at = trait_flags(&bytes, table, role_declaration(&bytes, spelling)?)?;
        assert_eq!(bytes[at..at + 2], flags.to_le_bytes(), "{spelling}");
    }
    let declared = <[u8; 4]>::try_from(&bytes[send + SEND.len() + 4..send + SEND_ROW])?;
    let at = trait_flags(&bytes, table, declared)?;
    assert_eq!(bytes[at..at + 2], (UNSAFE | AUTO).to_le_bytes());
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in REJECTED {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
#[test]
fn forged_auto_declarations_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "forged", "provider", PROFILE)?;
    let bytes = &std::fs::read(&packet.metadata)?;
    // A custom auto trait's unsafety moves only its own flags.
    let table = calibrated(&packet, bytes)?;
    let heads = <[u8; 4]>::try_from(&bytes[table..table + 4])?;
    assert_eq!(trait_flags(bytes, table, heads)?, table + FLAGS);
    let send = unique(bytes, SEND)?;
    let address = send + SEND.len() + 4;
    let declared = <[u8; 4]>::try_from(&bytes[address..address + 4])?;
    let pointer = variant(
        &packet,
        bytes,
        "raw-c",
        "pub struct Raw",
        "#[repr(C)]\npub struct Raw",
    )?;
    let pointer = declaration(bytes, pointer)?;
    let flags = |spelling| -> Result<usize> {
        trait_flags(bytes, table, role_declaration(bytes, spelling)?)
    };
    for (name, changed, diagnostic) in [
        (
            "auto-add",
            with_flags(bytes, flags("add")?, |f| f | AUTO),
            "source auto trait requires its exact empty auto declaration",
        ),
        (
            "safe-sync",
            with_flags(bytes, flags("sync")?, |f| f & !UNSAFE),
            "source Sync role requires its exact auto declaration flags",
        ),
        (
            "unsafe-unpin",
            with_flags(bytes, flags("unpin")?, |f| f | UNSAFE),
            "source Unpin role requires its exact auto declaration flags",
        ),
        (
            "plain-send",
            with_flags(bytes, trait_flags(bytes, table, declared)?, |f| f & !AUTO),
            SEND_TEXT,
        ),
        ("struct-send", spliced(bytes, address, &pointer), SEND_TEXT),
        (
            "duplicate",
            role_copies(bytes, send, SEND_ROW, 2)?,
            "duplicate source diagnostic identity",
        ),
        (
            "spelling",
            spliced(bytes, send + 5, b"Senx"),
            "unknown source diagnostic identity spelling",
        ),
    ] {
        let mut consumer = packet.forged(name, changed, "answer")?;
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Without its identity, Send is an ordinary custom auto trait.
    let mut consumer =
        packet.forged("deleted", role_copies(bytes, send, SEND_ROW, 0)?, "answer")?;
    super::artifacts::checked("deleted", &packet.folder, &mut consumer)?;
    Ok(())
}
#[test]
fn older_profiles_fence_auto_declarations_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation: String = fixture.split_inclusive('\n').take(FOUNDATION).collect();
    assert!(foundation.ends_with("pub unsafe auto trait Freeze {}\n"));
    for (name, declaration, diagnostic) in GATES {
        let gate = source(
            &packet,
            &format!("{name}.rs"),
            &format!("{foundation}// Generated older-profile gate.\n{declaration}"),
        )?;
        for profile in OLDER {
            let label = format!("{name}-{profile}");
            let metadata = packet.folder.join(format!("{label}.rgcrate"));
            let mut command = packet.export(profile, &gate, &metadata);
            packet.rejected(&label, &mut command, diagnostic)?;
        }
        export(
            &packet,
            &gate,
            &packet.folder.join(format!("{name}.rgcrate")),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn auto_declaration_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
