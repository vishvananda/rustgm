# Genuine foreign type identity

LIB-032 is the unchanged `unsafe extern "C" { type VTable; }` declaration at
`library/core/src/ptr/metadata.rs:170`. It is an opaque external type, with no
value layout. It must never become an empty aggregate or a type-alias fallback.
The source successor has merged private and CLI qualification and advances
unchanged core loading beyond VTable, as recorded below. Complete Thin semantics,
broader Foreign operations and transported declarations remain required.

The approved representation appends `ItemKind::ForeignType`,
`DefKind::ForeignType` and canonical `types::Kind::Foreign(ItemId)`. The identity
is the actual resolver declaration and source item. Normal names, import/privacy,
duplicate checks, substitution/equality and captured/original source identity
remain authoritative. No type-level ABI, fabricated fields, new LangItem or
metadata wire permission is added. The existing source exporter rejects this
unsupported declaration kind; type codecs also must reject unsupported transport.

The parser retains the actual `type` token in the existing sparse gate-sites
owner. Feature `ExternTypes = 24` consumes the actual checked crate request or
that exact token's authenticated macro allowance. The generic gate-site source,
context and transfer journal are reused. Defaultness and alias accessors remain
filtered to their own kinds. Inactive Foreign syntax has no pre-expansion warning
and creates no semantic declaration; this differs from observed defaultness.

Foreign is well formed and PointeeSized, but neither Sized nor MetaSized. Its
pointer metadata is unit. Those facts are independent: neither the unit result
nor a pointer's thin storage grants a value size/alignment. Struct/tuple tails
preserve the actual terminal type. The existing aggregate sizing-tail walk will
use the same `abi::metadata::declared_step` classifier and existing proof-work
accounting, replacing the old representation-only query. Missing Fields stays
a genuine dependency. Unknown parameter/projection evidence stays scoped and
closed. There is no second cache, graph or proof engine.

Ordinary value layout, by-value ABI/constant bytes and unimplemented transport
fail explicitly. Genuine pointer/reference storage can use thin metadata without
demanding the pointee's value layout. Foreign contents cannot receive automatic
structural trait evidence from an invented empty field list. Pinned
`candidate_assembly.rs` explicitly excludes Foreign from automatic structural
trait candidates and from Sized/MetaSized candidates.

The 17 authored no_core inputs were frozen at `90a20aa3` before 34 pinned-rustc
metadata observations. Both editions were observed once under the unchanged
12 GiB cap, with no Cargo/native execution. The run took 0.878 seconds and had
zero infrastructure failures. All source/status/command/raw JSON hashes are bound
by `ea03b271d4d766c90db1cc967830ac51b3f8a927bc11fa44be9d36299d18d570`
in the worktree's `obj/parallel/pa31/foreign-types-v1/oracles-v1/observed-local-v1`.

Four original prediction differences remain: generic syntax rejection has no
E0044 code, and inactive missing-feature syntax produces no warning, in both
editions. Direct/tail metadata positives, PointeeSized, actual direct/tail
MetaSized/Sized negatives, invalid by-value use, identities/privacy/duplicates,
forms and literal-versus-forwarded allowance matched the predictions. These are
metadata observations, without runtime or full core acceptance claims.

Two additional source Freeze inputs then produced four observations. Direct
Foreign and raw pointers without the genuine source impl both reject E0277;
the pointer-positive hypothesis was wrong in both editions. This does not
justify changing the structural pointer rule from an optimization-property
shortcut. Pinned core_marker.rs910–918 provides actual explicit source Freeze
implementations for raw pointers/references and PhantomData. A separately frozen
input with that genuine raw-pointer impl passes both editions. These six new
observations preserve all original predictions and do not repeat the first 34.

The focused source implementation appends variants, preserving old
discriminants and wire vocabulary. Six selected private groups are authored for
actual identity/owners, unit metadata versus sizing/body proof, active/literal
gates and invalid forms, actual source Freeze impls, unavailable value layout
and transport, and original replay plus 8/32/128 sparse storage/hot layouts.
The actual candidate measurements and their limits are recorded below.

B's qualified Pattern source `44ea6bff6` and documented head `9b7c7934` supply
the shared AST/type/inference baseline. They were normally merged on this
successor, followed by public main `339bdffb2`. Their prior qualification is
retained; Foreign's focused qualification is recorded below.
Foreign work preserves `Pattern { base, constraint }`, shared unary canonical
reconstruction, feature slots 22/23 and their runtime validity. The qualified
Pointee consumer reference remains `pa31-pointee-metadata-consumers-v1` at
`7e3dc6b8`. Root's real Thin alias consumer is queued for the combined join.

## Focused qualification

Candidate `27b1712ddc2ed6163dafb02561aefb79ed0ba76d` has six focused Foreign
controls plus the owning Pointee wrong-owner regression qualified. Five Foreign
groups passed in `obj/qualification/fast/1793041703161947534`; the remaining
ordinary-mode group and Pointee regression passed in
`obj/qualification/fast/1793042422991700261`. The last production change adds
ExternTypes to the existing ordinary-only request-retention predicate. All five
retained groups use Mode::Source/no_core, which does not execute that branch;
their semantic test bytes and all other production files are unchanged.

