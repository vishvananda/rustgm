//! Removed-library data is checked independently of executable API permission.
#[path = "helpers/retired_data.rs"]
mod controls;
use controls::Result;

#[test]
fn checked_removed_declarations_and_requests_match_the_frozen_contract() -> Result {
    let packet = controls::packet();
    let cases = std::fs::read_to_string(packet.join("cases.tsv"))?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, source, profile, expected, guests] = columns.as_slice() else {
            return Err("invalid removed feature contract".into());
        };
        if *profile == "import" {
            let bytes = rustgm::support::crates::encode(&controls::provider()?)?;
            let result = controls::client(rustgm::support::crates::decode(&bytes)?, source);
            assert_eq!(
                result.is_ok(),
                *expected == "0",
                "{name}: {}",
                result
                    .err()
                    .map_or_else(String::new, |error| error.to_string())
            );
        } else {
            let output = crate::loader_markers::run(&packet, name, source, profile, guests)?;
            assert_eq!(
                output.status.code(),
                Some(expected.parse()?),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if *expected == "1" {
                assert!(output.stdout.is_empty(), "{name}");
            }
        }
        count += 1;
    }
    assert_eq!(count, 37);
    Ok(())
}
