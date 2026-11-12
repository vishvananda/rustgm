# Comparison right operands

Pinned core's `small_sort_network` 7571
(`slice/sort/shared/smallsort.rs:355`) failed with "can't compare `*const T`
with `*mut T`". It tests `region.as_ptr() != v_base`. There `as_ptr` returns
`*const T` and `v_base` is `*mut T`.

## Diagnosis

`obj/parallel/pointer-rhs/core/declared.json` records three declared
source-core batches. Each uses `pa31/tests/spec/source-div/foundation.rs`,
followed by core's `PartialEq` and its raw pointer impls. Batches 1 and 2 had
malformed fixtures: they lacked core's `Not` and then `bool`'s `Copy`, and rustc
rejected every case with E0600 and then E0507. Batch 3 adds both fragments and
otherwise keeps the same declarations.

rustc's `check_overloaded_binop` works in three steps:

1. It gives the right operand's type a fresh variable `Rhs`.
2. `lookup_op_method` registers `*const T: PartialEq<Rhs>` and selects
   obligations where it can. The only impl then fixes `Rhs = *const T`.
3. Only then does it check the right operand and coerce it to `Rhs`, here a
   mutable-to-const pointer coercion.

Two raw pointers are also builtin scalar operands (`is_builtin_binop`). So
`enforce_builtin_binop_types` then demands that the coerced `Rhs` be a subtype
of the left operand.

RustGM's source comparison used the right operand's own type as `Rhs`. The
obligation `*const T: PartialEq<*mut T>` has no impl.

## Change

- **Selected `Rhs`.** A source comparison whose operands are not two primitive
  scalars gives `Rhs` a fresh variable. A resumable continuation
  (`Work::CoerceOperatorRhs`) refines the comparison's own goals through the
  existing call-input frontier (`refine_call_inputs`), which call arguments
  already use. It then coerces the right operand to `Rhs` with the ordinary
  coercion. A declaration wait resumes the same continuation.
- **Builtin raw pointers.** After the coercion, two raw pointer operands, under
  one shared reference, are equated. This is rustc's builtin comparison rule.
- **Erasure.** A coerced pointer comparison is still erased to the primitive
  operation (`binary BangEqual left=299 right=299` in `const-ne-mut`). The
  solved-stage check reads the coerced right type.
- **Per-body table.** The coerced right operands are cleared with the other
  per-body inference tables. Node identities are file-local, and the checker
  state is reused across bodies.

Two primitive scalars keep the existing builtin unification, under which a
coercion would be an identity. No new engine, backend, storage policy or
compiler limit is added.

Tradeoffs:

- Ordinary mode keeps its own rule, which coerces a right operand to a raw
  pointer on the left. It already accepts `*const u8 == *mut u8`
  (`pointer_coercions`, `reverse-equality`).
- Source arithmetic operators still use the right operand's own type as `Rhs`.
  rustc coerces there too (`String + &String`), but no reached core body needs
  it.
- rustc also treats function pointers and items as builtin scalars. Their
  post-coercion relation is not added, because no reached case needs it.
- Each such source comparison refines its own goals once before its right
  operand coerces (see *Bounded work*).
- Batch 2's malformed fixture moves a non-Copy `bool` out of `*self`. RustGM
  f2cb90629 accepts that move, which rustc rejects (E0507). This is a separate
  source-mode move-check gap, recorded in `declared.json`.

## Contract

**PA31** (`pa31/tests/spec/comparison-rhs/`, twelve source-core reductions).
They agree with rustc 1.98.0 12/12 (`outcomes.json`). Eleven of twelve
declarations agree with rustc. `premise-rhs` was declared accept, and its rustc
rejection is the oracle.

| Case | Form | Rust | f2cb90629 |
| --- | --- | --- | --- |
| `core-network` | core's `region.as_ptr() != v_base` loop | accept | reject |
| `const-ne-mut` | `*const T != *mut T` | accept | reject |
| `const-eq-mut` | `*const T == *mut T` | accept | reject |
| `concrete` | `*const u8 != *mut u8` | accept | reject |
| `fresh-rhs` | `a == fresh()`, `*mut U` inferred through the coercion | accept | reject |
| `ref-rhs` | `*const u8 == &x` | accept | reject |
| `mut-ref-rhs` | `*mut u8 == &mut x` | accept | reject |
| `same-const` | `*const T == *const T`, a control | accept | accept |
| `nominal` | a user `PartialEq` impl, a control | accept | accept |
| `mut-eq-const` | `*mut T == *const T` | E0308 | reject |
| `premise-rhs` | `Rhs = *mut T` through a where clause | E0308 | accept |
| `ambiguous-user` | a second `PartialEq<Local> for *const u8` | E0308, E0277 | reject |

