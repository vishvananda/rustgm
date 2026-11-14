# Generic conditional member calls

The original core library build at clean candidate28 reached
`mem/drop_guard.rs:90`, `DropGuard::dismiss`, and rejected its generic
`ManuallyDrop` field calls with `const function member lacks its current
conditional trait premise`. Complete source analysis had already succeeded.
A const impl for a nominal family can prove such a call without a written
premise naming that whole family in the caller's environment.

The defining-body validator now authenticates completed normalized generic
applications through its existing source-request path. It checks the actual
body/caller, typed source operation, complete target and opened signature,
selected method or UFCS fact, retained access step and source effect. Generic
projected and unsafe requests retain their existing abstract-premise path.
This is defining legality only: source-use handles remain locations, and actual
implementation dispatch, const entry, argument/lifetime and ownership proofs
remain independently required before execution. No metadata format, dense AST,
body/value field, new proof cache, backend or storage policy changes.

The ordinary custom-trait minima belong to
[PA25](../pa25/tests/spec/conditional-member-calls/manifest.json); actual
Deref/DerefMut language roles belong to
[PA31](../pa31/tests/spec/conditional-member-calls/manifest.json). The packets
freeze19 prepared outcomes:13 accept and6 reject, including8/32/128 scaling.
Pinned Rust1.98 independently agrees on ordinary and standard-core semantic
companions. The original ManuallyDrop/MaybeDangling declarations and method
bodies are reduced separately; complete original upstream files are retained
byte-exact with attribution. The native source/independent pair observes48
through dependent mutable, shared and direct generic calls. Four prior native
const-coercion controls also pass.

Final evidence lives in `obj/parallel/conditional-members/`:

- fast3 qualifies four frontend controls; fast4 (1794670401867519614,54.94s)
  qualifies the complete new source packet and the existing18-case const-premise
  packet. Seven previously recorded accepting lowering gaps now succeed; their
  original independent acceptance expectations are unchanged.
- fast5 (1794670571182299902,43.49s) qualifies scaling after prepared lowering,
  including defining eligibility work. Inference/canonical totals at8/32/128 are
  7314/3916,29034/13782,115914/53222. Strict selected/production/CLI Clippy,
  complete formatting and architecture checks pass. No timing/speed claim.
- native1794670477428325259 qualifies two exact groups/five actual programs in
  37.63s. `reference-native2/` retains the independent48 observation.
- replay authenticates all3964 compiler source hashes/37 packet inputs including
  manifests, byte equality of three complete originals and all three new regular
  cases.315 harness tests/495 roots/13 binaries/44 stages/one component and
 128 upstream+126 Reference checks pass. The new private forgery regression is
  retained for full CI; no cold local private harness was built.

The first broad candidate wrongly routed projected abstract calls into the
normalized path; fast1 caught that regression, and the old premise path remains.
Fast2 exposed seven newly passing expectations, updated to require success.
The original-fields Rust companion initially omitted const_trait_impl (E0658);
its feature setup was corrected with the accepting contract unchanged. A native
source draft used addition without supplying Add; the final pair uses dependent
references, retaining all three observed calls and stdout48. First diagnostics
remain in the raw record. The initial independent object-only C link lacked
Rust's real panic implementation; the independent linker harness uses the pinned
Rust standard runtime. No shim or outcome inversion was introduced.

The three genuine library commands for shared-field, mutable-field and
direct-method now clear the conditional-premise error and reach
`source declaration kind is outside the initial source profile`. Each remains a
required accepting metadata/LowIR contract; no metadata exists and LowIR is empty.
Prepared/native success does not qualify those surfaces. A clean release29
original-core library retry is next. Original v13/v14 artifacts, native original
consumers, all future state/poll/drop requirements and every earlier named
handoff requirement remain open. PA25–PA44 remain planned as whole assignments;
the113-case PA25 const-machine component is unchanged.
