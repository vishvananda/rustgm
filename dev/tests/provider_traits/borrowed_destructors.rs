//! Exact independently settled receiver/capture controls; no signature equality relaxation.
use super::lower;
pub(super) const CASES: &[(&str, &str, bool, u8)] = &[
    (
        "original-elided",
        "drop-captures/original-elided.rs",
        true,
        7,
    ),
    (
        "named-independent",
        "drop-captures/named-independent.rs",
        true,
        7,
    ),
    (
        "captured-receiver",
        "drop-captures/captured-receiver.rs",
        false,
        0,
    ),
    (
        "static-receiver",
        "drop-captures/static-receiver.rs",
        false,
        0,
    ),
    (
        "two-captures-first",
        "drop-captures/two-captures-first.rs",
        true,
        7,
    ),
    (
        "two-captures-second",
        "drop-captures/two-captures-second.rs",
        true,
        11,
    ),
];
#[test]
fn actual_source_drop_captures_preserve_independent_receiver_lifetimes() -> super::TestResult {
    for &(name, source, accepted, _) in CASES {
        let source = super::fixture(source)?;
        let result = lower(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
