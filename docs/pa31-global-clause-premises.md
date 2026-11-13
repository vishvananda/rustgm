# Global clause premises

Release candidate9's core census (`obj/parallel/capture-match/candidate9-release`,
4f989cae, clean 99a409d03) fails portable-simd's `div` 36996
(`core_simd/src/ops.rs:139`) with "trait obligation requires an unambiguous
matching assumption or implementation", and nine more bodies are blocked
behind it. `for_base_types!` expands each integer `Div` and `Rem` impl with a
global where clause, one that names no parameter, and `int_divrem_guard!`
selects through a mask:

```rust
impl<const N: usize> Div<Self> for Simd<i8, N>
where
    i8: SimdElement,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        // ...
        let rhs = if <i8>::MIN != 0 {
            (self.simd_eq(Simd::splat(<i8>::MIN))
            & rhs.simd_eq(Simd::splat(-1i64 as _)))
            .select(Simd::splat(1 as _), rhs)
        } else {
            rhs
        };
        // ...
    }
}
```

rustc accepts it.

## Diagnosis

`obj/parallel/simd-divrem/declared.json` records four declared batches. Batch
1 was malformed: its prefix lacked `Neg` impls for its negated literals.
Batch 2 was malformed too: rustc requires the `freeze` lang item for its
`const fn`s. Batches 3 and 4 are the packet. Temporary instrumented dev-fast
builds, not committed, traced the failure (`obj/parallel/simd-divrem/trace/`).

The batches narrowed the form to

```rust
pub fn f<const N: usize>(m: Mask<i8, N>, b: Simd<i8, N>) -> Simd<i8, N>
where
    i8: SimdElement,
{
    m.select(Simd::splat(1 as _), b)
}
```

It types without the where clause, and with `u8`'s clause, whose mask is
`i8`.

- The call's `Mask<i8, N>: MetaSized` goal defers forever. Inside its search,
  `<i8 as SimdElement>::Mask == i8`, required by `MaskElement:
  SimdElement<Mask = Self>`, defers when the equality completes.
- The normalizer of `<i8 as SimdElement>::Mask` suspends at its trait premise
  `i8: SimdElement`. The search proves that premise from the caller clause
  first. A clause carries no written result equality, so the normalizer
  returns the symbolic projection, which never becomes `i8`.
