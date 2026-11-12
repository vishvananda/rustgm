# Destructuring assignment

Pinned core's `bidirectional_merge` 7581
(`slice/sort/shared/smallsort.rs:812`) failed with "assignment requires a
place". It assigns `(left, right, dst) = merge_up(left, right, dst, is_less)`.
Core's integer `pow` bodies (`num/int_macros.rs`, `num/uint_macros.rs`) use the
same form, `(acc, tmp_overflow) = acc.overflowing_mul(base)`. No other core
assignee form appears.

## Diagnosis

Every assignment required its left operand to be a place: a local, a static, a
field, an index or a dereference. A tuple on the left was entered as a tuple
value and rejected. `_` on its own was not an expression at all.

rustc desugars `assignee = value` into
`{ let <pattern with fresh bindings> = value; leaf = binding; ... }`. The right
operand is evaluated first. Each leaf is then assigned from left to right, and
`_` assigns nothing.

`obj/parallel/assignee/core/declared.json` records five declared batches. All
20 well-formed probes agree with their declarations. Three probes were
malformed: a closure and an array index on a prefix without the Fn traits or
`Index`, and a closure prefix without `Freeze`. Batch 1 ran rustc and the
previous exploratory pin before any implementation.

## Change

- **Syntax.** `crate::syntax::assignee` classifies an assignment's left operand.
  A tuple (or a group around one) or `_` destructures. Its walk lists shells,
  `_` and place leaves in source order, skipping inactive elements.
- **Typing.** A destructuring assignment enters its assignee instead of a
  value. Tuple shells take the tuple of their elements' types, `_` takes a fresh
  variable, and place leaves are entered as an assignment's place (union write
  and mutable language place marks included). Each leaf must be assignable as
  `leaf = value` is (`assignment_target`). The right operand coerces to the
  assembled type. The node's operation is `Operation::Destructure`
  (`destructure` in `--emit-types`).
- **Closures.** A captured leaf is a mutable use, and `_` mentions nothing.
- **Temporary scopes.** `_` has no operands.
- **Body construction.** The right operand is evaluated into a place: a
  materialized temporary, or the place a place expression names. Its tuple
  fields are projected. Each leaf's place is evaluated after the stores before
  it, and its field is moved or copied into it. `_` and unread fields stay in
  the right operand's place. Ownership checking runs on these ordinary stores,
  so E0384, E0507 and E0382 need no new rule.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Only tuple and `_` assignees are implemented. Rest (`(a, ..)`), array, struct
  and tuple struct assignees fail closed. A struct with a `..` rest does not
  yet parse. rustc accepts all five, and core uses none of them. A unit struct
  path assignee stays an ordinary assignment, which rejects it.
- The right operand coerces to the assembled tuple once. rustc instead
  coerces each binding at its own `leaf = binding`. For a tuple expression the
  two agree, because its elements coerce to the expected element types. A
  non-literal right operand whose field needs a coercion to its leaf type is
  rejected here. The declared probe `field-coercion` (batch 5), `(p,) = f(r)`
  with `f` returning `(&mut u8,)` and `p: *mut u8`, is accepted by rustc and
  rejected here ("incompatible types").
- Fields move into their leaves at each store, not first into fresh bindings.
  If a later leaf's place evaluation panics, the unread fields drop with the
  right operand's place rather than as bindings. The drop set is the same. The
  right operand's temporaries end with the assignment statement, as the
  desugared block's do.

## Contract

**PA31** (`pa31/tests/spec/destructuring-assignment/`, seventeen source-core
reductions). They agree with rustc 1.98.0 17/17 (`outcomes.json`), and so do
all declarations. The base is the previous exploratory pin (batch 1) or the
clean candidate 00a18645 (later batches), and it rejects every case with
"assignment requires a place" or, for `_`, an unimplemented body construct.

