# PA24 fixture ownership

The manifest covers 578 source files: 408 accepted
and 170 rejected. Every source outcome was independently checked with pinned
rustc before reference generation.

The accepted native classifications comprise 395 runtime comparisons, three
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
correction and release qualification. It is the consolidated inventory; an early
32-case capture plan is not the assignment's complete scope.

Existing `*-hypotheses.json`, `*-outcomes.json` and other draft outcome files
retain independently declared expectations and source hashes for their respective
implementation checkpoints. Their historical unsupported observations are not
current language expectations. In particular, the input-only lifetime bridge
now accepts `formation-truncation/reference-result.rs`, while
`output-interfaces/escape-outer.rs` must fail its ordinary lifetime proof.

Required regressions belong here when PA24 is the earliest assignment owning all
their features. Preserve upstream bytes and attribution. Every input
must appear in `manifest.json` with its explicit source/native classification;
generated `.ref`, `.ref.stderr`, `.ref.exit_status` and `reference.json` files ship
together. Never hand-edit the generated answers or add a synthetic main to an
entry-free source merely to make a test executable.

The `spec/shared-reborrow-captures/` controls distinguish identical shared return
values from fresh call/partial-annotation reborrows. Keep actual source capture
ancestry across final coercion resets and branch joins. Explicit dereferences,
move closures and genuine call reborrows retain their independently checked
positive behavior; short-lived carriers and local references still reject.

The `spec/callable-proof-width/` reductions freeze three independently accepted
width128 callable/context/object inputs. Their original scaling invariants and
negative context controls remain in the maintainer suites.

The `spec/callable-results/` packet adds 51 reductions for generic callable
outputs before structural consumers: 38 executable programs and 13 language
rejections. Associated outputs, opaque adapters, moves, borrowed results and
quantified correlation retain their ordinary proof. Six width/noise cases
exercise bounded work without scanning unrelated types. See the
[checkpoint review](../../docs/pa24-callable-results.md).
