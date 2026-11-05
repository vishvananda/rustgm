# Pin receiver and async signature composition

Three complete authored sources combine the existing actual Pin/Deref/Target/
LegacyReceiver foundation with genuine Future/Output and a required async method.
The manifest freezes0/1/1 before candidate outcomes; pinned Rust1.98.0 confirms
all three. Missing LegacyReceiver and wrong Self remain semantic opponents.
The positive retains actual Self/lifetime captures and hidden=None.

This ordinary authored trait is a prerequisite composition test. It does not
admit the AsyncDrop role, alter original core or provide async state/runtime.
`async_signatures::source_pin_and_async_signatures_require_real_receiver_proof`
owns the maintainer check. See the [joined checkpoint](../../../../docs/pa42-async-signatures-checkpoint.md).
