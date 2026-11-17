# Fixed TLS target fact and authenticated query permission

`outcomes.json` owns all source/data inputs and predeclares fifteen source
outcomes plus two driver-only override rejections. Pinned Rust 1.98.0 reports
`target_thread_local` for the fixed x86-64 GNU abort target, both with ordinary
`--print cfg` and with `-Zunstable-options`. This target fact is independent of
permission to query it or to create thread-local storage.

The dedicated `cfg_target_thread_local` feature or an actual literal macro's
allowance admits demanded query leaves. Captured caller expressions do not
borrow an allowance. False and short-circuit predicates retain their demanded
uses; a disabled unvisited subtree does not. The existing provenance machinery
owns every query check. The `thread_local` storage feature and its allowances
remain separate and cannot pay for a query, and query permission cannot create
thread-local storage. Driver `target_*` overrides stay forbidden.

The private controls retain the two-word feature set (storage bit 80, query bit
81) and one-byte known-consumer mask. The query consumes its final bit 7, with
eight canonical consumers; every previous mask bit remains fixed. Existing
width/call scaling controls also exercise the new gate. A ninth consumer needs
an explicit representation review.

These source-mode controls and target observations do not qualify native TLS,
an original std artifact, metadata transport or an assignment release. Storage
execution retains its separately qualified existing tests.

Forbidden target overrides fail during argument parsing with the established
driver usage status 2. The semantic source outcomes retain their declared
statuses.
