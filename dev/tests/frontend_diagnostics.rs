//! Reviewed warning bytes never authorize arbitrary successful stderr.
use crate::native_references::frontend_diagnostics::accepted;
use std::{
    os::unix::process::ExitStatusExt,
    process::{ExitStatus, Output},
};
fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status: ExitStatus::from_raw(status << 8),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}
#[test]
fn declared_successful_warnings_preserve_the_strict_compiler_boundary() {
    let warning = b"warning[unused_must_use]: file 0: 37: unused must_use result\n";
    assert!(accepted(&output(0, b"program", b""), b"").is_ok());
    assert!(accepted(&output(0, b"program", warning), warning).is_ok());
    for (status, stdout, actual, expected) in [
        (0, b"program".as_slice(), warning.as_slice(), b"".as_slice()),
        (0, b"program".as_slice(), b"".as_slice(), warning.as_slice()),
        (
            1,
            b"program".as_slice(),
            warning.as_slice(),
            warning.as_slice(),
        ),
        (0, b"".as_slice(), warning.as_slice(), warning.as_slice()),
        (
            0,
            b"program".as_slice(),
            b"ERROR: failure\n".as_slice(),
            b"ERROR: failure\n".as_slice(),
        ),
    ] {
        assert!(accepted(&output(status, stdout, actual), expected).is_err());
    }
    let added = [warning.as_slice(), warning.as_slice()].concat();
    assert!(accepted(&output(0, b"program", &added), warning).is_err());
    let error_after_warning = [warning.as_slice(), b"error: failure\n".as_slice()].concat();
    assert!(
        accepted(
            &output(0, b"program", &error_after_warning),
            &error_after_warning
        )
        .is_err()
    );
    let unknown = b"warning[unknown]: file 0: 37: unsupported code\n";
    assert!(accepted(&output(0, b"program", unknown), unknown).is_err());
}

#[test]
fn declared_attribute_warning_codes_do_not_relax_byte_comparison() {
    for code in [
        "malformed_diagnostic_attributes",
        "malformed_diagnostic_format_literals",
        "misplaced_diagnostic_attributes",
    ] {
        let warning = format!("warning[{code}]: file 0: 37: declared attribute diagnostic\n");
        assert!(
            accepted(
                &output(0, b"program", warning.as_bytes()),
                warning.as_bytes()
            )
            .is_ok()
        );
        assert!(accepted(&output(0, b"program", warning.as_bytes()), b"").is_err());
        let error = format!("{warning}error: failure\n");
        assert!(accepted(&output(0, b"program", error.as_bytes()), error.as_bytes()).is_err());
    }
}
