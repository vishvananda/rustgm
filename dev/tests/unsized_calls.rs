//! Actual ?Sized specialization and automatic destruction through a real C observer.
#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn generic_unsized_parameter_original_executes_sized_instance_and_drop()
-> Result<(), Box<dyn std::error::Error>> {
    crate::observed_program::compare(
        "unsized-parameter-runtime",
        "pa30/tests/spec/unsized-parameters",
        b"7\n9\n",
    )
}
