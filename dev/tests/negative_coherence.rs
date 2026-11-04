//! Actual positive dispatch is selected using genuine explicit negative coherence.
#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn explicit_negative_coherence_selects_actual_positive_runtime_implementations()
-> Result<(), Box<dyn std::error::Error>> {
    crate::observed_program::compare(
        "negative-coherence-runtime",
        "pa31/tests/spec/negative-coherence",
        b"7\n9\n",
    )
}
