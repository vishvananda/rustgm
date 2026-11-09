//! Fresh CLI processes re-prove version-thirteen unsafe traits and the safety
//! and polarity of every impl of a packet's own non-auto trait. A negative
//! impl maps no member, and each consumer re-proves an impl's safety against
//! its trait; a negative impl denies its application and is never evidence.
use super::fresh_packets::{Packet, Result, relabel, spliced};
use super::marker_transport::{export, source, trait_flags, variant, with_flags};
use super::polarity_transport::{application, applications, id};
const SPEC: &str = "pa29/tests/spec/unsafe-negative-impls";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
const GENERIC: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const SAFETY: &str = "trait and implementation safety declarations disagree";
const ROW_SAFETY: &str = "selected impl safety/member kinds disagree with its actual trait";
const MEMBERS: &str = "auto or negative trait implementations cannot declare members";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const TRAIT_FENCE: &str = "source const/auto/unsafe traits require a wider explicit profile";
const IMPL_FENCE: &str =
    "source const/negative/unsafe implementation requires a wider explicit profile";
const CONDITIONAL: &str = "conditional negative implementation requires scoped debt payment";
/// Frozen outcomes from `pa29/tests/unsafe-negative-impl-outcomes.json`.
const ACCEPTED: [&str; 5] = [
    "trusted",
    "local-trusted",
    "exclusive",
    "error-fault",
    "local-negative",
];
const REJECTED: [(&str, &str); 11] = [
    ("safe-searcher", SAFETY),
    ("unsafe-plain", SAFETY),
    ("unsafe-negative", SAFETY),
    ("deref-positive", OVERLAP),
    ("error-str", GENERIC),
    ("iterator-slice", GENERIC),
    ("deref-shared", CONDITIONAL),
    ("clone-mut", CONDITIONAL),
    ("fuse-plain", GENERIC),
    ("searcher-fault", GENERIC),
    // rustc accepts it: a recorded frontend gap. Negative coherence does not
    // pay the `?Sized` negative impl's MetaSized premise from the
    // intersection, with every item local or imported alike.
    ("negative-coherence", OVERLAP),
];
/// An impl header row: see `polarity_transport.rs`.
const IMPL_ROW: usize = 52;
const APPLICATION: usize = 13;
const UNSAFE: usize = 17;
const NEGATIVE: usize = 19;
/// The trait header's unsafe flag bit; see `marker_transport.rs`.
const UNSAFE_TRAIT: u16 = 1;
const FLAGS: usize = 16;
/// The provider tail's trait impls in source order, each `(unsafe, negative)`.
const TAIL: [(u8, u8); 12] = [
    (0, 1), // !Clone for &mut T
    (0, 1), // !DerefMutLike for &T
    (0, 0), // DerefMutLike for u8
    (0, 1), // !Iterator for [T]
    (0, 0), // Error for &T
    (0, 0), // Error for Fault
    (1, 0), // TrustedFused for Counter
    (1, 0), // Searcher for u8
    (1, 0), // Searcher for Counter
    (1, 0), // TrustedFused for Fuse<I>
    (0, 1), // !Searcher for Fault
    (0, 0), // Plain for Counter
];
const NEGATIVE_DEREF: usize = 1;
const ERROR_FAULT: usize = 5;
const FUSED_COUNTER: usize = 6;
const SEARCHER_U8: usize = 7;
const NEGATIVE_SEARCHER: usize = 10;
const PLAIN: usize = 11;
/// The foundation lines (C1's sized family, `LegacyReceiver`, `Clone`, `Copy`
/// and `Freeze`) that follow the provider's own feature line.
const FOUNDATION: usize = 57;
/// Each tail impl's row, found from the one byte that making `Plain for
/// Counter` negative moves; every row's flags are then checked.
fn impl_rows(packet: &Packet, bytes: &[u8]) -> Result<[usize; 12]> {
    let negative = variant(
        packet,
        bytes,
        "negative-plain",
        "impl Plain for Counter {}",
        "impl !Plain for Counter {}",
    )?;
    let first = (negative - NEGATIVE)
        .checked_sub(PLAIN * IMPL_ROW)
        .ok_or("impl header table")?;
    let rows = std::array::from_fn(|index| first + index * IMPL_ROW);
    for (row, (unsafe_, negative)) in rows.iter().zip(TAIL) {
        assert_eq!(bytes[row + APPLICATION - 1], 1, "trait application tag");
        assert_eq!(
            bytes[row + UNSAFE..=row + NEGATIVE],
            [unsafe_, 0, negative],
            "impl row {row}"
        );
    }
    Ok(rows)
}
/// The flags of the trait each impl at `rows` applies, through the trait
/// header table found from the one byte that making `Iterator` unsafe moves.
/// That variant is generated here and is not a fixture.
fn applied_trait_flags(packet: &Packet, bytes: &[u8], rows: &[usize]) -> Result<Vec<usize>> {
    let iterator = variant(
        packet,
        bytes,
        "unsafe-iterator",
        "pub trait Iterator {",
        "pub unsafe trait Iterator {",
    )?;
    let table = iterator.checked_sub(FLAGS).ok_or("trait header row")?;
    assert_eq!(bytes[table..table + 4], bytes[table + 4..table + 8]);
    assert_eq!(bytes[iterator..iterator + 2], [0, 0]);
    let applications = applications(bytes, rows)?;
    rows.iter()
        .map(|&row| {
            let trait_ = id(bytes, application(bytes, applications, row)?.0 + 4)?;
            trait_flags(bytes, table, trait_.to_le_bytes())
        })
        .collect()
}
#[test]
fn unsafe_and_negative_impls_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "modifiers-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rows = impl_rows(&packet, &bytes)?;
    // TrustedFused and Searcher are unsafe; Plain and Error are not.
    let flags = applied_trait_flags(
        &packet,
        &bytes,
        &[
            rows[FUSED_COUNTER],
            rows[SEARCHER_U8],
            rows[PLAIN],
            rows[ERROR_FAULT],
        ],
    )?;
    for (at, expected) in flags.into_iter().zip([UNSAFE_TRAIT, UNSAFE_TRAIT, 0, 0]) {
        assert_eq!(
            bytes[at..at + 2],
            expected.to_le_bytes(),
            "trait flags at {at}"
        );
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
fn forged_impl_modifiers_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "modifiers-forged", "provider", PROFILE)?;
    let bytes = &std::fs::read(&packet.metadata)?;
    let rows = impl_rows(&packet, bytes)?;
    let traits = applied_trait_flags(&packet, bytes, &[rows[FUSED_COUNTER], rows[SEARCHER_U8]])?;
    let safe_traits = traits.iter().fold(bytes.clone(), |forged, &at| {
        with_flags(&forged, at, |f| f & !UNSAFE_TRAIT)
    });
    for (name, changed, diagnostic) in [
        (
            "unsafe-plain",
            spliced(bytes, rows[PLAIN] + UNSAFE, &[1]),
            ROW_SAFETY,
        ),
        (
            "safe-fused",
            spliced(bytes, rows[FUSED_COUNTER] + UNSAFE, &[0]),
            ROW_SAFETY,
        ),
        (
            "unsafe-negative",
            spliced(bytes, rows[NEGATIVE_SEARCHER] + UNSAFE, &[1]),
            ROW_SAFETY,
        ),
        ("safe-traits", safe_traits, ROW_SAFETY),
        // Admission rejects a negative impl that maps members, and a positive
        // impl that maps none of its trait's members.
        (
            "negative-searcher",
            spliced(bytes, rows[SEARCHER_U8] + NEGATIVE, &[1]),
            MEMBERS,
        ),
        (
            "positive-deref",
            spliced(bytes, rows[NEGATIVE_DEREF] + NEGATIVE, &[0]),
            "source implementation omits an actual trait member mapping",
        ),
        (
            "version-12",
            relabel(bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let mut consumer = packet.forged(name, changed, "trusted")?;
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    let mut honest = packet.forged("honest", bytes.clone(), "trusted")?;
    super::artifacts::checked("honest", &packet.folder, &mut honest)?;
    Ok(())
}
#[test]
fn generated_modifier_controls_keep_their_diagnostics() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "modifiers-gates", "provider", PROFILE)?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    // Core's same-head pair exports, and a consumer re-proves it from the
    // packet's perspective, where `Error` is local.
    let pair = source(
        &packet,
        "pair.rs",
        &format!("{fixture}// Generated control.\nimpl !Error for &str {{}}\n"),
    )?;
    let metadata = packet.folder.join("pair.rgcrate");
    export(&packet, &pair, &metadata)?;
    let answer = source(
        &packet,
        "answer.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n",
    )?;
    let mut consumer = packet.consumer(&answer, "pair-answer", &metadata, &packet.owner());
    super::artifacts::checked("pair-answer", &packet.folder, &mut consumer)?;
    // Older versions keep both fences: an unsafe trait meets the trait
    // fence, and a negative impl of a trait with members, which the ledger
    // now admits, meets the impl fence. Version 13 exports both.
    let foundation: String = fixture
        .split_inclusive('\n')
        .skip(1)
        .take(FOUNDATION)
        .collect();
    assert!(foundation.ends_with("pub unsafe auto trait Freeze {}\n"));
    for (name, line, diagnostic) in [
        (
            "unsafe-only",
            "pub unsafe trait TrustedFused {}",
            TRAIT_FENCE,
        ),
        (
            "negative-only",
            "impl<T: PointeeSized> !Clone for &mut T {}",
            IMPL_FENCE,
        ),
    ] {
        let gate = source(
            &packet,
            &format!("{name}.rs"),
            &format!(
                "#![feature(negative_impls)]\n{foundation}// Generated older-profile control.\n{line}\n"
            ),
        )?;
        export(
            &packet,
            &gate,
            &packet.folder.join(format!("{name}.rgcrate")),
        )?;
        for profile in OLDER {
            let label = format!("{name}-{profile}");
            let metadata = packet.folder.join(format!("{label}.rgcrate"));
            let mut command = packet.export(profile, &gate, &metadata);
            packet.rejected(&label, &mut command, diagnostic)?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn modifier_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "modifiers-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
