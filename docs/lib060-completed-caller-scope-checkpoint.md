# LIB-060 completed ordinary caller scopes

The sealed caller adapter previously rejected the genuine ordinary
`PanicInfo::location` method: lifetime-only/empty declaration scopes can correctly
finish without a declaration environment Arc. Their existing scope rows therefore
looked incomplete to the conservative adapter.

Actual body checking now builds that missing canonical scope once, passes it
through the existing defining/body-alias completion, and retains only the final
environment in the existing body environment map. This includes an unchanged
empty environment. A function with no scope rows still allocates nothing. The
sealed getter remains unchanged and still rejects genuinely incomplete scopes.

Source attributes pay through their actual producer. Ordinary attribute-free
bodies charge the existing bounded body work ledger, with a 64 MiB retained
map/Arc envelope checked before new Arc allocation and map publication. Failed
attempts retain their work charge. No body demand, per-use scope scan, callee
premise, new dense record or permission is introduced.

Private gate `1793222062561293891` passes all five selected controls in 244.95s,
including strict private Clippy, formatting, architecture checks and the actual
original-source handler path through checked LowIR. The existing caller authority
opponents and cleanup reuse pass. The added lifetime-only method retains its
final Arc; a function with absent scope rows stays allocation-free. Existing
Store/Environment/NodeFacts and cold queue layout assertions still pass.

Two filename-stem audit failures and a test-only private-field access failure are
retained in the evidence. Production/CLI and broader joined source/native
qualification remain pending. This is no assignment, metadata transport, original
core or release completion claim. Source hashes and receipts are recorded under
`obj/parallel/completed-caller-environment-v1/`.
