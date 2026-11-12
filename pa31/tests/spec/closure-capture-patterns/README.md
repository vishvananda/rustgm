# Closure capture patterns (source core)

Twenty-nine no_core source-core libraries reduce the closure capture failure of
pinned core's `Duration` `fmt_decimal` 9097 (`time.rs:1493`). Its
`emit_without_padding` closure tests `if let Some(integer_part) = integer_part`
on a captured `Option<u64>`. They are checked against rustc 1.98. Each case's
first line names its prefix, `default-const-closures/count.rs` before its
`ZeroablePrimitive`, which declares the `Fn` traits. Three cases use edition
2018 (`cases.tsv`'s third column).

rustc's closure capture analysis walks a pattern matched against a captured
place:

- a variant, literal, range or slice-length test reads the place;
- a by-value binding copies the place (a shared capture) or moves it (a
  by-value capture and an `FnOnce` closure);
- `ref` and `ref mut` bindings borrow it.

Precision stops at an enum variant and at an index projection, so those uses
capture the matched place itself. A tuple or struct field projection, or a
dereference of a captured reference, captures a more precise place. Edition
2018 captures whole variables.

- Positives:
  - `core-form` is core's form, an `Fn` closure called twice;
  - `match-copy`, `match-move`, `ref-mut`, `nested-enum`, `or-literal`,
    `guard-call` and `at-binding-move` are `match` forms;
  - `let-condition-move`, `while-let` and `let-else` are conditional forms;
  - `let-array` destructures an array, and `computed-let-capturing` a computed
    tuple beside a matched capture;
  - `nested-capture` and `move-closure` capture through an inner closure and a
    `move` closure;
  - `copy-then-use` moves the place after the copying closure's last call;
  - `legacy-ref-scrutinee` and `legacy-tuple-copy` capture whole variables.
- Negatives, rejected by the borrow checker:
  - `match-move-twice`, `at-binding-twice`, `move-then-use`,
    `let-else-move-then-use` and `legacy-tuple` use a moved closure or place
    (E0382);
  - `shared-conflict` moves a borrowed place (E0505);
  - `ref-mut-conflict` and `discriminant-conflict` borrow a place the closure
    holds (E0502).
- Recorded gaps, which rustc accepts:
  - `wildcard` does not capture its scrutinee, which prepared lowering cannot
    yet match;
  - `let-tuple-precise` and `ref-scrutinee` capture a field and a referent.
    They fail closed at typing.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 29
declarations agree. `obj/parallel/capture-match/core/declared.json` records
three batches, including one malformed probe and one ungraded control. The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/matched_captures.rs` checks every case's typing through the CLI. It
runs prepared lowering for every case that types, and checks bounded work. Its
ignored native test runs six ordinary-mode programs against rustc. See
`docs/pa31-closure-capture-patterns.md`.
