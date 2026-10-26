# Source Not declaration and ordinary caller proof

Compiler source `d0dbb9875da609ea4ae5b99a20f5716d1ac7b318` passes six
focused controls in hosted run `40267233322`, after merging public
`52bb6ddabed14f5d8b900df1b0bc59c51b2a8a60`. Actual source Not calls now use
their real trait, method, Self, Output and caller requirements. Primitive
execution is erased to the existing unary operation only after those ordinary
requirements have been checked. Overloaded execution retains its actual method
call and checked original body through LowIR.

This is a local source consumer checkpoint. Candidate native execution, source
Assert expansion, panic execution, Abort execution and foreign Not role transport
remain required. None is established by the source language-item claim.

## Contract and primary source

The pinned compiler is Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. Its type checker first checks the
actual Not language declaration and implementation even for a primitive operand:
`rustc_hir_typeck/src/expr.rs:435`, `op.rs:969` and `method/mod.rs:346`.
`callee.rs:1038` applies the real const-trait use effect. Scalar writeback at
`writeback.rs:145` removes the resolved method after checking; THIR's expression
builder then emits the primitive operation. This avoids recursion in the actual
primitive Not implementation, whose body itself uses `!self`.

The current source adapter borrows the actual completed source header and its
attribute ledger. Its bounded profile requires a safe nongeneric trait with its
real required Output type and safe synchronous Rust `not(self) -> Self::Output`
method. It checks actual active member nodes, trait ownership, canonical Self,
projection application, callable signature and current method syntax. A warm
query still validates the actual owner, source incarnation, ABI, flags and
parameter-list identity. Missing or fake declarations supply no permission.

The body checker then uses the ordinary source method instantiator and trait
obligations, including inherited clauses, const effect, type, lifetime and caller
requirements. Its scalar erasure does not install a bootstrap operation or
factory. No new persistent field, result cache, Rvalue, metadata role permission
or backend operation was added. Ordinary crates keep their prior path.

## Independent observations

The new no-core reductions are distinct from the sixteen published standard
library Assert observations. Their exact sources, failed hypotheses and
prerequisite corrections remain under
`obj/parallel/pa31/source-not-no-core-v1` through `v4`.

- Run `40261347742`: eighteen metadata/object observations. Missing Not,
  same-name ordinary trait, missing primitive implementation and wrong Output
  negatives are confirmed. Five other reducers stopped at genuine prerequisites.
- Run `40262303834`: ten corrected observations stopped at the real
  LegacyReceiver prerequisite; those results remain failed predictions.
- Run `40262767393`: ten observations using the intact minimal-provider
  foundation establish primitive and overloaded bodies, and reject a const-trait
  call using an ordinary implementation. Two object adapters still required real
  drop glue; their successful metadata results are recorded separately.
- Run `40263689075`: only those two corrected object adapters ran. Both exported
  ANSWER byte `01`, using the pinned actual drop-glue/Destruct declarations.
  An ordinary Not method returning false still yields true for primitive
  `!false`, demonstrating the type-check/execution distinction. These are object
  bit observations, not native program execution.

No prior observation was repeated to qualify this implementation. The const
candidate reducer uses the actual Destruct implementation-restriction attributes;
the earlier omitted-marker failure is retained.

## Candidate evidence

Run `40267233322` freshly passed metadata, formatting, architecture audit, strict
production and private Clippy, one private build, nonignored exact inventory,
six executed controls, harness and upstream contracts. All 2,714 frozen source
fingerprints and the tool hash match before and after. The helper plus mandatory
CLI build stayed within the five-minute feedback budget.

The three new groups cover actual primitive/overloaded body proof and scalar
erasure; missing, fake, malformed, foreign, replaced-image and warm-mutated
headers; and real constant evaluation with ordinary/const implementations.
The const-trait/ordinary-impl rejection reaches the actual producer:
`trait obligation requires an unambiguous matching assumption or implementation`.
Both accepted constant consumers lower to the actual LowIR literal `1`.
The other three groups retain ordinary storage/closed-wire, complete-domain old
bytes and closed Abort behavior.

Actual assertions and output retain `SourceNot` 12 bytes, cold Catalog 704,
Reservations 808, SourceCore 176, Resolution 2056 and Package 256. The descriptor
is transient. Failed attempts retain their existing canonical-work debt, and a
warm valid query incurs fresh work. There is no new ordinary allocation owner.

Evidence is under `obj/parallel/pa31/source-not-private-v3`:

- `plan.json` SHA-256
  `251b0de2fb0c4d814bdb65d63469e8cf229fed1b2ce96df627ff83dabc6c0675`.
- `qualified-evidence/1792987578817686481/receipt.json` SHA-256
  `80d7158d20eafbbfb7b1858b45817bc35190803f1ee683c2fa0d4880b264f781`.
- `verified.json` SHA-256
  `9d246e98dc6635246e5a5424fe03fa25ef44257269b6c79a9bf562fee25cbb01`.
- Fast evidence artifact `12627081187`, ZIP SHA-256
  `be30bc909ebc9dc614138d03bc8c96cfba63d69629c39aa22395b97daeceefbd`.
- Actual pinned CLI SHA-256
  `150b8b9262ced88522705e0e69243c0be6b011954c5f18867334241e26d3bb33`.
- Selected artifact `12626682356` retains the immutable CLI and full binding;
  ZIP SHA-256
  `0e780f57c244a915ce7053003c038dde11ab2bd925f862a608a38d6250566be8`.
  It remains remote; local evidence hydration did not extract an ELF.

The prior filename-audit failure and subsequent four-pass/two-failure run remain
at `source-not-private-v1` and `v2`. The successful checkpoint does not rewrite
their statuses.

## Required ordinary factory follow-up

The existing executable regression
`sema::primitive_methods::tests::primitive_factory_requires_matching_selection_and_preserves_ordinary_template_gate`
remains unchanged and requires successful primitive factory proof. It currently
fails before that proof with `imported must_use contract transport is not
implemented`. The exact same source, containing no `!`, was observed once on
immutable qualified predecessor `93fe092f` CLI `e88d41c1` in run `40266698743`.
All 2,672 source fingerprints, tool, source and binary hashes match. This is an
open PA29/PA31 imported-contract requirement; it is not a Not regression or a
passing ordinary factory observation.

Its source, actual diagnostic, full binary binding and closure selector are
retained at `obj/parallel/pa31/not-factory-predecessor-v1/verified.json`, SHA-256
`f075fef900dd1be6e767ce7d234a55bceea6f9cc01b68b3b65fafafe015b710f`.
No ordinary imported-contract permission was broadened for this checkpoint.
