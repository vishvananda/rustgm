//! Fresh CLI processes re-authenticate the version-thirteen Tuple marker and
//! FnOnce/FnMut/Fn roles, and re-prove every transported call operator.
use super::fresh_packets::{Packet, Result, relabel, unique};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/callable-roles";
const GROUP: &str = "callable-roles";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/callable-role-outcomes.json`.
const ACCEPTED: [&str; 9] = [
    "once-item",
    "once-closure",
    "once-capture",
    "mut-item",
    "mut-closure",
    "mut-capture",
    "ref-item",
    "ref-closure",
    "ref-capture",
];
const OPPONENTS: [&str; 3] = ["wrong-output", "ref-mutating", "mut-consuming"];
const UNPROVED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
/// Body IR node 29 (`Rvalue::Invoke`), arity 3, then its call-trait flag.
fn invoke(kind: u8) -> [u8; 8] {
    [29, 3, 8, kind, 0, 0, 0, 4]
}
/// `apply_mut`'s receiver: node 12 (`Rvalue::Borrow`) of local 0, flag 1.
const TWO_PHASE: [u8; 12] = [12, 2, 11, 0, 0, 0, 0, 8, 1, 0, 0, 0];
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
fn with_byte(bytes: &[u8], offset: usize, value: u8) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset] = value;
    changed
}
#[test]
fn callable_roles_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for kind in 0..3 {
        unique(&bytes, &invoke(kind))?;
    }
    unique(&bytes, &TWO_PHASE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for name in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), UNPROVED)?;
    }
    let source = packet.fixture("provider");
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "named, associated or generic alias transport requires its wider profile",
        )?;
    }
    // Closures inside provider bodies stay fenced by name.
    let nested = packet.folder.join("nested.rs");
    std::fs::write(
        &nested,
        format!(
            "{}pub fn twice(x: u8) -> u8 {{ apply(|y| y, x) }}\n",
            std::fs::read_to_string(&source)?
        ),
    )?;
    let fenced = packet.folder.join("nested.rgcrate");
    packet.rejected(
        "nested",
        &mut packet.export(PROFILE, &nested, &fenced),
        "nested callable source arguments require a wider transport profile",
    )?;
    Ok(())
}
#[test]
fn forged_call_operators_and_older_wires_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let once = unique(&bytes, &invoke(0))? + 3;
    let mutable = unique(&bytes, &invoke(1))? + 3;
    let borrow = unique(&bytes, &TWO_PHASE)? + 8;
    let role = unique(&bytes, b"fn_mut")?;
    let mut relabelled_role = bytes.clone();
    relabelled_role[role..role + 6].copy_from_slice(b"unsize");
    for (name, client, wire, diagnostic) in [
        (
            "once-as-fn",
            "once-item",
            with_byte(&bytes, once, 2),
            "source call operator receiver changed its call trait borrow",
        ),
        (
            "once-as-mut",
            "once-item",
            with_byte(&bytes, once, 1),
            "source call operator receiver changed its call trait borrow",
        ),
        (
            "unknown-trait",
            "once-item",
            with_byte(&bytes, once, 3),
            "body IR invoke node has an unknown call trait",
        ),
        (
            "mut-as-once",
            "mut-item",
            with_byte(&bytes, mutable, 0),
            "source call operator result is not its callee's declared Output",
        ),
        (
            "unique-receiver",
            "mut-item",
            with_byte(&bytes, borrow, 2),
            "encoded unique Rvalue::Borrow is not reconstructable",
        ),
        (
            "role-relabel",
            "mut-item",
            relabelled_role,
            "source const/auto/unsafe traits require a wider explicit profile",
        ),
        (
            "relabelled",
            "once-item",
            relabel(&bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let source = packet.fixture(client);
        let mut consumer = packet.consumer(&source, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // The stricter immediate receiver is a different valid body; the
    // consumer's own borrow proof re-checks it.
    let immediate = packet.folder.join("immediate-receiver.rgcrate");
    std::fs::write(&immediate, with_byte(&bytes, borrow, 0))?;
    let source = packet.fixture("mut-item");
    let mut consumer = packet.consumer(&source, "immediate-receiver", &immediate, &packet.owner());
    super::artifacts::checked("immediate-receiver", &packet.folder, &mut consumer)?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn call_operator_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "appliers")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("pub fn apply<")
        .next()
        .ok_or("callable foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::apply000(|x| match x { 41 => 42, _ => 7 }, 41u8) as i32 }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let appliers = (0..width).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "pub fn apply{index:03}<F: FnOnce(u8) -> u8>(f: F, x: u8) -> u8 {{ f(x) }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("appliers{width}.rs"));
        std::fs::write(
            &source,
            format!("{foundation}// Generated scaling control.\n{appliers}"),
        )?;
        let metadata = packet.folder.join(format!("appliers{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every applier adds the same packet bytes, and the
    // consumer instantiates only the one it calls.
    let ([small, middle, large], [first, second, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(
        third.abs_diff(second) <= 5 * second.abs_diff(first).max(64),
        "{lowir:?}"
    );
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn callable_role_instances_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
