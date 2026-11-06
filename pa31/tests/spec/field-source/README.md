# Field language headers: independent reference packet

This PA31 packet retains the complete pinned Rust 1.98.0 `core/src/field.rs`
unchanged in `original-field.rs`. `provenance.json` records its revision, license,
checksum and exact declaration intervals. `third_party/sources.json` registers
the original. Companions use a minimal authored no_core dependency prefix and
embed the original struct, unsafe trait, associated declarations and ordinary
Copy/Clone implementations without changing their bytes. They retain the real
`field_offset` intrinsic signature and attributes. Other original bodies are
preserved in the full source, but are not compiled by these companions.

Predictions were frozen before each version's Rust observations. Sources and
predictions from all versions remain intact. `reference-v1.json` binds compiler,
profile, exact source hashes, diagnostics and raw receipts. Every direct run used
the shared resource/storage wrappers, a 90-second outer deadline, explicit
16 GiB output cap, unchanged 150 GiB reserve and two-job policy. All three outer
runs completed successfully. This is reference work only: there is no RustGM
candidate observation, compiler edit or native execution.

## Standard Send identity successor

The sixteen `*-v4.rs` companions differ from their qualified v2/v3 predecessors
only by adding the exact `#[rustc_diagnostic_item = "Send"]` attribute from pinned
`marker.rs:87`. The already vendored full original at
`../coerce-pointee-source/original-marker.rs` matches the pinned bytes;
`provenance-v4.json` records the source interval, license and checksums. No second
copy of the original module is needed. All original Field/Copy/Clone cutout bytes
remain unchanged.

`predictions-v4.json` was frozen before observing the companions. All sixteen
sources pass their predicted metadata and object outcomes, 32 observations, under
the same profile and guards. `reference-v4.json` binds exact sources, commands,
diagnostics and raw receipt. This variant gives the ordinary Send declaration
its genuine standard diagnostic identity; it grants no trait proof by itself.

The earlier qualified v2/v3 sources remain valid Rust programs or intended
ordinary negatives. Their custom auto trait Send is a broader PA20 implementation
requirement, not a setup failure. Their observations are unchanged and remain
separate from the genuinely invalid dependency setup in v1 and the four
original-derived v2 attempts described below. Candidate implementation remains
Root-owned; this successor runs no RustGM candidate or native executable.

## Qualified selection

Sixteen sources have both metadata and object observations, 32 checks total.
Use the twelve clean `*-v2.rs` controls and these four `*-v3.rs` successors:

- `original-headers-v3.rs` accepts the actual declarations, including the genuine
  unused provided OFFSET body and ordinary Copy/Clone bodies.
- `unused-alternative-v2.rs` accepts three ordinary type parameters and ordinary
  data fields on the representing struct, a safe Field trait without the original
  supertraits, a `?Sized` Base, and differently named members with a u8 constant.
- `unrelated-owner-v2.rs` accepts all three associated language members in a
  genuine nongeneric trait different from the empty Field role declaration.
- `inherent-owner-v2.rs` accepts those roles in a genuine nongeneric inherent
  impl with `inherent_associated_types` enabled. Do not reject either owner form
  merely for not belonging to the Field role.
- Five `wrong-*-target-v2.rs` cases reject with precise uncoded attribute-target
  errors. Three `wrong-*-arity-v2.rs` cases reject E0718. Associated arity counts
  the actual parent trait's written generic parameters.
- `invalid-ordinary-body-v2.rs` rejects E0308. Header recognition does not suppress
  the ordinary type error in an unused provided associated constant.
- `explicit-impl-v3.rs` rejects solely E0322 through the actual
  `rustc_deny_explicit_impl` contract.
- `demanded-offset-v3.rs` accepts a genuine builtin Field demand, Type projection
  and default OFFSET CTFE on a repr(C) struct. Object generation succeeds, but
  this packet does not link or execute a native observer.
- `demanded-packed-v3.rs` rejects solely E0277. An existing field in a packed
  struct does not confer the builtin Field implementation.

The JSON supplies the exact selected filenames and causes; earlier versions of
the four original-derived controls must not substitute for their v3 successors.

## Preserved setup failures

All v1 sources claim a nonexistent `send` language item and are unqualified,
including negatives where the intended error also appeared. The original-derived
sources additionally needed the module's Sized import and `fundamental` gate.
V2 corrects these dependencies. Its four original-derived cases still fail
because the prefix omitted real PhantomData Clone/Copy support and the active
`field_projections` const-intrinsic feature. V3 adds those dependencies only and
reruns those four sources. No setup failure or compiler crash counts as a
semantic negative. There were no ICEs in this packet.

## Declaration and demanded-consumer boundaries

The five headers can complete through actual ordinary nominal, canonical trait,
associated-type and constant producers. Independently present members retain
their genuine owner; a completed member alone does not establish a coherent
Field operation family. Alternative unused flags, bounds, member names, constant
types, owners and struct shapes remain legal where ordinary Rust checks allow.

A demanded compiler-generated Field implementation instead needs the authentic
representing type application, valid base/variant/field identity and genuine
trait/member contracts. The pinned solver's implementation eligibility requires
Sized base and field, plus an unpacked struct or tuple. The macro also accepts
enum and union field descriptions; this does not make those descriptions Field
implementations. Base/Type normalization and OFFSET evaluation need their own
actual builtin evidence and monomorphic layout/intrinsic contracts.

Builtin `field_of!` expansion, privacy and invalid-index handling, complete
original Debug/type-reflection and remaining impl bodies, executable reflection,
field projection/layout consumers and authenticated transport remain required.
This finite header packet grants none of those permissions and does not release
PA31 or build an original library artifact. Root owns candidate registration and
the producer implementation.
