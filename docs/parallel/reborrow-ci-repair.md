# Reborrow CI compile repair

Full42285466362 on main3ce6b5d05 stopped during lint/compilation. The PA10 test
scalar executor's exhaustive match omitted the new Reborrow operation, and the
private unsupported-role control exceeded Clippy's100-line function limit.
No compiler execution failure was reached by that run; preserve its raw failed
log separately from the preceding passing source/native qualification.

The scalar helper now explicitly rejects Reborrow alongside other aggregate
operations outside its scope. The private test keeps the same72 independently
reviewed roles in an explicit constant inventory, separate from the compiler's
admission table. Its unsupported-role assertions remain unchanged. These are
test-only changes, with no lint allowance or production compiler edit.

One bounded CI-driven batch passes formatting, architecture audit, strict
all-target/all-feature metadata compilation and the existing10,000-operation
scalar execution/view-determinism control. It does not link or execute the full
private harness. The compiler executable remains byte-identical at9aa812d6;
no fresh original-core observation or broader runtime pass is inferred.
[Machine evidence](reborrow-ci-repair.json) binds the commands, timings, source
hashes and failed-run log. Full successor CI remains required.
