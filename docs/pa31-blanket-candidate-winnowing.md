# Blanket candidate winnowing

Release candidate6's core census (`obj/parallel/capture-match/candidate6-release`,
54039f5d, clean 0d5530cca) fails the `default` body of
`slice/iter/macros.rs:494` for `Iter` with "trait obligation requires an
unambiguous matching assumption or implementation". The same macro's `default`
for `IterMut` is blocked. The body is

```rust
impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        (&[]).into_iter()
    }
}
```

rustc accepts it. The receiver is `&[?E; 0]`, and `?E` is known only once
`<&[?E; 0] as IntoIterator>::IntoIter` is normalized to `Iter<'_, ?E>` and
equated with `Self`.

## Diagnosis

`obj/parallel/array-into-iter/core/declared.json` records four declared
batches. Batch 1 was malformed: its prefix had no `copy` lang item, which
rustc requires. Batch 2 corrected it. On the base, core's form, the `&mut`
form, a `let` annotation and the path call `IntoIterator::into_iter(&[])`
failed with the obligation failure. A known element type and a slice receiver
passed.

Batch 3's ungraded diagnostics removed impls from core's form:

- without the blanket `impl<I: Iterator> IntoIterator for I`, it typed;
- with only the `&'a [T; N]` impl, it typed;
- with the blanket and both array reference impls, it failed.

Two temporary trace pins (`wipiter`, `wipiter2`, both retired) printed the
selection search and the body's projection queries.

- The selection search already discards the blanket candidate. Its trial's
  condition `&[?E; 0]: Iterator` is absent. The array impl's trial waits on
  `?E: Sized`, and the refining query commits it as partial inference.
- `?E` is resolved by neither. Without the blanket, the projection's
  symbolic result does it: `symbolic_result` in
  `sema/traits/selection/result_templates.rs` finds a unique compatible header
  and exports `Iter<'a, ?E>`. With the blanket, two headers are compatible.
  The blanket header matches every subject, so no symbolic result is exposed,
  and the projection waits forever.

rustc's selection winnows the same two candidates. When several candidates
remain, it evaluates each and discards those whose nested obligations fail
(`EvaluatedToErr`). `&[?E; 0]: Iterator` has no candidate, so only the array
impl remains, and normalization uses it.

## Change

- **Winnowing.** `symbolic_result` collects every compatible header, as
  before. When more than one is compatible, it selects each candidate's trait
  conditions in that candidate's probe with the ordinary selector. A
  candidate is discarded when one of those conditions is absent. Exactly one
  remaining candidate exposes its result, as a unique header did.
- **Conservative outcomes.** A deferred, pending, proven or refuted condition
  keeps its candidate. A selector error is propagated. The probe steps each
  selection adds are charged to the caller.
- **Bounded scan.** Winnowing never discards a candidate without a trait
  condition. So the header scan exposes nothing at the second compatible
  unconditional header, before instantiating the rest, as the scan did before
  winnowing.

The unique header path and the selection search are unchanged. The exported
result still carries no premise proof, and the body still proves the original
obligation with the ordinary selector. No new engine, backend, storage policy
or compiler limit is added.

Tradeoffs:

- Only trait conditions are winnowed. Builtin conditions such as `Sized` and
  higher-ranked conditions keep their candidate. rustc evaluates every nested
  obligation.
- A refuted condition, from a negative impl, keeps its candidate, although
  rustc would discard it. Consuming the refutation is left to the ordinary
  selector.
- Each compatible conditional header now costs a probe and its condition
  selections before uniqueness is decided. A family of unconditional headers
  costs no more than before.
- The scan stops before instantiating any header after the second
  unconditional one, so an error from such a header is no longer reported
  here. The ordinary selector still meets it when the body proves the
  obligation.
- Typing then fails `unresolved-element` with the obligation failure where
  rustc reports E0282, and `wrong-target` with "incompatible types" where rustc
  reports E0308. The base gave the obligation failure for both.

## Contract

**PA31** (`pa31/tests/spec/blanket-candidate-winnowing/`, 11 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).
Eight come from batch 2 and three from batch 4. Batch 4 was declared after
the trace diagnosis, before rustc, any RustGM run of its files and any fix.
The base is release candidate6.

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `core-form` | core's `Iter` `default` | accept | obligation | accept |
| `mut-form` | core's `IterMut` `default` | accept | obligation | accept |
| `let-annotated` | a `let` annotation | accept | obligation | accept |
| `path-call` | `IntoIterator::into_iter(&[])` | accept | obligation | accept |
| `nominal-subject` | `bag().into_iter()` for `Bag<?T>` | accept | obligation | accept |
| `known-element` | `(&[1u8, 2]).into_iter()` | accept | accept | accept |
| `slice-receiver` | `s.into_iter()` on `&[T]` | accept | accept | accept |
| `iterator-subject` | an `Iter` receiver, the blanket's own subject | accept | accept | accept |
| `unresolved-element` | `let _ = (&[]).into_iter();` | E0282 | obligation | obligation |
| `wrong-target` | an `IterMut<'a, u8>` return | E0308 | obligation | incompatible |
| `ambiguous-subject` | `Wrap<?T>`, with `Wrap<u8>: Iterator` and an impl for `Wrap<u16>` | E0283 | obligation | obligation |