The first actual execution found that ordinary loading discarded the new feature
request; it was fixed in the existing LocalMacros owner rather than bypassed in
tests. No extra owner/field was added. Ordinary crates without such requests keep
the previous no-owner path. Both successful static/build receipts and every
prior failed audit/compile/lint/test receipt remain retained.

The final gate passed metadata, formatting, global architecture, private strict
Clippy, build and both exact controls in 112.86 seconds under the unchanged
12 GiB absolute cap. Receipt SHA-256:
`38a9b8bae8b3128bca94c5c918ee73c0c7ecf30268f4afb1c061925ad74057ea`.
The private binary SHA-256 is
`6b1b638f19750af3635ab68380de73b40003b40b94a39ea8ab6cecb7e205123f`.
This is not production/CLI/native, whole-core or assignment activation evidence.

Actual bytes: ItemKind 28, Item 64, Type Kind 24, AST Kind 28, Node 36,
inference Constructor 16, Store 472, Unit 688, Package 256, Report 3040,
resolver Definition 36 and Scope 32. Sparse gate row/owner remain 24/56.
No new fields are added to those enclosing owners.

| Foreign declarations | Gate rows/capacity | Cumulative steps | Prepaid bytes | Canonical types |
| --- | --- | --- | --- | --- |
| 8 | 8/8 | 42 | 824 | 29 |
| 32 | 32/32 | 162 | 3128 | 53 |
| 128 | 128/128 | 642 | 12344 | 149 |

Prepaid bytes are logical cumulative budget charges, not allocator observations.
The actual row-vector capacity payloads are 192/768/3072 bytes; the cold owner
is 56 bytes. Allocator bookkeeping and total allocation counts were not measured.
Two original replays validate their genuine source incarnation and type-keyword
site; Foreign rows never become defaultness. Ordinary syntax has no gate owner.

The existing active PA5 `tests/spec/reject/external-type.rs` is now a grammar
positive. Its exact source bytes must remain unchanged while the owning case is
reclassified and its text sidecars officially regenerated. Its independent
whole-program Rust rejection without the feature is distinct from PA5 syntax.
The following integration completes that reference change and CLI/core
observation. Actual Thin consumers and broader Foreign transport remain required.

## Integration and unchanged-core loading

The integrator merged Foreign/Pattern and the qualified selected-header component
with canonical alias declaration checking. Both strict lint modes passed; the
first combined build timed out while compiling the private harness. Its completed
CLI and frontend target were then reused by the standard runner: three actual
frontend/LowIR controls plus production strict checks passed in 26.18 seconds.
Merged private execution subsequently passes sixteen exact controls with strict
checks and all 230 PA5 text cases in 27.20 seconds using the completed build. An
initial selector typo referred to a filename instead of the actual storage_relation
module; the runner rejected it before test execution. That failed receipt is
retained, and corrected selectors came from the actual executable inventory.

A no-build broad ordinary run then found three stale Pattern expectations:
the candidate availability table still closed pattern_type, the unavailable-frame
count remained 39 instead of 38, and the cold Reservations assertion remained 872
instead of the measured 888. Only those expectations and their explanatory
fixture documentation changed. The complete 1,798 ordinary private tests now
freshly pass with private strict checks in 127.06 seconds, receipt
`obj/qualification/fast/1793045013398412869/receipt.json`. The preceding 1,795-pass/
3-failure output is retained. Production sources are unchanged from the qualified
CLI/frontend run. These separate attempts are not one five-minute integration.

The actual NonNull source and emitted LowIR also match the worker's native-qualified
bytes exactly (source 86ab25bc, LowIR be70bf2e); its native `19 23 119 1` observation
is retained, not presented as a fresh native execution. No full CI or native
sweep is implied by the ordinary private result.

CLI SHA-256 `eb644b412597aa77a1d2d8accc539a802e6009e05a940d5ada6e0bfb87d8839a`
passes original VTable loading and reaches `core/src/clone.rs:389`, byte14948,
at `final fn share`. The observer verifies all 722 unchanged inputs, opens 72 files
and takes 1.996 seconds. This advances loading from 65 opened files; it does not
establish complete semantics, an artifact, or a compile percentage. The same
previously qualified target/profile manifest is reused. A preceding attempt with
the older default compiler-source manifest failed setup before source execution;
both records are retained in `obj/parallel/foreign-pattern-integration-v1`.

The official selected PA5 generator replaces only the unchanged external-type
fixture's sidecars after moving it to `tests/spec/syntax-only/`. Its source bytes
still independently reject in pinned rustc for the missing feature; the manifest
now explains that later semantic boundary. The reviewed grammar output has one
ForeignType item under its actual external block, empty stderr and status0.
All 230 stage comparisons pass. The 217 harness tests, all 383 maintainer-suite
registrations, assignment layout/reference provenance and upstream verification
also pass. No whole-core artifact or new assignment release is claimed.