## Qualification

Focused receipt 1794510795726503064 passes in 170.9s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the memory and
frontend suites, the CLI and the private library, plus thirteen exact tests:

- the three new tests (`dev/tests/pointer_coercions/comparison_rhs.rs`). The
  first runs the packet through the CLI, checks that `const-ne-mut` stays a
  builtin operation, and lowers every positive. The second checks bounded
  work. The third checks that `stale-node`'s two comparisons both stay
  primitive operations at their own types.
- ordinary raw pointer coercions (`pointer_coercions`);
- the source builtin binary packet (`binop_operand_types`) and four source
  comparison authority contracts;
- two guaranteed-pointer source contracts;
- PA43's pinned-call packet and its bounded work. They re-check the
  [smart receiver name set](pa43-pinned-method-calls.md), which is now boxed.

Earlier attempts stopped at:

- Clippy's function length limit, then a guarded match arm that did not count
  toward exhaustiveness;
- the private harness's `Declarations` size pin, which f2cb90629's unboxed
  smart receiver set had grown;
- the five-minute budget, after the cold private-lint rebuild;
- a private test path written with the wrong module name;
- three resource admissions refused while the shared volume was at its reserve;
- receipt 1794509679847620497, which passed before the per-body table was
  cleared. Its exploratory pin a150d249 then failed pinned core's
  `digits_to_dec_str` (`num/imp/flt2dec/mod.rs:233`, `frac_digits > 0`) with
  "source comparison operands/result require unsupported scalar
  representation". A comparison in an earlier body, in another file, had left
  its coerced operand at the same node. The four declared single-file probes
  and four multi-file probes are in `obj/parallel/pointer-rhs/regress/`.
  `stale-node` comes from them.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/pointer-rhs/scaling-raw.txt`), for 8/32/128
generic functions, each comparing `*const T != *mut T`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 33226 | 6662 | 431 |
| 32 | 41458 | 7958 | 527 |
| 128 | 74386 | 13142 | 911 |

That is 343 inference steps, 54 canonical steps and four trait selections per
comparison over the prefix's fixed cost. Adding 64 unrelated nominals, each
with its own `PartialEq` impl, adds a constant 256 inference and 1,780
canonical steps at every width, and no trait selection.

## Core progress

Exploratory pin wip2 (00a18645, `obj/parallel/pointer-rhs/wip2-dev-fast`)
carries this checkpoint's working tree on 25edf3f42. The unchanged 120s
observer ran exclusively at load 7. The census `original-wip2-comparison.txt`
compares it with the previous checkpoint's exploratory observation
(`obj/parallel/sync-view/original-wip2`):

- 14,960 item attempts, up from 13,875. The run reaches item 10031
  (`core_arch` `x86/sse2.rs:1532`), up from 8127.
- `small_sort_network` 7571 is reached and passes. No previously passing reached
  body fails.
- The first failure is `bidirectional_merge` 7581
  (`slice/sort/shared/smallsort.rs:812`, `(left, right, dst) = merge_up(...)`):
  destructuring assignment, which comes next.
- The five later known failures remain: `as_underlying` 7631 (transmute size),
  `partial_compare` 7660 and `compare` 7676 (`slice/cmp.rs`, invalid inference
  identity), and `as_slice` 7874 and `next_back` 7900 (`slice/iter.rs`,
  incompatible types).
- Five newly reached bodies fail: `is_sorted_by` 8380 (`slice/mod.rs:4782`,
  array pattern type), `clone` 8624 (`str/iter.rs:1438`, ambiguous
  obligation), `fmt_decimal` 9097 (`time.rs:1493`, capture patterns for a
  matched place) and two `core_arch` `simd.rs` `eq` bodies, 9611 and 9643
  (ambiguous obligation).

The first exploratory pin, wip1 (a150d249), found the stale per-body table (see
*Qualification*). Its observation is kept as
`obj/parallel/pointer-rhs/original-wip1`. An earlier attempt is kept as
`original-wip1-missing-inputs`: its pinned source tree had been removed with the
`rustgm-pa29-work` checkout. The 722 manifest files were then copied and
verified from `/home/vishvananda/rustgm-sources/rust-1.98.0-88d9e12ae...`
into `obj/parallel/original-core-inputs-88d9e12ae178fab0fb5cc050a94da85685d449ea`.

Not yet run: a clean candidate pin from the committed tree, and its
observation. Exploratory observations are not candidate evidence.
