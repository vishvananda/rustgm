# Ordering and AtomicU32 load/store review

Canonical `AtomicU32::load` and `store` now execute through cached runtime-only intrinsic shims. Both methods use safe shared receivers; ordinary shared-reference assignment remains forbidden. `AtomicU32::new` also has a completed monomorphic callable template for runtime use. The template API preserves late-bound lifetimes and rejects abstract types, escaping or unbound lifetime slots, and duplicate publication.

`Ordering` has exact nominal/variant identities and a separate external-open coverage fact. Its five known variants alone cannot exhaust an external match; a wildcard covers the separate unknown coverage case. No unknown case is assigned a real variant identity or storage discriminant.

Rust tags and backend order encodings are distinct:

| Ordering | Rust tag | Load encoding | Store encoding |
| --- | ---: | ---: | ---: |
| Relaxed | 0 | 0 | 0 |
| Release | 1 | invalid | 3 |
| Acquire | 2 | 2 | invalid |
| AcqRel | 3 | invalid | invalid |
| SeqCst | 4 | 5 | 5 |

The default `invalid_atomic_ordering` denial consumes retained method-call syntax and direct canonical variant expressions. UFCS, aliases, variables and reified calls retain runtime ordering checks. The runtime switch translates tags through operation-specific typed LowIR orders and aborts invalid dynamic orders. Constant eligibility rejects these operations independently, including skipped branches and reified calls.

Whole-arm `unreachable_patterns` denial uses the existing usefulness facts and lexical allow/warn/deny/forbid policy. The default/warn policy remains non-failing under the current quiet diagnostics profile. Partial redundancy inside one alternative pattern remains outside this whole-arm slice.

## Qualification

- 34 independently declared source outcomes, plus the preserved valid-Rust assertion-macro boundary, match pinned Rust 1.98/edition 2024/x86-64. The boundary is an unresolved `assert!` macro, not an Ordering failure.
- Six general policy controls belong to active PA13; the closure policy control remains in planned PA24. All seven agree independently with pinned rustc.
- Fourteen native comparisons pass against the unchanged locked CPPGM backend. Six complete normally; eight check invalid runtime ordering.
- Native success means status 0 with empty stdout/stderr on both tools. For panic controls, rustc uses panic=abort and terminates by signal 6 with a panic diagnostic; the backend exits 134 with empty stdout/stderr. These outcomes are distinct from compile rejection.
- Scaling at 8, 32 and 128 generated call pairs preserves exactly two canonical intrinsic entries and two shared shims, each with three legal ordering branches; logical body work stays bounded.
- Strict all-target/all-feature Clippy, fourteen affected test binaries, Rust audit (1,271 files, zero findings), and pinned upstream checks pass.
- All 7,786 active PA1–23 text cases agree after integrating the coordinator's qualified opaque-obligation correction. The initial 76 PA21 and four PA23 differences are retained as pre-correction evidence.
- Assignment layout passes. No assignment is activated. Development wall/RSS measurement remains a separate integration gate.

Local evidence is under `obj/parallel/pa28/atomic-methods/`: independent qualification, final-gates.json, active-text/, corrected-text/, and pa13-generation-review.json. Native diagnostic files remain under `obj/parallel/pa28/atomic-method-native-*/`.

Exact independent panic reasons:

| Controls | Rust diagnostic text |
| --- | --- |
| `dynamic-release-load`, `ufcs-invalid`, `reified-invalid`, `allowed-invalid` | `there is no such thing as a release load` |
| `dynamic-acqrel-load` | `there is no such thing as an acquire-release load` |
| `dynamic-acquire-store`, `const-alias-invalid` | `there is no such thing as an acquire store` |
| `dynamic-acqrel-store` | `there is no such thing as an acquire-release store` |

## Six scoped PA13 references

The normal generator selected only the six new manifest paths. All 2,389 preexisting reference files stayed byte-identical, including the refreshed full `reference.json` with SHA-256 `d16483f65d97c1d21c9818835a48a5c0adad386660d60ba991e145c1634eba05`. The scoped receipt is `pa13/reference-batches/1792577511477098280-370b647b2fc7.json`; it attests only this selected batch.

`unreachable-default`, `unreachable-inner-allow` and `unreachable-inner-warn` each have status `0`, empty stderr, and exactly this shared stdout:

```text
selected-patterns 1
type 0 never
type 1 tuple []
type 2 primitive bool
type 3 primitive char
type 4 primitive str
type 5 primitive i8
type 6 primitive i16
type 7 primitive i32
type 8 primitive i64
type 9 primitive i128
type 10 primitive isize
type 11 primitive u8
type 12 primitive u16
type 13 primitive u32
type 14 primitive u64
type 15 primitive u128
type 16 primitive usize
type 17 primitive f32
type 18 primitive f64
type 19 function binders=0 parameters=[11] result=1 unsafe=false
type 20 function-item def=13 signature=19
root 0:9 pattern=0
root 0:12 pattern=1
pattern 0 source=0:9 input=11 wildcard
pattern 1 source=0:12 input=11 range=0
range 0 ranks=0:0
match 0 source=0:15 input=0:8 arms=0:2 exhaustive=true uninhabited=false work=6
arm 0 source=0:11 pattern=0 guard=- useful=true value=0:10
arm 1 source=0:14 pattern=1 guard=- useful=false value=0:13
eof roots=2 patterns=2 fields=0 alternatives=0 ranges=1 matches=1 arms=2 coverage-work=6 region-vars=0 region-equalities=0 conditions=0 chains=0
```

The other three cases each have status `1` and empty stdout. Their exact stderr is:

`unreachable-deny`:

```text
ERROR: file 0: 54: unreachable pattern
```

`unreachable-forbid-weaken`:

```text
ERROR: file 0: 37: forbidden unreachable pattern lint cannot be weakened
```

`unreachable-nested-item`:

```text
ERROR: file 0: 61: unreachable pattern
```

## Remaining contract

PA27 constant union interpretation, initialized-byte validity and provenance are still unsupported. Required initializer evaluation now rejects both demanded and unused invalid union fixtures through that boundary; this is not a scalar-validity proof. The original invalid fixtures are unchanged. Packed access and raw-pointer liveness controls are the next bounded qualification slice.

PA28 remains planned. This checkpoint does not add exchange, compare-exchange, fetch operations, fences, wider atomic families, numeric intrinsics or concurrency qualification.
