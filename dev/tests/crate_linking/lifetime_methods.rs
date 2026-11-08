//! Fresh CLI processes re-prove version-thirteen methods of inherent impls
//! that capture lifetimes: the impl's lifetimes lead each method's original
//! and public frames, provider bodies call them with the captured regions as
//! leading application arguments, and each consumer rebuilds their templates.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/impl-region-methods";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "impl-region-methods";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/impl-region-method-outcomes.json`.
const ACCEPTED: [&str; 13] = [
    "new",
    "from-static",
    "get",
    "text",
    "pick",
    "cell",
    "pair",
    "slot",
    "info",
    "path",
    "make",
    "relay",
    "wrap",
];
const OUTLIVES: &str = "required lifetime does not outlive its target";
const OPPONENTS: [(&str, &str); 4] = [
    ("text-escape", OUTLIVES),
    ("info-local", OUTLIVES),
    ("cell-shrink", OUTLIVES),
    ("pick-borrow", "access conflicts with a live reference loan"),
];
const HELD: &str = "// Generated captured-lifetime gate.\npub struct Held<'a> { pub text: &'a str }\nimpl<'a> Held<'a> { pub fn get(self) -> &'a str { self.text } }\n";
/// `spec/source-nominals/provider.rs` has no `Copy` for `u8`, so the client
/// keeps the captured `'static` result without reading through it.
const HELD_CLIENT: &str = "let _text: &'static str = base::Held { text: \"hi\" }.get(); 42";
/// Const captured methods travel since the const inherent-method checkpoint.
const CONSTANT: &str = "// Generated const captured-method gate.\npub struct Held<'a> { pub text: &'a str }\nimpl<'a> Held<'a> { pub const fn count(&self) -> u8 { 42 } }\n";
const CONSTANT_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\nconst N: u8 = base::Held { text: \"hi\" }.count();\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match N { 42 => 42, _ => 7 } }\n";
const FENCED: [(&str, &str, &str); 2] = [
    (
        "trait-impl",
        "// Generated captured trait-impl gate.\npub struct Held<'a> { pub text: &'a str }\npub trait Get { fn get(self) -> u8; }\nimpl<'a> Get for Held<'a> { fn get(self) -> u8 { 42 } }\n",
        "source function early/captured callable frame transport is not implemented",
    ),
    (
        "early",
        "// Generated early captured-method gate.\npub struct Held<'a> { pub text: &'a str }\nimpl<'a> Held<'a> { pub fn shorter<'b>(&self, x: &'b u8) -> &'b u8 where 'a: 'b { x } }\n",
        "source function early/captured callable frame transport is not implemented",
    ),
];
const WIRE: &str = "// Generated captured-method wire control.\npub struct Hold<'a> { pub t: &'a u8 }\nimpl<'a> Hold<'a> { pub fn choose<'b>(&self, a: &'a u8, b: &'b u8) -> &'b u8 { b } }\n";
const VARIANT: &str = "// Generated captured-method wire control.\npub struct Hold<'a> { pub t: &'a u8 }\nimpl<'a> Hold<'a> { pub fn choose<'b>(&self, a: &'a u8, b: &'b u8) -> &'a u8 { a } }\n";
const CHOOSE: &str = "let x = 42u8; let y = 7u8; let h = base::Hold { t: &x }; match *h.choose(&x, &y) { 7 => 42, _ => 8 }";
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
        .split("// Authored captured-region method transport controls.")
        .next()
        .ok_or("captured-method foundation")?
        .to_owned())
}
fn client(packet: &Packet, name: &str, body: &str) -> Result<std::path::PathBuf> {
    generated(
        packet,
        &format!("{name}.rs"),
        &format!(
            "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ {body} }}\n"
        ),
    )
}
#[test]
fn captured_methods_survive_wire_and_fresh_consumers() -> Result {
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
fn older_profiles_and_unsupported_captured_shapes_reject_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let source = generated(&packet, "held.rs", &format!("{nominals}{HELD}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("held-{profile}.rgcrate"));
        packet.rejected(
            &format!("held-{profile}"),
            &mut packet.export(profile, &source, &older),
            "source lifetime binding has no actual original slot",
        )?;
    }
    let held = packet.folder.join("held.rgcrate");
    exported(&packet, &source, &held)?;
    let consumer = client(&packet, "held-client", HELD_CLIENT)?;
    let mut fresh = packet.consumer(&consumer, "held-client", &held, &packet.owner());
    super::artifacts::checked("held-client", &packet.folder, &mut fresh)?;
    let source = generated(&packet, "const.rs", &format!("{nominals}{CONSTANT}"))?;
    let constant = packet.folder.join("const.rgcrate");
    exported(&packet, &source, &constant)?;
    let consumer = generated(&packet, "const-client.rs", CONSTANT_CLIENT)?;
    let mut fresh = packet.consumer(&consumer, "const-client", &constant, &packet.owner());
    super::artifacts::checked("const-client", &packet.folder, &mut fresh)?;
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
fn word(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
fn unique_offset(found: &[usize], what: &str) -> Result<usize> {
    match found {
        [offset] => Ok(*offset),
        _ => Err(format!("{what} {found:?}").into()),
    }
}
/// `choose`'s function header: public and original signatures over three
/// parameters, then a region scheme of three slots, one captured and two
/// late. Returns the offset of the public signature.
fn choose_header(bytes: &[u8]) -> Result<usize> {
    let signature = |offset: usize| {
        word(bytes, offset + 8)
            .zip(word(bytes, offset + 4))
            .is_some_and(|(end, start)| end.wrapping_sub(start) == 3)
            && bytes.get(offset + 16..offset + 19) == Some([0, 0, 0].as_slice())
    };
    let regions = |offset: usize| {
        let field = |index: usize| word(bytes, offset + 4 * index);
        field(0) == Some(3)
            && field(1) == Some(1)
            && field(2).is_some()
            && field(2) == field(3)
            && field(5)
                .zip(field(4))
                .is_some_and(|(end, start)| end.wrapping_sub(start) == 2)
    };
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| signature(offset) && signature(offset + 19) && regions(offset + 38))
        .collect();
    unique_offset(&found, "captured-method function headers")
}
/// `Hold`'s inherent impl header: Self definition and scope (never the root
/// declaration 0), Self type, no trait or modifiers, one captured lifetime,
/// no region bounds and one member. Returns the offset of its capture count.
fn hold_impl(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| {
            bytes.get(offset..offset + 48).is_some_and(|header| {
                word(header, 0) == word(header, 4)
                    && word(header, 0) != Some(0)
                    && header[12..16].iter().all(|&byte| byte == 0)
                    && word(header, 16) == Some(1)
                    && word(header, 20) == word(header, 24)
                    && word(header, 44)
                        .zip(word(header, 40))
                        .is_some_and(|(end, start)| end.wrapping_sub(start) == 1)
            })
        })
        .collect();
    unique_offset(&found, "captured impl headers").map(|offset| offset + 16)
}
/// `choose`'s two own region slot rows: owner, ordinal, index, no
/// definition and the region kind, at indices 1 and 2. Returns the offset
/// of the first row's index.
fn choose_slots(bytes: &[u8]) -> Result<usize> {
    let row = |offset: usize, ordinal: u32| {
        word(bytes, offset + 4) == Some(ordinal)
            && word(bytes, offset + 8) == Some(ordinal + 1)
            && bytes.get(offset + 12..offset + 14) == Some([0, 1].as_slice())
    };
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| {
            row(offset, 0) && row(offset + 14, 1) && word(bytes, offset) == word(bytes, offset + 14)
        })
        .collect();
    unique_offset(&found, "captured-method late slot rows").map(|offset| offset + 8)
}
type Forgery = (&'static str, Vec<u8>, &'static str);
/// The method-family admission rejects every disagreement between a method's
/// captures and its impl's before the binder and scheme checks run.
const FRAME: &str = "source captured method changed its impl's exact lifetime frame";
fn forgeries(honest: &[u8], claimed: &[u8]) -> Result<Vec<Forgery>> {
    let header = choose_header(honest)?;
    if choose_header(claimed)? != header {
        return Err("the variant moved `choose`'s header".into());
    }
    let result = |wire: &[u8], offset: usize| -> Result<Vec<u8>> {
        Ok(with_id(
            wire,
            offset,
            word(claimed, offset).ok_or("claimed result")?,
        ))
    };
    let (public, original) = (header + 12, header + 31);
    let regions = header + 38;
    let captures = hold_impl(honest)?;
    let slot = choose_slots(honest)?;
    Ok(vec![
        (
            "claimed-result",
            result(&result(honest, public)?, original)?,
            OUTLIVES,
        ),
        (
            "claimed-callable",
            result(honest, public)?,
            "source signature reference mutability or exact lifetime differs",
        ),
        ("no-captures", with_id(honest, regions + 4, 0), FRAME),
        ("two-captures", with_id(honest, regions + 4, 2), FRAME),
        ("impl-captures", with_id(honest, captures, 0), FRAME),
        (
            "captured-slot",
            with_id(honest, slot, 0),
            "source late opening has foreign, repeated or nonregional formals",
        ),
        (
            "version-12",
            relabel(honest, 12),
            "unknown source declaration kind",
        ),
    ])
}
#[test]
fn forged_captured_method_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let variant = generated(&packet, "variant.rs", &format!("{foundation}{VARIANT}"))?;
    let claimed = packet.folder.join("variant.rgcrate");
    exported(&packet, &variant, &claimed)?;
    let user = client(&packet, "choose", CHOOSE)?;
    let mut honest = packet.consumer(&user, "choose", &packet.metadata, &packet.owner());
    super::artifacts::checked("choose", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    for (name, wire, diagnostic) in forgeries(&bytes, &std::fs::read(&claimed)?)? {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, wire)?;
        let mut consumer = packet.consumer(&user, name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn captured_method_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "methods")?;
    let foundation = foundation(&packet)?;
    let consumer = client(
        &packet,
        "client",
        "let v: u8 = 42; let w = base::Wide { t: &v }; match *w.m005() { 42 => 42, _ => 7 }",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut methods = String::new();
        for index in 0..width {
            write!(
                methods,
                "pub fn m{index:03}(&self) -> &'a u8 {{ &*self.t }} "
            )?;
        }
        let source = generated(
            &packet,
            &format!("methods{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub struct Wide<'a> {{ pub t: &'a u8 }}\nimpl<'a> Wide<'a> {{ {methods}}}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("methods{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every method adds the same declaration, slot,
    // binder, region, header and body bytes.
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
fn captured_method_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
