//! Fresh CLI processes prove coherence absence from the perspective of the
//! crate whose coherence is proved. A consumer re-proves an imported packet's
//! same-head pair from that packet's perspective, over what the packet could
//! see, and a client's own pairs from the client's. Absence is knowable when
//! that crate owns the trait or an input type through `&`, `&mut` and
//! fundamental nominals, and no downstream crate could implement the goal.
use super::fresh_packets::{Packet, Result};
use std::path::{Path, PathBuf};
const SPEC: &str = "pa29/tests/spec/knowable-absence";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const ORPHAN: &str = "library trait implementation requires a local type";
/// Frozen outcomes from `pa29/tests/knowable-absence-outcomes.json`.
const ACCEPTED: [&str; 6] = [
    "answer",
    "answer-local-impl",
    "local-error-ref",
    "local-mut-ref",
    "local-copy",
    "trait-argument",
];
/// Each opponent is rejected at its own impl, `file 0: OFFSET:`, never at
/// the imported pair, which has no source key.
const REJECTED: [(&str, &str, &str); 7] = [
    ("foreign-self", "impl Mine for base::Counter {}", OVERLAP),
    ("uncovered", "impl<T> Mine for &T {}", OVERLAP),
    ("tuple-self", "impl Mine for (Local, u8) {}", OVERLAP),
    (
        "nonfundamental-wrapper",
        "impl Mine for base::Slot<Local> {}",
        OVERLAP,
    ),
    ("local-uncovered", "impl<T> Mine for &T {}", OVERLAP),
    ("orphan-blanket", "impl<T> base::Error for T {}", ORPHAN),
    ("orphan-str", "impl base::Error for str {}", ORPHAN),
];
/// The provider's own traits for the generated controls. Each blanket's
/// condition names one with a `?Sized` parameter; `Seal`'s negative impl is
/// `?Sized` (a `MetaSized` premise) and `Mark`'s is `PointeeSized`.
const SEAL: [&str; 8] = [
    "pub trait Seal {}",
    "impl<T: ?Sized> !Seal for &T {}",
    "pub trait Holds<X: ?Sized> {}",
    "impl<T: Seal + ?Sized, S> Holds<T> for S {}",
    "pub trait Mark {}",
    "impl<T: PointeeSized> !Mark for &T {}",
    "pub trait Keeps<X: ?Sized> {}",
    "impl<T: Mark + ?Sized, S> Keeps<T> for S {}",
];
/// Generated clients: `features`, a client-local `Local`, `item`, and an
/// entry returning 42. `negative-metasized`'s premise `str: MetaSized` is
/// concrete; `negative-metasized-param`'s `U: MetaSized` is covered only by
/// the client impl's own `U: ?Sized`, which `pay_negative` cannot pay yet
/// (Lane B's recorded rejects-valid gap; rustc accepts it).
const CONTROLS: [(&str, &str, &str, bool); 5] = [
    (
        "negative-metasized",
        "no_core,with_negative_coherence",
        "impl base::Holds<&'static str> for Local {}",
        true,
    ),
    (
        "negative-metasized-param",
        "no_core,with_negative_coherence",
        "impl<U: ?base::Sized> base::Holds<&U> for Local {}",
        false,
    ),
    (
        "negative-pointee-param",
        "no_core,with_negative_coherence",
        "impl<U: ?base::Sized> base::Keeps<&U> for Local {}",
        true,
    ),
    (
        "negative-pointee",
        "no_core,with_negative_coherence",
        "impl base::Keeps<&'static str> for Local {}",
        true,
    ),
    (
        "stable-pointee",
        "no_core",
        "impl base::Keeps<&'static str> for Local {}",
        false,
    ),
];
/// The diagnostic located at the one `site` in `path`'s text.
fn located_in(path: &Path, site: &str, diagnostic: &str) -> Result<String> {
    let text = std::fs::read_to_string(path)?;
    let offset = text.find(site).ok_or("located impl")?;
    assert_eq!(text.matches(site).count(), 1, "{site}: one located impl");
    Ok(format!("file 0: {offset}: {diagnostic}"))
}
/// The diagnostic located at the one `impl` in the client's fixture text.
fn located(packet: &Packet, name: &str, site: &str, diagnostic: &str) -> Result<String> {
    located_in(&packet.fixture(name), site, diagnostic)
}
fn generated(packet: &Packet, name: &str, features: &str, items: &str) -> Result<PathBuf> {
    let path = packet.folder.join(format!("{name}.rs"));
    std::fs::write(
        &path,
        format!(
            "#![feature({features})]\n#![no_core]\nextern crate base;\nstruct Local;\n{items}\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ 42 }}\n"
        ),
    )?;
    Ok(path)
}
#[test]
fn packet_and_client_pairs_prove_knowable_absence() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "absence-frontend", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, site, diagnostic) in REJECTED {
        let diagnostic = located(&packet, name, site, diagnostic)?;
        packet.rejected(name, &mut packet.client(name), &diagnostic)?;
    }
    Ok(())
}
/// Coherence reads an imported impl's where-clauses as selection does, in the
/// provider's terms. A condition that holds keeps its pair overlapping, and a
/// `with_negative_coherence` client excludes a pair only through a negative
/// impl whose premise it can pay: `PointeeSized`, not yet `MetaSized`.
#[test]
fn generated_controls_read_imported_conditions() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "absence-gates", "provider", PROFILE)?;
    let site = "impl base::Error for &Local {}";
    let holds = generated(
        &packet,
        "condition-holds",
        "no_core",
        &format!("impl base::Error for Local {{}}\n{site}"),
    )?;
    let mut consumer =
        packet.consumer(&holds, "condition-holds", &packet.metadata, &packet.owner());
    packet.rejected(
        "condition-holds",
        &mut consumer,
        &located_in(&holds, site, OVERLAP)?,
    )?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let seal = packet.folder.join("seal.rs");
    std::fs::write(
        &seal,
        format!("{fixture}// Generated control.\n{}\n", SEAL.join("\n")),
    )?;
    let metadata = packet.folder.join("seal.rgcrate");
    let mut export = packet.export(PROFILE, &seal, &metadata);
    super::artifacts::checked("seal", &packet.folder, &mut export)?;
    for (name, features, item, accepted) in CONTROLS {
        let source = generated(&packet, name, features, item)?;
        let mut consumer = packet.consumer(&source, name, &metadata, &packet.owner());
        if accepted {
            super::artifacts::checked(name, &packet.folder, &mut consumer)?;
        } else {
            packet.rejected(name, &mut consumer, &located_in(&source, item, OVERLAP)?)?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn absence_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "absence-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
