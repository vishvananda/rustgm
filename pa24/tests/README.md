# PA24 fixture ownership

PA24 remains planned. The complete reviewed inventory contains 467 source files:
323 accepted and 144 rejected. Its generated manifest and text oracles are staged
for release qualification; they are not installed as an active grading suite.

The accepted native classifications comprise 311 runtime comparisons, two
code-generation-only controls for intentionally unused bodies, and ten entry-free
library object controls. Object generation does not establish runtime behavior.
Negative tests must reach their intended capture, permission, type or lifetime
rule rather than an unrelated unsupported-feature diagnostic.

The inventory covers these families:

- Anonymous identity, contextual inference, generic inputs and nested closures.
- Shared, mutable, unique and moved captures; field precision, prefix merging,
  wildcard/rest patterns, discriminants and shared-reference truncation.
- Environment Copy/Clone/auto-trait obligations, partial moves and destruction.
- Fn/FnMut/FnOnce selection, generic and nested reference receivers, call
  permissions and noncapturing function-pointer conversion.
- Borrowed callable objects, projected and quantified inputs, distinct signature
  slots, captured input conditions and returned references.
- Original and demanded body proof, unused nested closures and library body
  failures before metadata export selection.

The owning [handout](../README.md) defines the required behavior and ABI.
The [release review](../../docs/pa24-release-review.md) maps those requirements to
fixture families and records independent qualification, the reviewed diagnostic
correction and remaining gates. It is the consolidated inventory; an early
32-case capture plan is not the assignment's complete scope.

Existing `*-hypotheses.json`, `*-outcomes.json` and other draft outcome files
retain independently declared expectations and source hashes for their respective
implementation checkpoints. Their historical unsupported observations are not
current language expectations. In particular, the input-only lifetime bridge
now accepts `formation-truncation/reference-result.rs`, while
`output-interfaces/escape-outer.rs` must fail its ordinary lifetime proof.

Required regressions belong here when PA24 is the earliest assignment owning all
their features. Preserve upstream bytes and attribution. At release, every input
must appear in `manifest.json` with its explicit source/native classification;
generated `.ref`, `.ref.stderr`, `.ref.exit_status` and `reference.json` files ship
together. Never hand-edit the generated answers or add a synthetic main to an
entry-free source merely to make a test executable.
