# Derived tuple field privacy

Release candidate3 (`obj/parallel/capture-match/candidate3-release`, 8d56d3e0,
clean fc41ecb7c) was the first core observation with a release pin. The
unchanged 120s observer reached the end of the body pass in 23.7s:

- 42,310 declarations checked, none failed;
- 25,760 bodies checked, 60 failed and 38 blocked;
- the completion pass was not run, because bodies failed.

31 of the 60 failures were "aggregate field is private". All of them are
derived bodies on tuple structs whose fields are private to their own module:
26 `fmt` and 5 `eq`. An example is `array/mod.rs:182`:

```rust
#[derive(Debug, Copy, Clone)]
pub struct TryFromSliceError(());
```

## Diagnosis

`obj/parallel/derive-privacy/core/declared.json` records four declared
batches. Batch 1 was malformed. `derive` is not in a submodule's scope under
`no_core`, and the prefix has no `&Token == &Token`. Batch 2's
`enum-tuple-submodule` and batch 3's `tuple-clone-copy-eq` were malformed
diagnostics. The prefix has no `&u8 == &u8` and no `Clone` assertion helper.

On the base:

- named fields passed, and so did a tuple struct in the root module;
- every private tuple field in a submodule failed.

The base derive output spelled the index of `self.0` as synthetic text in the
derive's own context. Field selection checks privacy from the module of the
field token's context, which here was the derive macro's module, not the
item's. A named field's spelling is copied from the item, so it kept the
item's context. rustc spans the index with its field
(`Ident::from_str_and_span(index, field.span)`, `deriving/generic/mod.rs`).

## Change

- **Anchored indices.** A derived tuple index is a new `OriginKind::Anchored`
  origin whose original location is its field's first byte. Its generated text
  is an unsuffixed decimal index. It claims no copied bytes and grants no
  allowance. The context and edition lookups follow the anchor, so the index
  takes the syntax context and edition of the field. That is the item's module,
  or the `macro_rules` definition that wrote the field (`tuple-macro-defined`).
- **Uses.** Comparisons (`self.0 == other.0`), `Clone`'s field calls and
  `Debug`'s receiver fields (`&self.0`) emit anchored indices. Pattern
  bindings and named fields are unchanged.
- **Validation.**
  - Derive output requires each anchored origin to be one byte of the item's
    file at an actual field's start, with no context or introduction and a
    decimal spelling. Otherwise it fails with "derived field index lost its
    actual field".
  - The derive list's vocabulary replay accepts an anchored origin only
    inside its owner item.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- The whole index takes the context of its field's first byte, as rustc's
  span does. For a field written by `macro_rules`, privacy is therefore checked
  from that macro's definition context.
- Each derived field index costs one origin, as each copied spelling already
  does. Validation adds one binary search over the item's fields.
- `Clone` has no packet case. The prefix lacks the `Clone` derive's assertion
  helper, and core's `Copy` tuple structs clone by `*self`. The existing
  `derive_contracts` tests cover the changed `Clone` path.
- RustGM's rejection message differs from rustc's E0616 text.

## Contract