- "Obligation" is "trait obligation requires an unambiguous matching assumption
  or implementation".
- "Incompatible" is "incompatible types".

`ambiguous-subject` stays ambiguous: the blanket's condition `Wrap<?T>:
Iterator` is not absent, since `Wrap<u8>` may apply. rustc reports "multiple
`impl`s satisfying `Wrap<_>: IntoIterator` found" (E0283), with E0282 for
`wrap`'s parameter. The test requires each rejection's message at its site.

## Qualification

One focused receipt passes with exact selections on the final source:
1794544733236803927 (204.0s). It covers:

- assignment metadata, formatting and the architecture audit;
- strict Clippy for the traits and projections suites and the CLI, and
  strict private Clippy;
- the two new tests in `dev/tests/winnowed_projections.rs`. The first types
  every case through the CLI, runs prepared lowering for every accepted case
  and requires each rejection's message at its site. The second checks
  bounded work and lowers every width;
- the symbolic template invariants (seven), including
  `ambiguous_or_input_refining_headers_cannot_publish_shapes_or_bind_holes`, and
  the inferred member invariants (five);
- `partial_projection_selection` (two) and `definition_readiness`'s
  `selected_definitions_preserve_source_order_and_lifetime_outcomes`;
- PA20's text oracles, 1616 of 1616.

Earlier attempt: 1794544699868836322 named suites instead of test modules, and
fastcheck rejected the selection before building.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/array-into-iter/scaling-raw.txt`), for 8/32/128
functions, each returning `(&[]).into_iter()` as an `Iter`:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 11674 | 857 | 72 |
| 32 | 46474 | 2921 | 288 |
| 128 | 185674 | 11177 | 1152 |

That is 1,450 inference steps, 86 canonical steps and 9 trait selections per
function, exactly linear across widths. The base cannot type these widths.

### Scan bound

One focused receipt passes on the bounded scan's final source:
1794547593986105559 (181.0s), shared with the
[AVX-512 operand checkpoint](pa41-avx512-operands.md). It reruns every
selection above with `unconditional_family_scan_stops_at_the_second_header`,
and PA20's text oracles. Earlier attempts:

- 1794546806234981648 failed private Clippy in the AVX-512 invariant;
- 1794546957138554308: the family source lacked `clone_fn`;
- 1794547199809374980 also named planned PA41's text stage, which the harness
  refuses;
- 1794547280331928866 passed before the family test's bounds and comment were
  corrected to the measured growth.

Family work (`obj/parallel/array-into-iter/family-scaling-raw.txt` and
`family-scaling-control.txt`) covers 16 uses of `1 << 15 | … | 1 << 8` over
the first K integer `Shl` impls, less the same source without uses:

| Impls | Inference | Canonical | Selections | Inference without the bound |
| --- | --- | --- | --- | --- |
| 8 | 752720 | 5296 | 2880 | 1287664 |
| 32 | 924976 | 5296 | 3136 | 2490576 |
| 128 | 1428784 | 5296 | 3136 | 6948048 |

Canonical work and selections are flat. Inference grows by about 352 steps
per impl and use, against 2,948 without the bound, which fails the test at
128 impls. The control build reverted only the bound and is not committed.

## Core progress

Release candidate7 (`obj/parallel/capture-match/candidate7-release`, clean
8f4e2f737, CLI f43e63a3) carried the first form of this change, which
instantiated every compatible header before deciding uniqueness. The
unchanged 120s observer ran it exclusively from load 7.3. After 15.4s, item
14975 `_mm512_movm_epi16` (`x86/avx512bw.rs:11065`), which candidate6 passes,
failed with "type constraint work budget exceeded", and the census ended at
that budget record. It made 18,997 item attempts, against candidate6's 35,120,
and had 12 records. The census reached neither the `default` bodies nor the
derived bodies.

```rust
let one = _mm512_set1_epi16(
    1 << 15
        | 1 << 14
        // ...
        | 1 << 0,
)
```

Each `?int << ?int` projection had a compatible header for every integer `Shl`
impl, and each symbolic result query instantiated all of them. The bounded
scan fixes this. `unconditional_family_scan_stops_at_the_second_header` reproduces the
growth: without the bound it fails at 128 impls. No core observation of the
bounded form has run yet.
