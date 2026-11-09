//! Fresh CLI processes consume version 13's zero-slot fn fields. Each
//! consumer forms a struct's or variant's closed fn-pointer field in that
//! nominal's own header frame, a called constructor's payload in its own
//! nominal's, and calls through the field with a signature its own function
//! owns. Auto traits and variance read the field as rustc does. Higher-ranked,
//! early-region and C fields keep their fences by name, and version twelve
//! rejects the frame. An unsafe field call in an associated fn exports since
//! the guarded-methods item, and a record variant's fn field since (i).
use super::fresh_packets::{Packet, Result};
use rustgm::support::crates;
const SPEC: &str = "pa29/tests/spec/callable-fields";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/callable-field-outcomes.json`.
const ACCEPTED: [&str; 15] = [
    "via", "slot-new", "method", "store", "table", "op", "op-new", "choose", "computed", "apply",
    "pair", "send", "widen", "inv", "markers",
];
const CALLABLE: &str = "source anonymous callable requires its supported closed Rust frame";
/// Each generated provider opponent appends these lines to the frozen
/// provider after `// Generated callable-field opponent.`.
const OPPONENTS: [(&str, &str, &str); 4] = [
    ("hr", "pub struct Hr { pub f: fn(&u8) -> u8 }\n", CALLABLE),
    (
        "region",
        "pub struct Ctx<'a> { pub m: PhantomData<fn(&'a ()) -> &'a ()> }\n",
        CALLABLE,
    ),
    (
        "c",
        "pub struct Cf { pub f: extern \"C\" fn(u8) -> i32 }\n",
        CALLABLE,
    ),
    (
        "argument-type",
        "pub enum ArgumentType<'a> { Placeholder { value: *const (), formatter: unsafe fn(*const (), &mut u8) -> i32, _lifetime: PhantomData<&'a ()> }, Count(u16) }\n",
        // Amendment 4: the record variant exports, and its formatter's elided
        // `&mut` is a late-bound region.
        CALLABLE,
    ),
];
/// The former `record-variant` opponent, which exports since (i): outcomes
/// amendment 4. Its native witness is `braced_variants`'.
pub(super) const RECORD_VARIANT: &str = "pub enum Kind { Placeholder { value: u8, formatter: unsafe fn(u8, u8) -> i32 }, Count(u16) }\n";
/// The former `associated` opponent, which exports since the guarded-methods
/// item: outcomes amendment 3, mover M4.
const ASSOCIATED: &str =
    "impl Table { pub fn go(&self, p: *const ()) -> i32 { unsafe { (self.call)(p) } } }\n";
const ASSOCIATED_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\nunsafe fn call(_p: *const ()) -> i32 { 42 }\nunsafe fn drop_it(_p: *const ()) {}\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { let v = (); let p: *const () = &v; let t = base::Table { call, drop: drop_it }; t.go(p) }\n";
/// The lines the older-profile controls append to an older provider.
const OLDER: &str =
    "// Generated callable-field control.\npub struct Slot { pub f: fn(u8) -> i32 }\n";
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// Exports mover M4's provider and consumes its client.
fn associated(packet: &Packet) -> Result {
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let source = packet.folder.join("associated.rs");
    let text = format!("{provider}// Generated callable-field opponent.\n{ASSOCIATED}");
    std::fs::write(&source, text)?;
    let metadata = packet.folder.join("associated.rgcrate");
    let mut export = packet.export(PROFILE, &source, &metadata);
    super::artifacts::checked("associated", &packet.folder, &mut export)?;
    let client = packet.folder.join("associated-client.rs");
    std::fs::write(&client, ASSOCIATED_CLIENT)?;
    let label = "associated-client";
    let mut consumer = packet.consumer(&client, label, &metadata, &packet.owner());
    super::artifacts::checked(label, &packet.folder, &mut consumer).map(drop)
}
#[test]
fn clients_call_fn_fields_and_wider_fields_keep_their_fences() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "field-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    // Contra<T> is contravariant in T, so narrowing its lifetime is unsound.
    packet.rejected(
        "narrow",
        &mut packet.client("narrow"),
        "required lifetime does not outlive its target",
    )?;
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    for (name, lines, diagnostic) in OPPONENTS {
        let source = packet.folder.join(format!("{name}.rs"));
        let text = format!("{provider}// Generated callable-field opponent.\n{lines}");
        std::fs::write(&source, text)?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        packet.rejected(
            name,
            &mut packet.export(PROFILE, &source, &metadata),
            diagnostic,
        )?;
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    let source = packet.folder.join("record-variant.rs");
    let text = format!("{provider}// Generated callable-field opponent.\n{RECORD_VARIANT}");
    std::fs::write(&source, text)?;
    let metadata = packet.folder.join("record-variant.rgcrate");
    let mut export = packet.export(PROFILE, &source, &metadata);
    super::artifacts::checked("record-variant", &packet.folder, &mut export)?;
    associated(&packet)
}
#[test]
fn version_twelve_rejects_fn_fields_and_thirteen_exports_them() -> Result {
    let packet = Packet::new(GROUP, SPEC, "field-older", "provider")?;
    let older = |provider: &str| -> Result<std::path::PathBuf> {
        let text = std::fs::read_to_string(packet.root.join(provider))?;
        let source = packet.folder.join(provider.replace('/', "-"));
        std::fs::write(&source, format!("{text}{OLDER}"))?;
        Ok(source)
    };
    let branches = older("pa29/tests/spec/source-branches/provider.rs")?;
    let twelve = packet.folder.join("twelve.rgcrate");
    let mut export = packet.export("module-branches-v12", &branches, &twelve);
    packet.rejected(
        "twelve",
        &mut export,
        "source shared call type requires an actual closed callable frame",
    )?;
    assert!(unwritten(&twelve)?, "version twelve wrote a packet");
    let thirteen = packet.folder.join("thirteen.rgcrate");
    let mut export = packet.export(PROFILE, &branches, &thirteen);
    super::artifacts::checked("thirteen", &packet.folder, &mut export)?;
    // The exporter's pre-existing arena fence, outside this item.
    let nominals = older("pa29/tests/spec/source-nominals/provider.rs")?;
    let arena = packet.folder.join("arena.rgcrate");
    packet.rejected(
        "arena",
        &mut packet.export(PROFILE, &nominals, &arena),
        "source anonymous callable requires its actual prepared type arena",
    )?;
    assert!(unwritten(&arena)?, "the arena control wrote a packet");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn fn_field_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "field-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    associated(&packet)?;
    let output = packet.native("associated-client")?;
    assert_eq!(output.status.code(), Some(42));
    assert!(output.stdout.is_empty());
    Ok(())
}