**PA31** (`pa31/tests/spec/derive-field-privacy/`, 17 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`). Six
come from batch 2, five from batch 3 and six from batch 4. Batch 3 was
declared after the change was written. Batch 4 was declared after the first
focused receipt passed. The base is release candidate3 (8d56d3e0, fc41ecb7c).

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `tuple-submodule` | `PartialEq` on `Token(u8)` in a module | accept | private | accept |
| `tuple-two-fields` | `Pair(u8, pub(crate) bool)` | accept | private | accept |
| `tuple-nested` | a nested module | accept | private | accept |
| `tuple-mixed-visibility` | private, `pub` and `pub(crate)` fields | accept | private | accept |
| `tuple-generic` | `Wrap<T>(T)` | accept | private | accept |
| `tuple-macro-defined` | a `macro_rules` struct | accept | private | accept |
| `tuple-eleven-fields` | eleven fields | accept | private | accept |
| `debug-tuple-submodule` | `Debug` on `Token(u8)` | accept | private | accept |
| `debug-unit-field` | core's `TryFromSliceError(())` | accept | private | accept |
| `debug-two-fields` | `Debug` on two fields | accept | private | accept |
| `debug-eq-both` | `Debug` and `PartialEq` | accept | private | accept |
| `record-submodule` | a named field | accept | accept | accept |
| `debug-record-submodule` | `Debug` on a named field | accept | accept | accept |
| `tuple-root` | a root-module tuple struct | accept | accept | accept |
| `outside-access` | `.0` from outside the module | E0616 | private | private |
| `manual-impl-outside` | a manual `PartialEq` outside | E0616 | private | private |
| `debug-outside-access` | a manual `Debug` outside | E0616 | private | private |

"Private" is "aggregate field is private" at typing.

## Qualification

The focused receipt 1794532983891913173 (122.3s) passes with exact
selections. It covers assignment metadata, formatting, the architecture
audit, strict Clippy for the frontend, traits and ownership suites and the
CLI, and strict private Clippy. Its 15 tests are:

- the two new tests in `dev/tests/tuple_index_privacy.rs`. The first runs the
  packet's typing through the CLI and prepared lowering for every accepted
  case. It requires "aggregate field is private" for each rejection. The
  second checks bounded work and lowers every width;
- the `derive_contracts` (two) and `derived_permissions` (two) modules;
- nine library tests:
  - two context origin and two edition provenance invariants;
  - three derive list invariants, including actual-location `Clone` field
    obligations;
  - the derive storage and the crate-root derive text invariants.

That receipt ran with the packet's first eleven cases. The six batch 4 cases
ran on its CLI (dev-fast 321591fd). On the final tree, with all seventeen
cases and the documentation, 1794533487664128269 (44.5s) passes the two
packet tests, with assignment metadata, formatting, the architecture audit
and strict Clippy for the frontend suite and the CLI.

Earlier attempts:

- 1794532499764911533 failed Clippy: `require` in the derive list vocabulary
  exceeded the function length limit. The owner-text check is now a helper.
- 1794532605720832722 passed Clippy and private Clippy. Its build was refused
  at the 16 GiB output cap, so its receipt was never finalized. Five superseded dev-fast census pins and the stale
  release library were retired before the next attempt.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/derive-privacy/scaling-raw.txt`), for 8/32/128
tuple structs in one module, each deriving `PartialEq` and compared once:

| Structs | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 8923 | 3836 | 163 |
| 32 | 24883 | 7888 | 451 |
| 128 | 88723 | 23524 | 1603 |

That is 665 inference steps, 163 canonical steps and 12 trait selections per
struct, linear across widths.

## Core progress

Candidate3's progress SHA-256 is
`42e338b777f7fd2463ee8ce4b2a966ef47e720e7b8d9a6db728dc6e47d69d9ff`, and its
comparison is `obj/parallel/capture-match/original-candidate3-comparison.txt`.
Its 31 private-field records:

- `fmt`:
  - `mem/maybe_dangling.rs:71`;
  - `error.rs:982`, `:990`, `:999`;
  - `array/mod.rs:182`, `char/convert.rs:317`;
  - `char/mod.rs:165`, `:231`, `:309`, `:380`, `:537`;
  - `ffi/c_str.rs:158`, `io/error.rs:655`, `net/parser.rs:486`;
  - `range/iter.rs:10`, `:171`, `hash/sip.rs:48`;
  - `str/iter.rs:283`, `:1161`, `:1226`;
  - `str/pattern.rs:811`, `:817`, `:873`;
  - `future/ready.rs:10`, `future/mod.rs:50`;
  - core_arch `x86/mod.rs:451`.
- `eq`: `char/convert.rs:317`, `ffi/c_str.rs:158`, `net/parser.rs:486`,
  `wtf8.rs:36` and `marker/variance.rs:107`.

No core observation of this change has run yet.
