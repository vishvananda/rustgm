# Blanket candidate winnowing (source core)

Eleven no_core source-core libraries check projection results whose subject
matches the header of the blanket `impl<I: Iterator> IntoIterator for I` and
of a second impl, against rustc 1.98. Pinned core's `slice/iter/macros.rs:494`
implements `Default` for `Iter` and `IterMut` as `(&[]).into_iter()`. Its
receiver is `&[?E; 0]`, and `?E` comes from `<&[?E; 0] as
IntoIterator>::IntoIter`. `core-form` and `mut-form` reduce it. Every case
shares an authored prefix: `Option`, `Iterator`, `IntoIterator` with core's
blanket impl, `Default`, `Iter`/`IterMut`, the four slice and array reference
`IntoIterator` impls, and the `sized`, `clone`, `copy` and `legacy_receiver`
lang items. Interfaces follow pinned core's `iter/traits` and `slice/iter.rs`;
bodies are `loop {}` and make no runtime claim.

rustc's selection evaluates each remaining candidate and discards those whose
nested obligations fail. `&[?E; 0]: Iterator` has no candidate, so the blanket
impl does not compete with the array impl.

- Positives:
  - `core-form`, `mut-form`, `let-annotated` and `path-call` reach the array
    impls with an unknown element;
  - `nominal-subject` reaches a nominal impl for `Bag<?T>`;
  - `known-element`, `slice-receiver` and `iterator-subject`, whose receiver
    is an `Iter`, are controls.
- Negatives:
  - `unresolved-element` (E0282) leaves the element unknown;
  - `wrong-target` (E0308) returns the result as an `IterMut`;
  - `ambiguous-subject` (E0283): `Wrap<?T>: Iterator` may hold through
    `Wrap<u8>`, so the blanket candidate is not discarded and competes with
    an impl for `Wrap<u16>`.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All eleven
declarations agree. `obj/parallel/array-into-iter/core/declared.json` records
four batches. Batch 1 lacked the `copy` lang item, and batch 2 corrected it.
Batch 3 holds ungraded diagnostics. The cases keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/winnowed_projections.rs` checks every case's typing through the
CLI. It runs prepared lowering for every accepted case, and checks each
rejection's message and site. It also checks bounded work. See
`docs/pa31-blanket-candidate-winnowing.md`.