- rustc discards a global where clause in favor of an impl that applies
  (rust-lang/rust#50825), so its normalization reads `type Mask = i8` from
  the impl. RustGM's declaration normal form already retries a concrete
  requirement against implementations (`nominals/traits/normal_form.rs`).
  The body search had no such rule.

## Change

- **Implementation-first premise.** When a normalizer opens a trait premise
  that is concrete, naming no parameter, projection, non-static free region or
  escaping binder, its child frame selects only implementations. That frame
  skips the caller assumption lookup. It neither reuses nor records a cached
  decision for its goal, because a decision that ignores the caller's clauses
  is not the goal's decision.
- **Clause fallback.** If the implementation-first frame proves nothing, the
  normalizer reopens the same request once, with the caller's clauses
  available as before. A concrete premise that only a caller clause proves
  keeps its existing symbolic result.

`Frame` records the choice as `Sources::All` or `Sources::Implementations`. No
new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Only a normalizer's premise changes. A concrete trait goal outside a
  normalizer still prefers a matching caller clause. Its proof can differ from
  rustc's only in which candidate it names, not in whether it holds: a
  concrete clause is itself an obligation that the caller proves.
- Scoped callable requests keep the ordinary order. Their premise sits beneath
  a callable binder, and none of the packet's forms needs one.
- A premise that no implementation proves, such as a clause for a type
  without an impl, costs one extra search before the clause is tried. rustc
  1.98.0 rejects such a trivially false clause with E0277 unless
  `trivial_bounds` is enabled (`obj/parallel/simd-divrem/trivial/`), so it
  arises only in programs that rustc already rejects, or under that feature.
- Implementation-first frames do not share the goal cache, so a concrete
  premise is searched again wherever it recurs, with or without a clause.
  Body work stays linear, and the clause form stays below the clause-free
  control (see Qualification). Method selection probes cost slightly more.
  Source-core mode charges them to a crate-wide budget of 16 steps per
  input-graph element, and 256 copies of the clause-free `select` exhaust it at
  function 229 on candidate9 and 222 with this change. The clause form
  exhausts it at function 215. That budget predates this change: candidate9
  already exhausts it at 128 clause-free copies
  (`obj/parallel/simd-divrem/width-base/`). A cache entry keyed by the
  implementation-only order could remove the repeat, and is left for later.
- Messages differ from rustc's. Every rejection is at typing, where rustc
  rejects.

## Contract

**PA31** (`pa31/tests/spec/global-clause-premises/`, 25 source-core
reductions). The base is release candidate9. Each file is a declared batch
file, byte for byte. Every declaration agrees with rustc 1.98.0
(`outcomes.json`).

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `core-form` | core's guard for `i8` | accept | obligation | accept |
| `core-form-unsigned` | core's guard for `u8` | accept | accept | accept |
| `global-clause-two-scalars` | the `i8` and `u8` guards in one library | accept | obligation | accept |
| `core-form-no-where` | the `i8` guard without its clause | accept | accept | accept |
| `global-where-guard` | `&` of two comparisons, then `select` | accept | obligation | accept |
| `global-where-select` | `select` alone | accept | obligation | accept |
| `global-clause-generic-companion` | `select` beside a `T: SimdElement` bound | accept | obligation | accept |
| `global-where-eq` | `simd_eq` | accept | accept | accept |
| `global-where-eq-splat` | `simd_eq` of a splat | accept | accept | accept |
| `global-where-unsigned-mask` | `u8`'s `simd_eq` into `Mask<i8, N>` | accept | accept | accept |
| `global-where-projection` | a `<i8 as SimdElement>::Mask` result | accept | accept | accept |
| `global-clause-let-annotation` | a `<i8 as SimdElement>::Mask` local | accept | accept | accept |
| `global-clause-nested-projection` | `<<i8 as SimdElement>::Mask as SimdElement>::Mask` | accept | accept | accept |
| `select-splat-cast` | `select` without the clause | accept | accept | accept |
| `select-splat-typed` | an `i8` splat | accept | accept | accept |
| `select-false-first` | the splat second | accept | accept | accept |
| `select-ufcs` | `Select::select(m, ..)` | accept | accept | accept |
| `and-select` | `&`, then `select` | accept | accept | accept |
| `eq-any` | `simd_eq(..).any()` | accept | accept | accept |
| `select-mask-values` | `select` of masks | accept | accept | accept |
| `select-literal-fallback` | integer literal fallback to `i32` lanes | accept | accept | accept |
| `select-wrong-element` | a `u16` splat for `i8` lanes | E0308 | incompatible | incompatible |
| `select-non-mask` | `select` on `u8` | E0599 | method | method |
| `global-where-wrong-projection` | `u16` returned as its `i8` mask | E0308 | incompatible | incompatible |
| `global-clause-select-wrong-output` | `select`'s `Simd<i8, N>` as `Simd<i32, N>` | E0308 | incompatible | incompatible |

- "Obligation" is "trait obligation requires an unambiguous matching
  assumption or implementation".
- "Incompatible" is "incompatible types".
- "Method" is "no applicable method for this receiver".

Fourteen cases declare a global clause: the first thirteen rows except
`core-form-no-where`, and the last two rejections. The other eleven declare
none.

## Qualification

One focused receipt passes on the final source: 1794556164673085248 (92.5s).
It covers:

- assignment metadata, formatting and the architecture audit;
- strict Clippy for the traits and projections suites and the CLI;
- `global_clause_premises_select_implementations_like_rustc`, which types all
  25 cases through the CLI, checks each rejection's message at its site and
  lowers every accepted case;
- `global_clause_premise_work_tracks_functions`;
- `body_projections_normalize_like_rustc`, the three `winnowed_projections`
  tests, ten `projection_normalization` tests, three
  `normalization_occurrences` tests and `partial_projection_selection_matches_rustc`;
- every text case of PA20 (1,616), PA21 (867) and PA24 (716).

The same source's dev-fast CLI, frozen as sha256 a45a33a3
(`obj/parallel/simd-divrem/cli-wip/`), passed all 9,307 active text cases in
PA1 to PA24 (`obj/parallel/simd-divrem/text-all.log`). No reference changes.
It also reproduced every packet outcome in `outcomes.json`.

Earlier attempts failed:

- 1794555576388265368, on a selection that named the suite rather than the
  module;
- 1794555630846080503, on Clippy's `struct_excessive_bools` for a fourth
  `Frame` flag, which became `Sources`;
- 1794555773896462278, on the scaling test's 128 copies, which exhausted the
  source method selection budget described above.

Body work is linear in the functions that declare the clause, and stays below
the clause-free `select-splat-cast` copies:

| Functions | Inference steps | Canonical steps | Trait selections | Control inference | Control canonical |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 14,023 | 2,955 | 158 | 14,879 | 5,259 |
| 16 | 33,619 | 4,719 | 362 | 36,419 | 13,935 |
| 64 | 112,003 | 11,775 | 1,178 | 122,579 | 48,639 |

Each quadrupling of the function count multiplies every measure by at most
3.3. The rows are in `obj/parallel/simd-divrem/qual/scaling-raw.txt`.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated. Full release
regular and native tests are left to scheduled CI.

## Core progress

Release candidate10 (`obj/parallel/capture-match/candidate10-release`, clean
534f3cc67, CLI 070d8da9) carries this change. The unchanged 120s observer ran
it exclusively from load 11.4. It reached the end of the body pass in 23.1s,
with all 35,120 item attempts, 25,850 checked bodies, and 4 failed and 4
blocked bodies, against candidate9's 5 and 13. `div` 36996 and the nine `div`
and `rem` bodies it blocked pass. Every candidate10 record is a candidate9
record, so no previously passing body failed
(`obj/parallel/capture-match/original-candidate10-comparison.json`).
