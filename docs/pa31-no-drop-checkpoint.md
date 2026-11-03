# Source NoDrop and union field checkpoint

The maintainer recognizes the actual `bikeshed_guaranteed_no_drop` language item,
required by the pinned core's union field checks. It requires a trait with zero
written parameters, including lifetime and const parameters. Actual attributes,
members, safety, constness and auto declarations remain source facts. The role
grants neither executable methods nor const implementations. Its real
dyn-incompatibility attribute and implementation restriction retain their owners.

The ordinary positive builtin proves references and the actual ManuallyDrop
identity, recursively requires NoDrop for tuple/array components, and otherwise
requires actual source Copy. Zero-length arrays still require their element's
marker. Normal structs without destructors and lookalike names gain no rule.
Source Copy/Clone tuple markers likewise retain each component's actual proof;
they provide no invented Clone method body.

The existing suspended closure-marker machinery now serves these component
proofs. It retains caller scopes, region debts, normalization, partial children
and canonical evidence on the same iterative stack. No Frame field was added.
Body call validation recognizes only the exact authenticated NoDrop family of
already selected evidence. General custom auto traits remain separate work.

Union field queries exposed a real producer-order error: ManuallyDrop's fields
and policy were installed, but its completion ledger ran after union validation.
Wrapper completion now follows all field templates, sizes and size rechecks,
before field validation. Its existing identity/header/policy checks remain;
constructors and executable bodies retain their later proofs. Completion runs
once, rather than repeating it after nominal validation.

## Shared type facts

Pinned Rust first uses a narrow trivial-Copy shortcut for union fields. When
that fails, it asks the actual NoDrop trait, or actual Copy if NoDrop is absent
in no_core. This shortcut is not a trait proof or value-validity permission.

The shortcut is computed from canonical child facts when a type is interned:
primitive scalars, Never, function items, shared references, raw pointers,
recursive arrays/patterns and recursive tuples of at most three fields. Function
pointers and mutable references do not receive this shortcut. A spare bit in the
existing four-byte Properties stores the result; old flag ordinals remain stable.
Each later field query reads that fact directly. Source membership proofs remain
scoped to their actual union caller, using the existing scratch set.

The first qualified implementation memoized a structural walk inside each union.
That still rescanned a shared deep type across different declarations. The final
implementation removes that map, work vector and boxed scratch. Repeated widths
8/32/128 now charge exactly8/32/128 field steps for both manual and scalar-array
families. Across the nine depth8/32/64 by declaration8/32/64 combinations, steps
equal declaration count independently of depth. A4096-node canonical array chain
and4096 warm queries preserve interner extent and four-byte Properties storage.
These are field-admission bounds, not a claim about every compiler phase.

Old language ordinals0–79 remain fixed; NoDrop is80 and the role count is81.
Measured cold Catalog is1824 bytes and Reservations2520. Provider/imported arrays
are1620/1368 bytes; their existing256-byte difference bound still passes. Ordinary
syntax36, NodeFacts80 and Body456 guards are unchanged.

## Qualification

The [frozen packet](../pa31/tests/spec/source-no-drop/README.md) declares35 source
outcomes and genuine native relay output `8`. All35 match pinned Rust object
checking; the C observer also executes `8` from both independently compiled Rust
and RustGM LowIR through the reviewed CPPGM backend. Full original CFG, move,
region and loan proof covers the ownership opponents. Sixty-six capped/uncapped
CLI constant observations cover33 cases. Borrow escape and repeated owned use
require the full CFG boundary and are qualified by the maintainer pipeline;
constant views are not counted as their ownership proof.

The final combined gate passes19 private and5 production controls plus PA12
125/125 in154.85 seconds, including admission, formatting, architecture and strict
all/pedantic Clippy. The earlier separate gates passed244.91 and177.86 seconds.
The first broad type-summary rebuild hit its bounded build window; its unchanged
warm retry passed without increasing the five-minute limit. A final annotation
review changed no executable bytes. Earlier genuine header/evidence/order and
lint failures remain recorded. Upstream checks and275 harness tests pass.

Final joined-v3 passes31.53 seconds, including native execution, CLI controls,
scaling and a fresh independent check of the shared alias input. Immutable
baseline/candidate binaries produce byte-identical output on the frozen shared
and ordinary inputs. Two matched attempts retain all56 observations: warmups,
four A/A observations and two ABBA blocks per input per attempt. Shared-input
baseline/candidate medians are46.65/40.61ms and47.01/35.73ms. A/A spreads are12.45ms
and1.94ms respectively: the first attempt is noisy; the second observes a local
improvement beyond that calibration spread. Ordinary medians14.09/13.88ms and
15.54/15.74ms lie within their A/A spreads4.59ms and2.22ms. These small synthetic
observations do not establish whole-crate throughput. The deterministic query
bounds and eliminated allocations establish the algorithmic improvement.

Contract-v3 SHA256:
7c23d9c3d52949d71f278e4bfb292d9228c53ba0d65ef60041d1c00c6e565f46.
Independent reference-v4:
176a46fb2ac2e6acf42cb213b604b6d2805c03e79ab0b348054be7e95a2f1708.
Combined1793683257531974532:
469f4f28f14f2acd0220841848c9e603318b5f39125f6ade0c8e99f1d492c363.
Final joined-v3:
77b07c8ea6ae21e2d173ce917f48c6670c2e381b0a6c0ca040d294c6c63bb263.
Final raw timing observations:
d2068a0f7c30bde13bb9243a376c3ee187ec5a6b9e7d99a095ac58db486557af.
Earlier matched observations:
fc07fc1965385a73953ed73fe0224932bf5b21181b0ee63860a91a2804cfa620.
Raw evidence, failed predictions, earlier fixture bytes and failed C++-linkage
observer attempt remain under obj/parallel/source-no-drop-v1. Final CLI is
a716191eacdaeee150445f36f4d81147d78e2cb6098ee24ad82d3f87f7481d26.

## Remaining library work

This qualifies local source behavior needed by LIB-123; it does not complete a PA
or original library artifact. The last original722-source core observation still
stops at the actual MaybeUninit claim in mem/maybe_uninit.rs352. No new original
core observation is inferred from these fixtures. Continue actual MaybeUninit
identity, opaque union storage, initialization/validity, real source methods and
assert_inhabited/transmute execution. General PA29 nominal/role transport and
fresh imported consumers remain immediate artifact requirements. No wrapper-only
metadata profile or replacement standard library is introduced.

The predecessor repair on public1e9b3c514 has full CI41572102514 success across
release/course, native, debug and independent checks. This source checkpoint's
broader release qualification follows integration in CI.
