//! Version thirteen carries `#[repr(simd)]` as nominal header representation
//! byte 3. Fresh consumers map it to their own SIMD representation, re-run
//! the declaration and layout checks over their own field and re-prove an
//! imported asm body's `xmm_reg` operand. SIMD forms outside the slice keep
//! named fences at export, older profiles keep theirs and forged
//! representation bytes reject by name.
use super::fresh_packets::{Packet, Result, inner};
use super::newtype_records::header_row;
use rustgm::support::crates;
const SPEC: &str = "pa41/tests/spec/simd-transport";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "simd-transport";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa41/tests/simd-transport-outcomes.json`.
const DECLARATION: &str = "source declaration kind is outside the initial source profile";
const WIDER: &str = "source nominal representation requires its wider complete contract";
const GENERIC: &str = "source generic SIMD record requires its wider complete contract";
const UNKNOWN: &str = "unknown source nominal representation";
const REPROOF: &str = "assembly SIMD operand requires its completed genuine representation";
/// The lines every generated control keeps: the provider's first 73, the
/// pinned foundation with `__m128`.
const FOUNDATION: usize = 73;
/// Each generated control is the foundation then one line, as the oracle
/// wrote it.
const CONTROLS: [(&str, &str, &str); 6] = [
    (
        "packed",
        "#[repr(simd, packed)] pub struct P(pub [f32; 3]);\n",
        WIDER,
    ),
    (
        "align",
        "#[repr(simd, align(32))] pub struct A(pub [f32; 4]);\n",
        WIDER,
    ),
    (
        "generic",
        "#[repr(simd)] pub struct G<T>(pub [T; 4]);\n",
        GENERIC,
    ),
    (
        "const-generic",
        "#[repr(simd)] pub struct N<const L: usize>(pub [f32; L]);\n",
        GENERIC,
    ),
    (
        "lane-limit",
        "#[repr(simd)] #[rustc_simd_monomorphize_lane_limit = \"8\"] pub struct L(pub [f32; 4]);\n",
        "SIMD lane limits require authenticated representation metadata transport",
    ),
    (
        "c",
        "#[repr(simd, C)] pub struct K(pub [f32; 4]);\n",
        "conflicting representation hints",
    ),
];

fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// The provider's nominal table: its count, 2, then `__m128`'s and `V`'s
/// 43-byte header rows, each ending in representation byte 3. `__m128` owns
/// field 0 and `V` field 1. Returns `V`'s representation offset.
fn simd_table(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (46..bytes.len().saturating_sub(43))
        .filter(|&first| {
            bytes[first - 46..first - 42] == [2, 0, 0, 0]
                && [first, first + 43]
                    .iter()
                    .all(|&offset| bytes[offset] == 3 && header_row(bytes, offset))
        })
        .collect();
    let [first] = *found.as_slice() else {
        return Err(format!("SIMD nominal tables {found:?}").into());
    };
    let simd = first + 43;
    assert_eq!(bytes[first - 20..first - 12], [0, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(bytes[simd - 20..simd - 12], [1, 0, 0, 0, 2, 0, 0, 0]);
    Ok(simd)
}
/// Writes `source` under the packet's folder and exports it under `profile`.
fn generated(
    packet: &Packet,
    name: &str,
    profile: &str,
    source: &str,
) -> Result<std::process::Command> {
    let path = packet.folder.join(format!("{name}.rs"));
    std::fs::write(&path, source)?;
    let metadata = packet.folder.join(format!("{name}.rgcrate"));
    Ok(packet.export(profile, &path, &metadata))
}
#[test]
fn simd_packets_export_and_fresh_consumers_reprove_their_vector_operands() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "simd-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // The foundation's builtin `arch::asm` row needs Lane B's macro
    // envelope around the version-thirteen core, as A's packet does.
    assert_eq!(bytes.get(12..14), Some([14, 2].as_slice()));
    let core = inner(&bytes)?;
    assert_eq!(bytes.get(core + 13..core + 15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    // `__m128` and `V`; neither carries a lane or count fact.
    simd_table(&bytes)?;
    // `[f32; 4]` and `[u32; 4]`: `14 | element | 0 | usize | 4`.
    let mut four = [0_u8; 16];
    four[0] = 4;
    let lengths: Vec<&[u8]> = bytes
        .windows(26)
        .filter(|row| row[0] == 14 && row[5] == 0 && row[10..] == four)
        .map(|row| &row[6..10])
        .collect();
    assert!(
        matches!(lengths.as_slice(), [f, u] if f == u),
        "{lengths:?}"
    );
    super::artifacts::checked("store", &packet.folder, &mut packet.client("store"))?;
    let store = std::fs::read_to_string(packet.folder.join("store.lowir"))?;
    assert_eq!(store.matches("assembly @asm").count(), 1);
    assert_eq!(
        store
            .matches("  text \"movups [\"\n  use 0 0 0 0\n  text \"],\"\n  use 1 0 0 0\n")
            .count(),
        1
    );
    packet.rejected(
        "mistyped",
        &mut packet.client("mistyped"),
        "incompatible types",
    )?;
    let source = packet.fixture("provider");
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        let mut export = packet.export(profile, &source, &older);
        packet.rejected(profile, &mut export, DECLARATION)?;
        assert!(unwritten(&older)?, "{profile} wrote a packet");
    }
    Ok(())
}
#[test]
fn simd_forms_outside_the_slice_keep_their_fences_at_export() -> Result {
    let packet = Packet::new(GROUP, SPEC, "simd-controls", "provider")?;
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let lines: Vec<&str> = provider.lines().take(FOUNDATION).collect();
    let foundation = format!("{}\n", lines.join("\n"));
    let mut alone = generated(&packet, "foundation", PROFILE, &foundation)?;
    super::artifacts::checked("foundation", &packet.folder, &mut alone)?;
    for (name, line, diagnostic) in CONTROLS {
        let mut export = generated(&packet, name, PROFILE, &format!("{foundation}{line}"))?;
        packet.rejected(name, &mut export, diagnostic)?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    // The older profiles reach the representation gate on the source-nominals
    // foundation, which does not stop them earlier.
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let gated = nominals.replacen("auto_traits)]", "auto_traits,repr_simd)]", 1);
    assert_ne!(gated, nominals);
    let gate = format!("{gated}#[repr(simd)] pub struct V {{ pub lanes: [u32; 4] }}\n");
    for profile in OLDER {
        let mut export = generated(&packet, &format!("gate-{profile}"), profile, &gate)?;
        packet.rejected(profile, &mut export, WIDER)?;
    }
    let mut export = generated(&packet, "gate", PROFILE, &gate)?;
    super::artifacts::checked("gate", &packet.folder, &mut export).map(drop)
}
#[test]
fn forged_simd_headers_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "simd-forged", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // Only `V` is an asm operand, so the re-proof text also shows which
    // header was patched.
    let simd = simd_table(&bytes)?;
    for (name, value, diagnostic) in [("rust", 0, REPROOF), ("byte-4", 4, UNKNOWN)] {
        let mut forged = bytes.clone();
        forged[simd] = value;
        let mut client = packet.forged(name, forged, "store")?;
        packet.rejected(name, &mut client, diagnostic)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn simd_store_client_executes_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "simd-native", "provider", PROFILE)?;
    super::artifacts::checked("store", &packet.folder, &mut packet.client("store"))?;
    let output = packet.native("store")?;
    assert_eq!(output.status.code(), Some(42), "store");
    assert!(output.stdout.is_empty(), "store");
    Ok(())
}
