# Shared language declarations and actual profiling bodies

The source checkpoint clears LIB-126 and replaces repeated per-name header work
with one append-only vocabulary and typed declaration profiles. It does not
complete PA31, implement the remaining builtin consumers or build a library.
The [specification packet](../pa31/tests/spec/compiler-language-claims/README.md)
retains those requirements explicitly.

The existing 87 identities and spellings are preserved, including diagnostic-only
Send. The 100 appended names follow the pinned Rust table and weak-only policy.
They are a raw-source inventory, not 100 active blockers. Configured item kind,
resolved owner, foreign status, method body presence and own/parent written
formal counts supply target/arity validation. None means unrestricted generics.
A claimed struct must be a real struct rather than a union sharing its resolved
kind. New variant claims participate in the existing indexed-owner walk.

Actual compiler_move/compiler_copy definitions retain ordinary source bodies.
Their pinned declaration restriction is a free nonforeign function with exactly
two written formals; no invented signature or no-op body is substituted. Both
independent Rust and the actual RustGM lowerer/locked CPPGM backend execute the
C observer and print `9 11`. Current codegen adds no profiling annotations.
Unimplemented builtin semantics stop before ordinary trait/layout/drop checking
could assume their absence, and old metadata profiles reject unencoded roles.
In particular UnsafeCell, AsyncDrop, pinning, reflection and weak memory hooks
have not acquired fabricated implementations. General source library publication
still rejects its existing source-stability metadata boundary.

Qualification of the final compiler/test source:

- 17 exact private regressions, full formatting, architecture and strict Clippy:
  obj/qualification/fast/1793747887328957709/receipt.json, 125.45 seconds,
  SHA-256 `9927486101f369a4dc936e1b2c3ebb773d79e9ddb5f771ff6396f978e42e122b`.
  These include all 135 declaration cases, all 98 unimplemented semantic fences,
  profiling lowering/transport and existing cold-role/ordinary-record controls.
- 19 production smoke tests across two targets, including CLI and production
  Clippy: obj/qualification/fast/1793748044038510438/receipt.json, 220.47 seconds,
  SHA-256 `6eac8ff237116aa1c0ed281a280adbdeb9c3730fab56bfa1bd49c12aa526237b`.
- Independent pinned Rust: 135 source outcomes and native `9 11`, 3.47 seconds;
  oracle-v5/receipt.json SHA-256
  `f57b6497513cb8530077ec9f5a9941fde1fe59eb7f985f47bda6aba67238ffa1`.
  Contract-v5 SHA-256 is
  `9303d4a2f0aa30c2580afd111d799324626a294fabcc48ae8a95c842a74fb471`.
- Joined qualification, 45.16 seconds: 135 CLI names outcomes, 104 semantic
  admission/fence observations, old-candidate rejection, the retained CLI
  library-publication failure, actual ordinary-body native output, upstream and
  harness checks, and unchanged original core. Its receipt is
  obj/parallel/compiler-language-claims-v1/qualification-v1/receipt.json,
  SHA-256 `e9cfafdb320273c9413895e935677b76100284650599484c6ec8d225fe1c4cd9`.

The immutable candidate is
obj/parallel/compiler-language-claims-v1/cli/50b4819716268d804b879f05f86506d844fdcb72e42c9d46d86aa19cb1198c7d/rustgm.

Actual measured cold sizes are Catalog3376, Reservations4744, Provider2992,
Imported3064 and completion-set24 bytes (three u64 words for187 roles). Ordinary
Body456, syntax Node36, NodeFacts80 and one-byte LangItem remain unchanged.
The final existing guards check these measurements. There is no new persistent
permission map, source reparse, speculative lookup or ordinary-record field.
No wall-time speedup is claimed. Earlier independent setup/prediction failures
and the local timeout/variant-owner failures remain retained; the final source
passes both coherent gates. The broad global type change needed a bounded
build/test recovery; limits were not raised. Subsequent focused validation was
125.45 seconds. The previous d12d1d133 full CI41731751383 passes every gate;
this checkpoint's CI is tracked by its own commit.

The original-core observation clears profiling.rs and reaches LIB-127:
`old_name = "try_trait"` on the unstable Try/FromResidual reexport in
library/core/src/ops/mod.rs197. Error: file83 byte7164, renamed library features
are not implemented. It takes18.6653 seconds and375 source opens, with all722
input hashes, six configuration hashes, target/options/capAllow and finite limits
unchanged. Opens do not count compiled files. The result SHA-256 is
`67b8022da79fca6d766af63bb8e566bd8b82f2129972f5ed3738c52c23a5e5aa`.
The current source work remains declaration admission; every required builtin,
body, authenticated imported contract and original core/alloc/std artifact must
still be completed. Next implement actual renamed-feature identity/gating and
transport rules rather than ignoring old_name.
