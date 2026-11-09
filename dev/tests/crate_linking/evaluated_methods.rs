//! Fresh CLI processes re-prove version-thirteen const inherent methods,
//! including those of lifetime-capturing impls: each consumer re-checks and
//! evaluates the transported original as it does a const free original, with
//! the impl's captured regions as leading early parameters. No const evidence
//! travels.
use super::fresh_packets::{Packet, Result, relabel, unique};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/const-methods";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "const-methods";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/const-method-outcomes.json`.
const ACCEPTED: [&str; 10] = [
    "new",
    "from-str",
    "relay",
    "make",
    "plain",
    "const-count",
    "const-relay",
    "const-plain",
    "const-held",
    "const-fn",
];
const NONCONST: &str = "non-const calls are not allowed in constant initializers";
const OPPONENTS: [(&str, &str); 4] = [
    ("runtime-in-const", NONCONST),
    ("runtime-argument", "unresolved value name"),
    ("wrong-payload", "incompatible types"),
    ("text-leak", "required lifetime does not outlive its target"),
];
const PLAIN: &str = "// Generated const inherent-method gate.\npub struct Plain { pub n: u8 }\nimpl Plain { pub const fn get(&self) -> u8 { 42 } }\n";
/// Guarded-method mover M1: version thirteen exports a const unsafe inherent
/// method, and a client's const item calls it in an unsafe block.
const UNSAFE: &str = "// Generated const unsafe inherent-method gate.\npub struct Plain { pub n: u8 }\nimpl Plain { pub const unsafe fn get(&self) -> u8 { 42 } }\n";
/// M1's client, the oracle's input: rustc 1.98.0 ICEs on `client`'s match
/// over this provider, which implements `Copy` for no primitive.
const UNSAFE_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\nconst N: u8 = unsafe { base::Plain { n: 1 }.get() };\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { N as i32 }\n";
const FENCED: [(&str, &str, &str); 1] = [(
    "early",
    "// Generated early-lifetime const-method gate.\npub struct Held<'a> { pub text: &'a str }\nimpl<'a> Held<'a> { pub const fn shorter<'b>(&self, x: &'b u8) -> &'b u8 where 'a: 'b { x } }\n",
    "source function early/captured callable frame transport is not implemented",
)];
/// A field read of a call's temporary, as `relay` reads `Self::new(..).n`.
const TEMPORARY: &str = "// Generated temporary-field gate.\npub struct P { pub n: u8 }\npub fn mp() -> P { P { n: 42 } }\npub fn f() -> u8 { mp().n }\n";
/// rustc 1.98.0 also admits this write; this frontend keeps it fenced by
/// name, a recorded deviation.
const TEMPORARY_WRITE: &str = "pub fn w() -> u8 { mp().n = 5; 42 }\n";
const GATE: &str = "// Generated const-method wire control.\npub struct Gate<'a> { pub t: &'a u8, pub n: u8 }\nimpl<'a> Gate<'a> { pub const fn get(&self) -> u8 { self.n } pub fn slow(&self) -> u8 { self.n } }\n";
/// `Gate` again, with a `slow` whose body calls an ordinary function.
const CALLING: &str = "// Generated runtime-call wire control.\npub fn helper(n: u8) -> u8 { n }\npub struct Gate<'a> { pub t: &'a u8, pub n: u8 }\nimpl<'a> Gate<'a> { pub const fn get(&self) -> u8 { self.n } pub fn slow(&self) -> u8 { helper(self.n) } }\n";
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
fn generated(packet: &Packet, name: &str, source: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, source)?;
    Ok(path)
}
fn exported(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut export = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut export).map(drop)
}
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored const inherent-method transport controls.")
        .next()
        .ok_or("const-method foundation")?
        .to_owned())
}
/// A client that evaluates `value` in a const item and reads it at runtime.
fn client(packet: &Packet, name: &str, value: &str) -> Result<std::path::PathBuf> {
    generated(
        packet,
        &format!("{name}.rs"),
        &format!(
            "#![feature(no_core)]\n#![no_core]\nextern crate base;\nconst N: u8 = {value};\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ match N {{ 42 => 42, _ => 7 }} }}\n"
        ),
    )
}
#[test]
fn const_inherent_methods_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let source = packet.fixture("provider");
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "named, associated or generic alias transport requires its wider profile",
        )?;
    }
    Ok(())
}
#[test]
fn older_profiles_and_unsupported_const_methods_reject_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let source = generated(&packet, "plain.rs", &format!("{nominals}{PLAIN}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("plain-{profile}.rgcrate"));
        packet.rejected(
            &format!("plain-{profile}"),
            &mut packet.export(profile, &source, &older),
            "source impl owner has no completed canonical conformance record",
        )?;
    }
    let plain = packet.folder.join("plain.rgcrate");
    exported(&packet, &source, &plain)?;
    let consumer = client(&packet, "plain-client", "base::Plain { n: 1 }.get()")?;
    let mut fresh = packet.consumer(&consumer, "plain-client", &plain, &packet.owner());
    super::artifacts::checked("plain-client", &packet.folder, &mut fresh)?;
    let source = generated(&packet, "unsafe.rs", &format!("{nominals}{UNSAFE}"))?;
    let guarded = packet.folder.join("unsafe.rgcrate");
    exported(&packet, &source, &guarded)?;
    let consumer = generated(&packet, "unsafe-client.rs", UNSAFE_CLIENT)?;
    let mut fresh = packet.consumer(&consumer, "unsafe-client", &guarded, &packet.owner());
    super::artifacts::checked("unsafe-client", &packet.folder, &mut fresh)?;
    let source = generated(&packet, "temporary.rs", &format!("{nominals}{TEMPORARY}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("temporary-{profile}.rgcrate"));
        packet.rejected(
            &format!("temporary-{profile}"),
            &mut packet.export(profile, &source, &older),
            "source operation omitted a required nontrivial formation input",
        )?;
    }
    let temporary = packet.folder.join("temporary.rgcrate");
    exported(&packet, &source, &temporary)?;
    let consumer = generated(
        &packet,
        "temporary-client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::f() { 42 => 42, _ => 7 } }\n",
    )?;
    let mut fresh = packet.consumer(&consumer, "temporary-client", &temporary, &packet.owner());
    super::artifacts::checked("temporary-client", &packet.folder, &mut fresh)?;
    let source = generated(
        &packet,
        "temporary-write.rs",
        &format!("{nominals}{TEMPORARY}{TEMPORARY_WRITE}"),
    )?;
    let metadata = packet.folder.join("temporary-write.rgcrate");
    packet.rejected(
        "temporary-write",
        &mut packet.export(PROFILE, &source, &metadata),
        "source Store writes through an immutable dereference",
    )?;
    for (label, control, diagnostic) in FENCED {
        let source = generated(
            &packet,
            &format!("{label}.rs"),
            &format!("{nominals}{control}"),
        )?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        packet.rejected(
            label,
            &mut packet.export(PROFILE, &source, &metadata),
            diagnostic,
        )?;
    }
    Ok(())
}
/// Every `&self` method header with an original: receiver flag, formal 0,
/// base, one reference, an immutable binding, constness and body tag 1.
/// Returns each constness offset in packet order.
fn receivers(bytes: &[u8]) -> Vec<usize> {
    bytes
        .windows(16)
        .enumerate()
        .filter(|(_, row)| {
            row[..5] == [1, 0, 0, 0, 0]
                && row[9..14] == [1, 0, 0, 0, 0]
                && row[14] <= 1
                && row[15] == 1
        })
        .map(|(offset, _)| offset + 14)
        .collect()
}
/// The constness offsets of `Gate`'s `get` and `slow` headers.
fn gate_methods(bytes: &[u8]) -> Result<(usize, usize)> {
    let found = receivers(bytes);
    let constant: Vec<usize> = found
        .iter()
        .copied()
        .filter(|&offset| bytes[offset] == 1)
        .collect();
    let [get] = *constant.as_slice() else {
        return Err(format!("const receiver headers {constant:?}").into());
    };
    // `Gate` is the provider's last item: `slow`'s header follows `get`'s.
    let following: Vec<usize> = found
        .iter()
        .copied()
        .filter(|&offset| offset > get)
        .collect();
    let [slow] = *following.as_slice() else {
        return Err(format!("receiver headers after `get` {following:?}").into());
    };
    if bytes[slow] != 0 {
        return Err("`slow`'s header is not ordinary".into());
    }
    Ok((get, slow))
}
/// A function header's constness follows 117 fixed bytes: declaration,
/// scope, parent, both signatures, region scheme, inputs, formals and
/// receiver.
const HEADER_PREFIX: usize = 117;
/// The context byte of the original row that a header's body names. The
/// original rows follow the headers; each leads with its body and
/// declaration, then its result and sites.
fn context(bytes: &[u8], constness: usize) -> Result<usize> {
    let start = constness
        .checked_sub(HEADER_PREFIX)
        .ok_or("header precedes the packet")?;
    let declaration = bytes.get(start..start + 4).ok_or("header declaration")?;
    let body = bytes
        .get(constness + 2..constness + 6)
        .ok_or("header body")?;
    let originals = bytes.get(constness..).ok_or("header constness")?;
    Ok(constness + unique(originals, &[body, declaration].concat())? + 16)
}
fn flipped(bytes: &[u8], flags: &[(usize, u8)]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    for &(offset, value) in flags {
        changed[offset] = value;
    }
    changed
}
const CONTEXTUAL: &str = "source contextual/borrow/call original requires a wider explicit profile";
type Forgery = (&'static str, Vec<u8>, &'static str);
fn forgeries(honest: &[u8]) -> Result<Vec<Forgery>> {
    let (get, slow) = gate_methods(honest)?;
    let defining = context(honest, get)?;
    if honest[defining] != 1 || honest[context(honest, slow)?] != 0 {
        return Err("`get` and `slow` originals lost their defining contexts".into());
    }
    // A header flag alone contradicts its original's context; both flags
    // together forge a coherent ordinary `get`.
    Ok(vec![
        ("get-ordinary", flipped(honest, &[(get, 0)]), CONTEXTUAL),
        (
            "get-runtime",
            flipped(honest, &[(get, 0), (defining, 0)]),
            NONCONST,
        ),
        ("slow-const", flipped(honest, &[(slow, 1)]), CONTEXTUAL),
        (
            "version-12",
            relabel(honest, 12),
            "unknown source declaration kind",
        ),
    ])
}
/// `slow` forged const with a const-definition context.
fn defining(bytes: &[u8]) -> Result<Vec<u8>> {
    let (_, slow) = gate_methods(bytes)?;
    Ok(flipped(bytes, &[(slow, 1), (context(bytes, slow)?, 1)]))
}
#[test]
fn forged_const_method_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{GATE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let user = client(&packet, "gate", "base::Gate { t: &1, n: 42 }.get()")?;
    let mut honest = packet.consumer(&user, "gate", &packet.metadata, &packet.owner());
    super::artifacts::checked("gate", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    for (name, wire, diagnostic) in forgeries(&bytes)? {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, wire)?;
        let mut consumer = packet.consumer(&user, name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Declared constness is never evidence: each consumer re-checks the
    // forged const `slow`, accepting `self.n` and rejecting a runtime call.
    let metadata = packet.folder.join("slow-defining.rgcrate");
    std::fs::write(&metadata, defining(&bytes)?)?;
    let mut consumer = packet.consumer(&user, "slow-defining", &metadata, &packet.owner());
    super::artifacts::checked("slow-defining", &packet.folder, &mut consumer)?;
    let source = generated(&packet, "calling.rs", &format!("{foundation}{CALLING}"))?;
    let calling = packet.folder.join("calling.rgcrate");
    exported(&packet, &source, &calling)?;
    let metadata = packet.folder.join("slow-calling.rgcrate");
    std::fs::write(&metadata, defining(&std::fs::read(&calling)?)?)?;
    let mut consumer = packet.consumer(&user, "slow-calling", &metadata, &packet.owner());
    packet.rejected(
        "slow-calling",
        &mut consumer,
        "source const-definition call requires its actual const target",
    )?;
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn const_method_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "wide")?;
    let foundation = foundation(&packet)?;
    let consumer = client(&packet, "client", "base::Wide { t: &1, n: 42 }.m005()")?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut methods = String::new();
        for index in 0..width {
            write!(
                methods,
                " pub const fn m{index:03}(&self) -> u8 {{ self.n }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("wide{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub struct Wide<'a> {{ pub t: &'a u8, pub n: u8 }}\nimpl<'a> Wide<'a> {{{methods} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("wide{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every method adds the same declaration, binding,
    // header and original bytes.
    let ([small, middle, large], [first, _, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(first.abs_diff(third) <= 64, "{lowir:?}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn const_method_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
