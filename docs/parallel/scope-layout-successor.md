# Sparse destruction scope layout assertion

Full42176016850 confirms one stale private assertion independently of its
remaining payer regression: actual Scope is64 bytes; the prior formula expects56.
The scope already retained a cumulative query-work word for paid source builtin
selection. The formula still accounted only for the older environment, builtin,
state-map and arena-binding fields.

The test now includes that existing word and adds a compile-time64-byte Scope
check. NeedsDrop remains72 bytes. No compiler field, resource limit, proof or
production source changes. [Evidence](scope-layout-successor-evidence.json) binds
the actual CI failure, unchanged production tree and strict metadata receipt.
Format, architecture and all-test/all-feature strict Clippy pass in181.18s of
inner commands, with a clean guarded wrapper exit. The width guard is evaluated
by the compiler; private runtime execution remains pending successor CI. No cold
private harness was linked for this assertion repair.

That full CI run passes2741 private checks and fails four. The other three and35
public failures share the remaining ordinary Some-cleanup-row payer issue,
LIB177, which has a separate compiler repair. This assertion update does not
waive those errors or count them as passing. Full/native/debug/independent
qualification of the complete successor remains required.
