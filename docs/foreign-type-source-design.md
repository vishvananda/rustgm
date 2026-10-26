# Genuine foreign type identity

LIB-032 is the unchanged `unsafe extern "C" { type VTable; }` declaration at
`library/core/src/ptr/metadata.rs:170`. It is an opaque external type, with no
value layout. It must never become an empty aggregate or a type-alias fallback.
The current producer and metadata consumer checkpoints remain separate evidence;
this successor has no candidate qualification yet.

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

Source implementation is still unqualified. It appends variants, preserving old
discriminants and wire vocabulary. Six selected private groups are authored for
actual identity/owners, unit metadata versus sizing/body proof, active/literal
gates and invalid forms, actual source Freeze impls, unavailable value layout
and transport, and original replay plus 8/32/128 sparse storage/hot layouts.
Unknown numbers remain unknown until a real candidate run.

B's Pattern source `c5fa74bc` supplies the shared AST/type/inference baseline.
It was normally merged on this successor; its separate qualification is pending.
Foreign work preserves `Pattern { base, constraint }`, shared unary canonical
reconstruction, feature slots 22/23 and their runtime validity. The qualified
Pointee consumer reference remains `pa31-pointee-metadata-consumers-v1` at
`7e3dc6b8`. Root's real Thin alias consumer is queued for the combined join.
