//! Fresh CLI processes consume calls through closed Rust fn pointers in
//! exported bodies, and constructors reified to fn pointers. Each consumer
//! re-proves a call's signature against the callee's pointer type and its
//! lexical safety, then calls the pointer value in its own lowering.
//! Higher-ranked, C and associated unsafe calls keep their fences by name, as
//! do pointers stored in record variants; older profiles and readers reject
//! the form, and malformed code-30 operand lists and unknown safety flags
//! reject by name. A call through a nominal fn field exports since (ii).
use super::fresh_packets::{Packet, Result, spliced, with_id};
use rustgm::support::crates;
const SPEC: &str = "pa29/tests/spec/pointer-calls";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/pointer-call-outcomes.json`.
const ACCEPTED: [&str; 13] = [
    "call",
    "twice",
    "generic",
    "reified",
    "block",
    "safe-block",
    "nested",
    "bare",
    "formatter",
    "method",
    "ctor",
    "variant",
    "some",
];
const CALLABLE: &str = "source anonymous callable requires its supported closed Rust frame";
const DIRECT: &str = "source Call requires its actual direct function-item application";
const MALFORMED: &str = "body IR value node has malformed operands";
/// Each generated provider opponent appends these lines to the frozen
/// provider. rustc 1.98.0 compiles every one but `const-fn`.
const OPPONENTS: [(&str, &str, &str); 5] = [
    (
        "hr",
        "pub fn hr_call(f: for<'a> fn(&'a u8) -> &'a u8, v: &u8) -> u8 { *f(v) }\n",
        CALLABLE,
    ),
    (
        "c",
        "pub fn c_call(f: extern \"C\" fn(u8) -> i32) -> i32 { f(7) }\n",
        CALLABLE,
    ),
    (
        "associated",
        "pub struct H;\nimpl H { pub fn call(f: unsafe fn(u8) -> i32) -> i32 { unsafe { f(7) } } }\n",
        "source unsafe associated Calls require their wider actual profile",
    ),
    (
        "record-variant",
        "pub enum Kind { Placeholder { value: u8, formatter: unsafe fn(u8, u8) -> i32 }, Count(u16) }\npub struct Arg { pub ty: Kind }\npub fn stored(value: u8) -> Arg { Arg { ty: Kind::Placeholder { value, formatter: render } } }\npub unsafe fn stored_fmt(arg: &Arg, f: u8) -> i32 { match arg.ty { Kind::Placeholder { value, formatter } => unsafe { formatter(value, f) }, Kind::Count(_) => 7 } }\n",
        "source record variants require their wider profile",
    ),
    (
        "const-fn",
        "pub const fn const_call(f: fn(u8) -> i32) -> i32 { f(7) }\n",
        "non-const calls are not allowed in const functions",
    ),
];
/// The former `field` opponent, which exports since (ii): outcomes
/// amendment 2.
const FIELD: &str =
    "pub struct Slot { pub f: fn(u8) -> i32 }\npub fn via(s: &Slot) -> i32 { (s.f)(7) }\n";
/// The lines the older-profile control appends to the nominals provider.
const OLDER: &str =
    "// Generated pointer-call control.\npub fn call_ptr(f: fn(u8) -> i32) -> i32 { f(7) }\n";
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// Offsets of value nodes `30 | 4 | Type | Value | Range | Flag`.
fn pointer_nodes(bytes: &[u8]) -> Vec<usize> {
    (0..bytes.len().saturating_sub(26))
        .filter(|&at| {
            bytes[at..at + 3] == [30, 4, 4]
                && bytes[at + 7] == 0
                && bytes[at + 12] == 9
                && bytes[at + 21] == 8
        })
        .collect()
}
/// Each node's safety flag, counted as `Safe`, `ExplicitUnsafeBlock` and
/// `UnsafeFunction`.
fn safeties(bytes: &[u8], nodes: &[usize]) -> Result<[usize; 3]> {
    let mut counts = [0; 3];
    for &at in nodes {
        let flag = u32::from_le_bytes(bytes[at + 22..at + 26].try_into()?);
        *counts
            .get_mut(usize::try_from(flag)?)
            .ok_or("safety flag")? += 1;
    }
    Ok(counts)
}
#[test]
fn clients_call_pointers_and_stored_pointers_keep_their_fences() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "pointer-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let nodes = pointer_nodes(&bytes);
    assert_eq!(nodes.len(), 11);
    assert_eq!(safeties(&bytes, &nodes)?, [7, 3, 1]);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    for (name, lines, diagnostic) in OPPONENTS {
        let source = packet.folder.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!("{provider}// Generated pointer-call opponent.\n{lines}"),
        )?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        packet.rejected(name, &mut export, diagnostic)?;
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    let source = packet.folder.join("field.rs");
    std::fs::write(
        &source,
        format!("{provider}// Generated pointer-call opponent.\n{FIELD}"),
    )?;
    let metadata = packet.folder.join("field.rgcrate");
    let mut export = packet.export(PROFILE, &source, &metadata);
    super::artifacts::checked("field", &packet.folder, &mut export)?;
    Ok(())
}
#[test]
fn older_profiles_and_malformed_pointer_calls_reject_by_name() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "pointer-older", "provider", PROFILE)?;
    let nominals = std::fs::read_to_string(
        packet
            .root
            .join("pa29/tests/spec/source-nominals/provider.rs"),
    )?;
    let source = packet.folder.join("older.rs");
    std::fs::write(&source, format!("{nominals}{OLDER}"))?;
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut export = packet.export(profile, &source, &metadata);
        packet.rejected(profile, &mut export, DIRECT)?;
        assert!(unwritten(&metadata)?, "{profile} wrote a packet");
    }
    let thirteen = packet.folder.join("thirteen.rgcrate");
    let mut export = packet.export(PROFILE, &source, &thirteen);
    super::artifacts::checked("thirteen", &packet.folder, &mut export)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let first = *pointer_nodes(&bytes).first().ok_or("pointer call node")?;
    // Same-width substitutes at positions 0, 1 and 3, then an unknown flag.
    let forgeries = [
        ("type-kind", spliced(&bytes, first + 2, &[6]), MALFORMED),
        ("callee-kind", spliced(&bytes, first + 7, &[11]), MALFORMED),
        ("flag-kind", spliced(&bytes, first + 21, &[0]), MALFORMED),
        (
            "flag-three",
            with_id(&bytes, first + 22, 3),
            "unknown source call safety context",
        ),
    ];
    for (name, forged, diagnostic) in forgeries {
        packet.rejected(name, &mut packet.forged(name, forged, "call")?, diagnostic)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn pointer_call_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "pointer-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
