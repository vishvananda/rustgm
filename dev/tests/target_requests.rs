//! A foreign request cannot alter the host target or ordinary scalar execution.
#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn original_core_target_requests_preserve_baseline_calls_and_configuration()
-> Result<(), Box<dyn std::error::Error>> {
    crate::observed_program::compare(
        "architecture-request-runtime",
        "pa41/tests/spec/architecture-requests",
        b"7\n9\n",
    )
}