| Case | Form | Rust | Stage |
| --- | --- | --- | --- |
| `core-merge` | core's `(left, right, dst) = merge_up(...)`, twice | accept | |
| `swap` | `(a, b) = (b, a)` | accept | |
| `nested` | `((a, b), c) = ((1, 2), 3)` | accept | |
| `wildcard-leaf` | `(a, _) = (1, 2)` | accept | |
| `place-leaves` | `(s.x, *r) = (1, 2)` | accept | |
| `underscore` | `_ = f()` | accept | |
| `deferred-init` | two declared locals initialized | accept | |
| `move-nominal` | two non-Copy values moved | accept | |
| `group-leaf` | `((a), (b)) = (1, 2)` | accept | |
| `unit` | `() = ()` | accept | |
| `closure-capture` | a closure assigns captured locals | accept | |
| `literal-leaf` | `(1u8, b) = (2, 3)` | E0070 | typing |
| `arity` | `(a, b) = (1, 2, 3)` | E0308 | typing |
| `compound` | `(a, b) += (1, 2)` | E0067, E0368 | typing |
| `immutable-leaf` | `(a, b) = (2, 3)` with `a` initialized and immutable | E0384 | ownership |
| `borrowed-source` | `(x, y) = *r` with non-Copy fields | E0507 | ownership |
| `moved-source` | `p` used after `(x, y) = p` | E0382 | ownership |

## Qualification

Focused receipt 1794512552168430216 passes in 106.6s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the ownership
and lowering suites and the CLI, plus four exact tests:

- the two new tests (`dev/tests/destructuring_assignments.rs`). The first runs
  the packet's typing through the CLI and requires a `destructure` operation
  for every positive. It runs prepared lowering for every case that types, so
  ownership rejects the three ownership negatives with their reasons. The
  second checks bounded work.
- `place_mentions::wildcard_lifetimes_match_pinned_rust`, for `let _`.
- `body_types::control_flow_and_assignment_types_match_pinned_rustc`, for
  ordinary assignments.

Earlier attempt 1794512359737312080 stopped at `_` in the temporary scope walk.
`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/assignee/scaling-raw.txt`), for 8/32/128 functions
each swapping two locals:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 29176 | 5797 | 375 |
| 32 | 29728 | 5845 | 375 |
| 128 | 31936 | 6037 | 375 |

That is 23 inference steps and 2 canonical steps per destructuring swap, and
no trait selection.

## Core progress

Exploratory pin wip2 (2ddb3a67, `obj/parallel/assignee/wip2-dev-fast`) carries
this checkpoint's working tree on db59a807b. The unchanged 120s observer ran
exclusively from load 22, which rose to 51 under external work. The census
`original-wip2-comparison.txt` compares it with the clean comparison candidate
(`obj/parallel/pointer-rhs/original-candidate`):

- `bidirectional_merge` 7581 is reached and passes. No previously passing
  reached body fails.
- 14,027 item attempts, against the candidate's 15,493 at load 7. The run ends
  at item 8360 (`slice/mod.rs:3725`), with 5 records. The shorter reach is load;
  it is not a failure.
- The first failure is the known `as_underlying` 7631 (`slice/cmp.rs:52`,
  "source transmute requires equal source and destination sizes"). It is
  followed by `partial_compare` 7660 and `compare` 7676 (invalid inference
  identity) and by `as_slice` 7874 and `next_back` 7900 (incompatible types).

Clean candidate `obj/parallel/assignee/candidate-dev-fast` (83906f0ac, no
patch) builds the same binary, 2ddb3a67. The unchanged 120s observer ran it
exclusively from load 22. Its census, `original-candidate-comparison.txt`,
compares it with the same clean comparison candidate:

- `bidirectional_merge` 7581 is reached and passes. No previously passing
  reached body fails.
- 13,842 item attempts and 5 records. The run ends at item 8060
  (`slice/iter.rs:1951`). The shorter reach comes from load, not a failure.
- The first failure and the four after it are the same as wip2's:
  `as_underlying` 7631, `partial_compare` 7660, `compare` 7676, `as_slice`
  7874 and `next_back` 7900.
