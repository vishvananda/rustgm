# Source structural constant patterns

The source pattern checker now consumes actual `PartialEq` and
`StructuralPartialEq` identities instead of the bootstrap derive-property table.
This qualifies19 source outcomes (12 accepting, seven rejecting), with three
actual Rust/C exports. It reduces candidate13's Alignment failure; an unchanged
successor original-core observation is still due. No original library artifact
or whole-PA completion is claimed.

## Contract

The complete pattern type needs ordinary `PartialEq`. Each nominal value visited
by expansion needs the actual structural marker. Only fields of the selected
constant variant are expanded: a different variant's nonstructural field cannot
reject a valid constant. Manual equality alone and an unrelated same-spelling
marker do not qualify. Under its internal feature, Rust permits a manual impl of
the real marker; matching still compares fields and never calls that equality
body. Const contexts also do not require const equality dispatch.

This eligibility query deliberately operates modulo regions. It grants no
trait/call evidence. Its private witnesses are retired as gaps in the existing
streamed allocator, while constant selection, initializer permission and ordinary
pattern borrows keep their separate proofs. Actual type conditions remain
mandatory. This follows the pinned [Rust implementation](https://github.com/rust-lang/rust/blob/88d9e12ae/compiler/rustc_mir_build/src/thir/pattern/const_to_pat.rs#L575).
The initial prediction that an unpaid equality-impl lifetime condition would
reject was disproved by Rust. That misprediction remains in the packet manifest;
the corrected positive is required and passes.

The existing cold role ledger validates the real empty structural marker even
without ConstParamTy. No dense syntax/type/body record is enlarged. Bootstrap
pattern behavior keeps its existing derive table. Authenticated imported
structural-pattern transport remains required.

Consuming a local derive list also retains its actual library API policy. The
completed inventory supplies the feature; the selected name token supplies the
allowance context. A literal attribute can use a Bang definition's allowance,
while a captured meta attribute keeps its caller context. A paid temporary index
visits each affected owner's attributes once; ordinary requested-feature uses
need no index. Capacity growth is paid before allocation.

## Evidence and boundaries

All input bytes and initial predictions were frozen before independent Rust
execution. Missing dependencies, old const-impl syntax and staged stability
setup failures remain raw; they are not language rejections. PA31's
[packet](../pa31/tests/spec/structural-constant-patterns/README.md) binds19 exact
sources and its C observer. PA20's ordinary controls retain all four sources in a
[serialized manifest](../pa20/tests/spec/structural-pattern-controls/manifest.json),
following the existing pending-input convention. Ordinary handwritten standard
trait implementation/unused-variant qualification remains required.

- 1794594933591102663: seven exact tests across frontend, constants and patterns,
  including all19 source prepared outcomes and existing closure/privacy controls;
  formatting, architecture and selected production/CLI strict Clippy pass.
- 1794595182597516639: final source/scaling assertions, including actual trait
  selection counts; hash-checked replay keeps the raw counters.
- 1794595309828174033: the final actual native program prints
  `1 0 1 0 1 0 0`, matching the independent Rust/C program. Its handwritten
  equality always returns true, so the unequal field result checks real matching.
- Upstream verifies128 fixtures and126 Reference files. All315 harness tests,
  487 roots/13 binaries and44-stage/component layout checks pass. No reference
  is generated or changed.

At8/32/128 constants, inference is5024/6752/13664 (72n+4448), canonical work is
4047/8055/24087 (167n+2711), and actual trait queries are16/64/256 (2n).
These are bounded-work observations, not timings.

Exploratory wippattern2-dev-fast binds3948 compiler sources and its dirty patch;
CLI SHA25656485edcef6d8823074fd1a0cd153d0409282bd3bce9077a7e649174d4e0b1a1.
Frozen nonempty paired-derive controls at8/32/128 and an ordinary control agree
with Rust and preserve complete outputs against same-profile wipclosure5.
Calibrated A/A+ABBA observations await the unchanged load gate: the last host
load was42.88/57.73, above14/20. Their raw declarations are under
obj/parallel/structural-constant-patterns/measure-inputs/.

The earlier empty derive-list scaling input is a valid required positive but
hits the baseline's source-vocabulary introduction/edition failure. Its bytes,
hashes and unrun larger widths remain serialized in the packet manifest. It is
not counted as passing or silently dropped. Private/full successor CI, the known
predecessor eight private failures, clean release/core observation and original
library artifacts remain required.
