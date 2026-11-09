//! Fresh CLI processes form every imported nominal a client names under its
//! transported where-clauses, as rustc does. A client's signature, field,
//! reference or impl header over `base::Rc<u8, NotAlloc>` is rejected at the
//! same kind of site as its source-local twin, and a satisfying argument, a
//! caller bound or a client impl of the imported trait is accepted.
use super::fresh_packets::{Packet, Result};
const SPEC: &str = "pa29/tests/spec/imported-formation";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
const DECLARED: &str = "nominal type argument must implement its declared trait";
const SIZED: &str = "nominal type argument must be sized";
/// Frozen outcomes from `pa29/tests/imported-formation-outcomes.json`.
const ACCEPTED: [&str; 6] = [
    "rc-global",
    "rc-generic-bound",
    "rc-client-impl",
    "rc-unsized",
    "bounded-copy",
    "local-positive",
];
/// Each opponent's diagnostic is located at its own item, `file 0: OFFSET:`:
/// a signature at its function, a field at the field, an impl at the impl.
const REJECTED: [(&str, &str, &str); 12] = [
    (
        "rc-not-alloc",
        "fn hold(_r: base::Rc<u8, NotAlloc>) {}",
        DECLARED,
    ),
    (
        "rc-generic-unbound",
        "fn hold<A>(_r: base::Rc<u8, A>) {}",
        DECLARED,
    ),
    ("rc-field", "r: base::Rc<u8, NotAlloc>", DECLARED),
    (
        "rc-reference",
        "fn hold(_r: &base::Rc<u8, NotAlloc>) {}",
        DECLARED,
    ),
    (
        "impl-header",
        "impl Mine for base::Rc<u8, NotAlloc> {}",
        DECLARED,
    ),
    (
        "bounded-not-copy",
        "fn hold(_b: base::Bounded<NotCopy>) {}",
        DECLARED,
    ),
    (
        "nested-not-alloc",
        "fn hold(_n: base::Nested<NotAlloc>) {}",
        DECLARED,
    ),
    ("slot-unsized", "fn hold(_s: &base::Slot<str>) {}", SIZED),
    (
        "local-control",
        "fn hold(_r: LocalRc<NotAlloc>) {}",
        DECLARED,
    ),
    ("local-field", "r: LocalRc<NotAlloc>", DECLARED),
    (
        "local-impl-header",
        "impl Mine for LocalRc<NotAlloc> {}",
        DECLARED,
    ),
    (
        "local-generic-unbound",
        "fn hold<A>(_r: LocalRc<A>) {}",
        DECLARED,
    ),
];
/// The diagnostic located at the one `site` in the client's fixture text.
fn located(packet: &Packet, name: &str, site: &str, diagnostic: &str) -> Result<String> {
    let text = std::fs::read_to_string(packet.fixture(name))?;
    let offset = text.find(site).ok_or("located item")?;
    assert_eq!(text.matches(site).count(), 1, "{name}: one located item");
    Ok(format!("file 0: {offset}: {diagnostic}"))
}
#[test]
fn clients_form_imported_nominals_under_their_where_clauses() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "formation-frontend", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, site, diagnostic) in REJECTED {
        let diagnostic = located(&packet, name, site, diagnostic)?;
        packet.rejected(name, &mut packet.client(name), &diagnostic)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn formation_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "formation-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
