# Complete source Drop and real pinned cleanup

[Evidence](full-drop-source-evidence.json) binds the complete original source,
fixtures, compiler, frozen predictions and bounded qualification receipts. Both
previous Rust-positive unused Drop declaration gaps now pass. Executable profile
checks occur only when actual exhaustive destructor registration needs them.
The complete source matrix passes12 outcomes; five existing generic-selection
controls and four declaration-policy controls also pass, with formatting,
architecture and strict Clippy. Every independent outcome matches pinned Rust.

The actual provided `Drop::drop` body can call pin_drop because its actual source
parent is Drop. An impl or subtrait default method has no such exemption; exact
body identity prevents propagation to nested bodies. A pin-only override keeps
its real Pin receiver and independent late lifetime. Its runtime recipe names
the actual provided Drop::drop declaration and canonical signature. Concrete
Self substitution uses the existing charged fold and instance interning, and
the original body still passes normal dispatch, materialization and loan proof.
Exact selected-callable validation is preserved rather than relaxed for ABI
similarity. The cold recipe registry retains that actual callable signature and
uses its cached parameter summary; no dense Body/Kind or recipe record grows.

A persistent ignored native test executes four real source programs through the
locked CPPGM object backend. Ordinary/pinned traces match `23 7` plus LF; generic
ordinary/pinned traces match `17 7` and `19 7` plus LF. These distinguish actual
provided-body forwarding and override dispatch from accepting declarations only.
Initial failed profiles, native setup, candidate gates and diagnostic receipts
remain retained. No generated reference was hand-edited.

Immutable CLI9ebf8ae5 preserves original core at LIB-171 AsyncIterator,
async_iter/async_iter.rs17/file108:636, in20.8564s/375opens. All722 input hashes,
six pins, fifteen configuration values, options/capAllow and finite limits remain
unchanged. Status is still1; no original core/alloc/std artifact is produced.
The measurement is a concurrent observation, not an A/B throughput comparison.
Current/predecessor pins are9ebf8ae5/15ac6c1c. Superseded8d4a4ff3 is unpinned and
pruned; its historical observations and receipts remain.

Exclusive immutable A/A-calibrated ABBA retains all72 CPU/wall observations and
complete output equality. Common generic Drop scales8/32/128 have candidate
median times.0233/.0530/.1808s and paired wall ratios1.045/1.006/1.022. The
ordinary4096 control is.9480s/ratio.965. A/A and individual blocks are noisy,
including Drop128 B/A1.169 with A/A.938. These observations support neither a
broad speedup nor a blanket regression-absence claim. Canonical method lookup
remains bounded by the actual two-member source contract; concrete substitutions
and retained instance growth charge their real payer.

Full42151987920 onbfd50b429 passes release/course, native and debug qualification,
then fails independent Rust checking because raw upstream source modules were
incorrectly treated as standalone programs. The checker now classifies source
fragments explicitly and compiles checksum-bound complete provider companions,
requiring their original bytes to remain embedded. Unknown oracle kinds fail;
same-basename diagnostics no longer overwrite one another. The successor passes
all18 whole-program upstream outcomes and both checker guards. Full282 harness
and upstream checksum checks pass in the preceding qualification. A complete
manual CI successor remains pending; later assignment-release requirements are
not represented by these focused checks.

Broader pin_v2/projection/feature closure, async destruction, CTFE and genuine
const value history, unwind and authenticated destructor/body transport remain
required. AsyncIterator source forwarders remain the original frontier. The
isolated const consumer still has a new hard negative mismatch and cannot be
integrated. This finite checkpoint does not release a whole PA after24 or
complete the original standard-library goal.
