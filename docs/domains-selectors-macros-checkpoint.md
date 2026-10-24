# Source domains, reference selectors and macro identities

This checkpoint combines three compiler changes on top of published main
`fe0eeffe`: source metadata domain validation, shared-reference constant selector
callbacks, and retained macro identifier contexts with indexed invocation
ancestry. The integrated compiler snapshot is `91113764`; subsequent specification
and checkpoint documentation do not change its compiler or test sources.

The source-domain index validates actual header ownership, canonical late-region
slots, binder ancestry, type-recipe domains and operation/input coordinates.
It is borrowed data support, with no descriptor, codec or imported-proof
permission. Signature equality, operation typing, complete obligations and fresh
client proof remain required before source-body transport can be enabled.
See [the domain checkpoint](pa29-source-domain-index-checkpoint.md).

Reference selector calls retain the proved caller's whole input/output relation
under the actual Code and input row. Code and row publish together after storage
checks. Named initializer demand preserves its declaration owner before proof;
callback entry retains both public and owned signatures. Existing defining
cleanup and const-member entry checks remain intact. The worker independently
qualified 63 native contexts on the same production changes; integrated release
and native qualification are still required. See
[the reference selector record](pa31-const-select-reference-progress.md).

Macro item and import identifiers retain their own contexts rather than adopting
the surrounding item's context. Root-only bindings keep their lazy indexes
unallocated. Textual and resolved lookup share one actual invocation ancestry
index, with constant-size records and logarithmic queries instead of repeated
parent walks. These structural changes do not enable scoped declarative macros:
contextual path selection, definition-site visibility, glob synchronization and
member lookup remain unfinished.

## Combined development qualification

The frozen `91113764` run completed successfully in
`obj/parallel/domains-selectors-macros-v2` in the validation worktree:

- Formatting, architecture/file audit (2,252 files, zero findings), strict
  all-target/all-feature Clippy, harness and upstream checksum checks passed.
- One focused debug build passed 127 private and 151 frontend tests, including
  all three corrected fixtures and the existing shared publication/proof controls.
- All 8,382 active PA1–PA24 text cases passed.
- Cleanup completed with receipt `1792821622618704486`; no generated files
  needed removal. The resource lease ended successfully.

The checks receipt SHA-256 is
`342c76aa720ea2b9aa71869cef9fba8c1c481306ec4860589f96a588d014c1e7`;
the focused receipt is
`a2e7e3d68a3535b27f7b51f334bb4ca804bf28fe246c56da4ae72351821d13ce`.
Earlier failed and superseded preparations remain preserved separately.

The complete-provider domain index retains 24,081 bytes with a 24,403-byte peak
envelope and 2,615 charged steps. The depth/query grid still adds 19 steps per
depth increment and six per query, without their product. The newly added
canonical-slot opponent rejects after 623 charged steps. Owner, OwnerKind and
Value remain 32, 20 and 24 bytes. Reference execution keeps Code, Frame, State
and Machine at 304, 248, 448 and 4,728 bytes; its cold Row is 96 bytes.
These are structural measurements, not a compiler throughput claim.

The corrected anonymous-reference observer now identifies the actual Cast root
and dereference Read: the read has canonical type `u8`, queries the source Copy
role, and receives `false` despite the fixture's actual Copy implementation.
The early implementation-readiness cause remains under investigation. The
separate anonymous scalar owner boundary is also unfinished; neither diagnostic
control qualifies the original Rust-positive program as supported.

Full release/native qualification, affected independent source/native checks
and immutable current-baseline performance comparisons remain pending. No new
assignment is activated, and full upstream core, alloc and std builds remain
unavailable.
