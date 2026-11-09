//! Fresh CLI clients of version-thirteen packets const-evaluate imported and
//! local bodies whose Copy decisions demand imported impls before the eager
//! walk. Each demand pays the walk's own conformance, which the walk then
//! skips; overlap pairs keep their check, and a client-local Copy impl keeps a
//! named fence.
use super::fresh_packets::{Packet, Result};
const SPEC: &str = "pa29/tests/spec/scheduling-context";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/scheduling-context-outcomes.json`: each
/// provider and the clients rustc and the frontend accept against it.
const ACCEPTED: [(&str, &[&str]); 6] = [
    (
        "none-provider",
        &[
            "four",
            "suffixed",
            "placed",
            "body",
            "local-placed",
            "local-literal",
        ],
    ),
    (
        "usize-provider",
        &[
            "four",
            "suffixed",
            "placed",
            "local-placed",
            "minspec",
            "body",
            "local-literal",
        ],
    ),
    (
        "u8-provider",
        &[
            "four",
            "suffixed",
            "placed",
            "body",
            "local-placed",
            "local-literal",
        ],
    ),
    (
        "bool-provider",
        &[
            "four",
            "suffixed",
            "placed",
            "body",
            "local-placed",
            "local-literal",
        ],
    ),
    ("nominal-provider", &["four", "placed", "local-placed"]),
    ("primitives-provider", &["four", "eight", "local-placed"]),
];
/// The accepted clients that the base pin stopped at the scheduling fence.
const DEMANDED: [(&str, &[&str]); 4] = [
    (
        "usize-provider",
        &["four", "suffixed", "placed", "local-placed", "minspec"],
    ),
    ("u8-provider", &["four", "suffixed", "placed"]),
    ("bool-provider", &["four", "suffixed", "placed"]),
    ("primitives-provider", &["four", "eight", "local-placed"]),
];
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const SOURCE_BACKED: &str =
    "client implementation validation before the eager walk requires a source scheduling context";
/// Amendment 1: rustc accepts `generic-usize`, but defining cleanup of the
/// Copy-bounded generic `id` requires the Destruct language item, which the
/// branch provider's first 57 lines lack. The base pin reproduces this
/// existing fence without any scheduling demand.
const CLEANUP: &str = "defining cleanup requires its actual language declaration";
/// A packet whose provider is the fixture `provider`, under the shared
/// `provider` owner identity.
fn published(label: &str, provider: &str) -> Result<Packet> {
    let packet = Packet::new(GROUP, SPEC, label, "provider")?;
    let mut export = packet.export(PROFILE, &packet.fixture(provider), &packet.metadata);
    super::artifacts::checked(provider, &packet.folder, &mut export)?;
    Ok(packet)
}
/// The diagnostic line a rejected client wrote beside its packet.
fn diagnostic(packet: &Packet, label: &str) -> Result<String> {
    let text = std::fs::read_to_string(packet.folder.join(format!("{label}.stderr")))?;
    Ok(text
        .lines()
        .find(|line| line.starts_with("ERROR"))
        .ok_or("rejection without a diagnostic")?
        .to_owned())
}
#[test]
fn clients_validate_imported_impls_on_executable_demand() -> Result {
    for (provider, clients) in ACCEPTED {
        let packet = published(&format!("scheduling-{provider}"), provider)?;
        for &client in clients {
            super::artifacts::checked(client, &packet.folder, &mut packet.client(client))?;
        }
    }
    assert!(DEMANDED.iter().all(|(provider, clients)| {
        ACCEPTED.iter().any(|(accepted, all)| {
            accepted == provider && clients.iter().all(|client| all.contains(client))
        })
    }));
    let packet = published("scheduling-rejected", "blanket-provider")?;
    packet.rejected("overlap-twin", &mut packet.client("overlap-twin"), OVERLAP)?;
    // The base pin stopped `overlap` at the scheduling fence before the walk's
    // pair check; the demand now reaches that check, which reports the twin's
    // diagnostic at the same impl.
    packet.rejected("overlap", &mut packet.client("overlap"), OVERLAP)?;
    assert_eq!(
        diagnostic(&packet, "overlap")?,
        diagnostic(&packet, "overlap-twin")?
    );
    let generic = published("scheduling-generic", "generic-provider")?;
    generic.rejected(
        "local-copy",
        &mut generic.client("local-copy"),
        SOURCE_BACKED,
    )?;
    generic.rejected(
        "generic-usize",
        &mut generic.client("generic-usize"),
        CLEANUP,
    )?;
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn demanded_clients_execute_natively() -> Result {
    for (provider, clients) in DEMANDED {
        let packet = published(&format!("scheduling-native-{provider}"), provider)?;
        for &client in clients {
            super::artifacts::checked(client, &packet.folder, &mut packet.client(client))?;
            let output = packet.native(client)?;
            assert_eq!(output.status.code(), Some(42), "{provider}/{client}");
            assert!(output.stdout.is_empty(), "{provider}/{client}");
        }
    }
    Ok(())
}
