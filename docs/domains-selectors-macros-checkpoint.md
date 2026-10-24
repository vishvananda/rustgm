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
qualified 63 native contexts on the same production changes; the combined
release/native qualification below also passes. See
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

## Release qualification and original-core observation

Frozen `1df52349` has the same compiler/test sources as `91113764`. Its
`obj/parallel/domains-selectors-macros-release-v1` run passes all 60 functional
stages: 3,605 release/native tests, all 8,382 active text cases, affected
independent source/native suites, seven transitive metadata consumers and both
measurement adapters. All 283 declared exact worker/integration test names were
observed passing. Recent format/audit/strict/harness/upstream receipts were reused
only after verifying their unchanged source bytes; the backend check ran fresh.

The functional receipt is
`65bc10401d3adbcaa71afac312e77d0fb9c8eacb71b1a1798043fe2b4cd01279`;
checks are `3b2986df1b87955e57091ac824ed6a502d8dfee86861d6e6590915774dc4638a`.
The immutable CLI is
`9793c30f2d29a99196687032f88960c317c28c47a3f4f6973ba681bb09698ac4`.
Cleanup receipt `1792824375809440414` removed 15,231 generated files and
17.20 GiB of unshared blocks, preserving current pins and losslessly compressing
one large diagnostic.

The unchanged original-core probe now parses past the previous scoped-macro
syntax error. It stops at the `core_panic` builtin declaration in
`core/src/macros/mod.rs:3`, with four opened original files verified against the
pinned inventory. See [LIB-013 and its source evidence](library-build-blockers.md).
No new assignment is activated, and upstream core, alloc and std builds remain
unavailable.

## Performance results and retained costs

`obj/parallel/domains-selectors-macros-perf-v3` compares the immutable published
`fc9f0313` compiler with this candidate: 83 inputs, 80 exact-output comparisons,
59 successful native programs and 46 passing growth controls. Three newly
supported reference-selector inputs have no baseline speedup comparison. Every
input and binary remains unchanged. The results and analysis SHA-256s are
`c5caa22af3e943be168d6d8a5bfe89f68e4fb4c0eb9866aff7c9b4afd6bdf99f` and
`83259195e6f79122ff900f7b0ea4d7615034377680cd74239fe55a1fa5767cde`.

This is **not a zero-regression performance report**. Eighteen attribution
thresholds fire, including four instruction increases in legacy macro ancestry
inputs: depth/width 8/8, 8/32, 32/8 and 32/32 cost 6.31%, 9.35%, 7.77% and
9.66% more instructions. Actual per-token context recovery and mark construction
add work alongside the shared ancestry improvement. The depth-128/width-32
control costs 0.40% more instructions; the 512-declaration historical-shadow
control uses 5.34% fewer. Retain the increased costs as an explicit limitation
of this correctness/representation checkpoint. The next macro work should reuse
prepared context transformations while preserving every token's actual parent,
and measure the same shallow and deep controls. Wall-time observations are
retained with A/A calibration; they are not used to dismiss instruction costs.

The fixed follow-up in `domains-selectors-macros-memory-v2` profiles the two
largest RSS deltas plus ordinary-scalar and introduced-context-width-512 controls.
Peak live heap is 46,362→45,962 bytes for the macro alias case,
1,574,920→1,574,843 for ordinary512, and 28,821→28,740 for ordinary-scalar.
The changed-path width-512 case grows from 2,631,994 to 2,705,874 bytes (2.81%).
Final process mappings show executable-page residency and retained allocator
pages contribute to RSS differences; the alias case's RSS direction reverses
in this attribution observation. This does not erase the original RSS samples
or prove that every unprofiled case has identical memory behavior. The eight
heap/mapping observations are recorded under SHA-256
`10e0b0c86dbf60af82de09409bb8655c643e515a442f32e8a1adc6d63922fa1e`.

Earlier failures remain intact: performance v2 omitted required crate identity
arguments on 25 library rows and stopped before timing/native execution; v3
corrected only those arguments. Memory v1's GDB redirection discarded the
inferior arguments, so its process observation is invalid; v2 supplies the exact
frozen arguments. Neither correction changes compiler sources, inputs, native
expectations or performance thresholds. Performance cleanup receipt
`1792825673417211162` and memory receipt `1792826066766309194` complete normally.
