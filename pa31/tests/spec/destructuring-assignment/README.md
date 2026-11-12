# Destructuring assignment (source core)

Seventeen no_core source-core libraries reduce pinned core's
`bidirectional_merge` 7581 (`slice/sort/shared/smallsort.rs:812`,
`(left, right, dst) = merge_up(left, right, dst, is_less)`). They are checked
against rustc 1.98. Each case's first line names its prefix. Sixteen use
`pa31/tests/spec/source-div/foundation.rs`. `closure-capture` uses
`pa31/tests/spec/scoped-body-selections/closure-no-caller-region.rs` before its
`Option`, plus core's `Freeze`, because closures need the Fn traits.

rustc desugars `assignee = value` into `{ let <pattern> = value; leaf = binding;
... }`. The right operand is evaluated first. Each leaf is then assigned from
left to right, and `_` assigns nothing.

- Positives: `core-merge` is core's form. `swap`, `nested`, `group-leaf` and
  `unit` are tuple forms. `wildcard-leaf` and `underscore` (`_ = f()`) use `_`.
  `place-leaves` assigns a field and a dereferenced `&mut u8`.
  `deferred-init` initializes two declared locals. `move-nominal` moves two
  non-Copy values. `closure-capture` assigns captured locals.
- Typing rejects `literal-leaf` (E0070), `arity` (E0308) and `compound`
  (E0067, E0368).
- Ownership rejects `immutable-leaf` (E0384), `borrowed-source` (E0507) and
  `moved-source` (E0382).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run. All 17 declarations agree, and both are recorded in
`outcomes.json`. Three malformed probes are recorded in
`obj/parallel/assignee/core/declared.json`: a closure and an array index on the
foundation prefix, and a closure prefix without `Freeze`. So are five
rustc-accepted forms that RustGM does not yet implement: rest, array, struct
(with and without `..`) and tuple struct assignees. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/destructuring_assignments.rs` checks every case's typing through the
CLI. It also runs prepared lowering, which checks ownership, and checks bounded
work. See `docs/pa31-destructuring-assignment.md`.
