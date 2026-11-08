# NonNull and pattern types

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/nonnull-patterns-20261108`. It was started on pushed main
`016b1f54d` and rebased onto `ad05f4d61` before its checks. It is a transport
slice, not a library milestone, and makes no claim about an original-core
artifact or a whole assignment.

It is the fourth prerequisite of the caller-location chain. Core declares

```rust
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}
```

and `Location` holds a `NonNull<str>`. Before this slice, a pattern-typed field
stopped export with "source type form requires an explicitly supported
transport row", and the diagnostic identity was silently dropped.

## Wire rule (version 13)

The integrator approved these shapes before code: (a) tag 254, (b) a
parameter-base fence, (c) a version fence for the identity and (d) fixed
u128 endpoints.

- **Type recipe tag 16**, version 13 only: `[16][u32 base][u8 kind]`.
  - Kind 0 is `!null`, with no operands, over a raw pointer recipe (tag 12).
  - Kind 1 is a closed scalar interval `[u128 start][u128 end]`, in raw
    target-width bits little-endian, over an integer or char recipe (tag 0).
  - Any other kind is "unknown source pattern constraint". Older readers
    reject tag 16 as "unknown source function type recipe".
  - In memory a range's endpoints sit in the cold `pattern_ranges` column, so
    a type row stays 24 bytes. The ranges are dense in type-row order, and the
    reader rebuilds the same references, so re-encoding is byte-exact.
- **Diagnostic identity tag 254** in version 13's role table:
  `[254][string spelling][address]`.
  - The spelling comes from a closed table in code,
    `schema_rows::DiagnosticRole`, which has one entry, `NonNull`. Each
    future spelling is a separate admission. An unknown spelling is "unknown
    source diagnostic identity spelling".
  - The rows follow every language row and lie outside the policy's role
    range, whose validation requires exactly `[0, roles)`. A language row
    after a diagnostic row is "source language role follows a diagnostic
    identity". Without a diagnostic row, the table's bytes are unchanged.
  - Older readers reject tag 254 as "unknown source function role ID".
- **`rustc_nonnull_optimization_guaranteed`** has no byte. It is
  documentary: the guarantee follows from the `!null` field, which every
  consumer re-derives.
- **No layout, niche or ABI fact travels.**

### Exporter

- `recipe_emission::pattern_row` emits a pattern row after its completed
  base, under version 13 only. Older profiles keep main's "source type form
  requires an explicitly supported transport row".
- (b) A `!null` pattern over a type parameter is "source non-null pattern over
  a type parameter is not transported". See the path record below.
- `authored_traits::diagnostic_roles` runs after every other export stage, so
  an export that fails earlier keeps its own diagnostic. Under version 13 it
  writes the role row. (c) Under versions 10 and 12 it rejects with "source
  NonNull diagnostic identity transport requires version 13"; main silently
  drops the identity. The identity run shows no existing version 10 or 12
  export flips from accepted to rejected.

### Reader and consumer

- `node_rows` (source validation) checks the structural form: a `!null`
  pattern's base row is a raw pointer, and a range's base row is an integer
  or char scalar with a range row of its own.
- `demanded_types::pattern` re-proves each pattern against the consumer's own
  base.
  - `!null` needs a raw pointer: "imported non-null pattern requires a raw
    pointer base".
  - A range in source order is interned through `intern_scalar_range`, the
    same checks as a local pattern. A start after its end in the base's
    source order is split into `MIN..=end` and `start..=MAX` and interned
    through `intern_scalar_or`, which only a signed base passes. An unsigned
    or char row in that order is "scalar alternatives require a signed
    integer base".
  - (d) Endpoint bits beyond the base width, or a surrogate char, are "scalar
    pattern endpoint is invalid for its actual base".
  - The store's interning work and growth are paid to the consumer's budget;
    a budget refusal keeps its own error.
- `ordinary_profile::diagnostic_bindings` requires the language profile, this
  provider's own declaration, no duplicate ("duplicate source diagnostic
  identity") and a struct with a nominal header ("imported NonNull diagnostic
  item requires its nominal declaration profile").
- `binding_views::diagnostic_roles` re-proves the local rule over the
  consumer's own remap: the declaration is a struct.
- The port plan's nodes carry the pattern, and signature matching compares
  range constraints by value.

## Recorded integrity limits

These join the later ABI-digest note.

- **Pattern-to-bare-base rewrite.** A packet whose pattern row is replaced by
  its bare base still decodes and validates. A consumer would then compute
  `Option<NonNull<T>>` at 16 bytes while the provider uses 8: a cross-crate
  layout disagreement. This is an ABI-class limit, the same class as the
  transparent byte flip, and is authenticated only by packet identity.
- **Const endpoints.** Endpoints travel as the provider's evaluated bits. The
  consumer re-proves that they are valid for its own base, but not that they
  equal the provider's const values. This is the same identity-authenticated
  class.

## (b) The parameter-base fence cannot fire on any export we produce

The only `!null` pattern over a type parameter in core is the impl header at
`pat.rs:85`:

```rust
impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is !null)> for pattern_type!(T is !null) {}
```

The path that would reach it is `implementation_headers::implementation`,
through `self.ty(schema.self_type)` into `recipe_emission`. Today no export
reaches it:

- an original-core v13 source export does not complete analysis: census 11
  ends at a budget record at `portable-simd/.../shift_scalar.rs:802`;
- even past analysis, `traits()` fails first on `unsafe_unpin`
  (`marker.rs:926`), before nominals or implementations;
- the other parameter-base sources are the pa31 local-only fixtures
  (`source-integer-niches/foundation.rs`, `partial-range.rs` and the others
  with core's `NonNull`), whose tests never export; no crate_linking provider
  contains `is !null` over a parameter.

When an export first reaches that impl, the answer becomes transport, as the
integrator required. pat.rs's other impls (`CoerceUnsized` at line 80,
`Unpin` at 87, `Freeze` at 89 and 91) are over `*const T` or `*mut T` bases,
which this slice transports. Each needs its own trait's transport in a later
item.

## Fences that item 5b starts from

- **Range helpers.** `authored_traits::require_role_transport`, from
  28cce9e7e: "source range helper contract transport is not implemented",
  whenever the catalog binds `RangeMin`, `RangeMax` or `RangeSub`. Core's
  open, exclusive and alternative ranges (for example `..0 | 1..`) need these
  roles.
- **Module const items.** `authored_export::kind`, from 603f3ebf5: "source
  declaration kind is outside the initial source profile", for every
  `DefKind` outside the admitted list, `Const` included. A const endpoint
  such as `HALF_USIZE` needs it.

## Selection: resolved upstream, no edit here

The first export of this slice's provider exhausted selection work ("canonical
trait selection work exhausted"). Moving a pattern value queries `Copy`, and a
pattern marker whose base lacked `Copy` relisted impl candidates until the
budget ran out. The reduction is `need::<pattern_type!(u32 is 0..=5)>()`
without `impl Copy for u32`. Lane A's `32e0d8259` (`bccd68182` after the
rebase) fixed it on main: refinement forwarding now completes with its own
decision instead of relisting. With the rebased CLI the reduction gives "trait
obligation requires an unambiguous matching assumption or implementation", the
same answer as `need::<u32>()`, and the provider exports. This slice changes no
selection code.

## Fixtures and outcomes

`pa29/tests/spec/nonnull-patterns/` holds one provider and ten clients. The
outcomes were declared in `pa29/tests/nonnull-pattern-outcomes.json` at
2026-11-08T09:40:34Z, before any rustc or RustGM run of these files.

The provider is the shared lines of the earlier packets, with
`#![feature(pattern_types)]`, plus:

- core's `pattern_type` builtin macro declaration and core's `NonNull`, each
  without its stability attribute;
- core's closed niche types, as `define_valid_range_type!` expands them,
  without their methods and impls: `Nanoseconds` (`u32 is 0..=999_999_999`),
  `NonZeroCharInner` (`char is '\u{1}'..='\u{10ffff}'`) and
  `CodePointInner` (`u32 is 0..=0x10ffff`);
- two authored closed ranges, `NonZeroU8` (`u8 is 1..=255`) and `Percent`
  (`i8 is 0..=100`).

Scratch probes on main (`obj/parallel/nonnull/explore/p1`) found the
provider's limits before the declaration. Core's open, exclusive and
alternative ranges need the range helper roles, and a const endpoint needs a
module `const` item; both are item 5b's fences above. Clients cannot build a
pattern-typed value (there is no transmute transport) or call `size_of`
(clients cannot declare intrinsics, and body-less intrinsics do not export).
So each client binds `Opt::None` of one imported type and matches on it, and
the layout witness is the LowIR slot. The provider did not export on main, so
those probes could not reach any client.

Four clients are accepted and exit 42. For each, the `entry` LowIR equals the
same entry compiled in one source-core crate with the provider's lines:

| Client | Local type | rustc size, align | RustGM |
| --- | --- | --- | --- |
| `nonnull` | `Opt<base::NonNull<u8>>` | 8, 8 | 8, 8 |
| `nonnull-str` | `Opt<base::NonNull<str>>` | 16, 8 | 16, 8 |
| `holder` | `Opt<H>`, `H(base::NonNull<u8>)` | 8, 8 | 16, 8 (divergence, below) |
| `nonzero-u8` | `Opt<base::NonZeroU8>` | 1, 1 | 1, 1 |

Four clients that rustc accepts are pending a local layout profile:
`nanoseconds`, `nonzero-char`, `code-point` and `percent`. The fresh consumer
and the single-crate control both reject each with "scalar pattern enum niches
require their complete ABI profile" (below).

Two opponents are rejected as rustc rejects them:

- `private` reads `NonNull`'s private field (E0616): "aggregate field is
  private";
- `mismatch` assigns an `Opt<NonNull<u8>>` to an `Opt<NonNull<u16>>` (E0308):
  "incompatible types".

The rustc sizes come from a std replica of the same declarations
(`obj/parallel/nonnull/oracle/sizes-std.rs`), because rustc 1.98.0 ICEs on a
`no_core` size observer of these types. The replica also shows that rustc
encodes `Opt<Percent>::None` as -1.

Each range type's own layout has a generated witness: `fn hold(_v: base::T)
-> i32 { 42 }`, whose address the entry stores in `let _f: fn(base::T) -> i32`,
so `hold` is lowered with `T`'s by-value slot. rustc accepts all five witnesses,
each exits 42, and a std replica gives 4/4 for `Nanoseconds`,
`NonZeroCharInner` and `CodePointInner` and 1/1 for `NonZeroU8` and `Percent`
(`obj/parallel/nonnull/oracle/witness/`). Every fresh consumer is accepted,
and each `hold` slot has rustc's size and alignment.

## Amended controls

Each amendment was approved by the integrator and recorded in the outcome file
before the run it governs.

- **Four pending range clients** (`nanoseconds`, `nonzero-char`,
  `code-point`, `percent`). They were declared accepted, as rustc accepts
  them. RustGM rejects each with "scalar pattern enum niches require their
  complete ABI profile", from `dev/src/abi/layout/niches.rs:316` (929b90e28,
  the pa12 niche profile). `integer_niches.rs` and `integer_source.rs`
  (partial-range) already pin that fence. Only `!null` pointers and
  zero-excluded unsigned ranges have nullable carriers. The same entry
  compiled in one source-core crate gives the same rejection, so this is a
  local layout gap, not a transport gap. The test asserts that the fresh
  consumer and the single-crate control reject alike. The fixtures and their
  rustc outcomes are unchanged.
- **holder's slot.** rustc's `Opt<H>` is 8x8. RustGM's is the tagged 16x8, in
  both the fresh consumer and the single-crate control, whose entry LowIR is
  equal. The nullable-carrier walk (`Niches::carrier`, the `Kind::Adt` arm)
  descends only `repr(transparent)` structs: rustc's guaranteed null-pointer
  optimization. rustc also places the niche through a non-transparent
  struct's field, which it does not guarantee. The outcome file keeps rustc's
  8x8 and records 16x8 as a separate divergence field. The test pins 16x8 with
  this reason, so the pin fails when field-niche placement lands and then
  flips to the declared 8x8. The LowIR equality and native exit 42 stay as
  declared.
- **Layout witness, single-crate control.** It was declared as LowIR equality
  with the witness compiled in one source-core crate. That crate cannot be
  built: version-13 module-body export has no row for fn-item-to-fn-pointer
  reification, so it rejects with "source rvalue requires a complete wider
  transport row" (`lower/transported_source/cfg_rows/value_rows.rs:92`).
  Exploration probes found no other single-crate form that lowers a by-value
  `hold`:
  - a by-value `extern "C"` forwarder: "C aggregate ABI profile requires a
    nonempty aligned repr(C) struct";
  - a Rust-ABI `no_mangle` forwarder: "exact callable requires its actual
    fixed primitive C signature";
  - a client struct declared through `base::pattern_type!`: "imported builtin
    macro requires its provider's source declaration";
  - a diverging call: `hold` is never lowered.

  The control now asserts that rejection, a pending body-transport gap. The
  equality claim waits for it, and two cross-consumer equalities stand in
  meanwhile: the `Nanoseconds` and `CodePointInner` witnesses (one u32 base)
  have identical LowIR, and so do the wrapped-Percent and honest Percent
  witnesses. One feasibility probe of the witness form, for `Nanoseconds`, ran
  before its declaration (`obj/parallel/nonnull/explore/c2/c-fp.rs`).
- **The wrapped-Percent wire control.** It is "a well-formed rewritten row,
  not an honest export": Percent rewritten to `1..=0xff`, the bits of
  `i8 is ..0 | 1..`. Its `Opt<Percent>` client reaches the niche fence before
  the interval matters, so its client is now the Percent witness. The
  rewritten packet is accepted, and its witness LowIR equals the honest
  packet's. The lib invariant
  `wrapped_rows_intern_the_local_connected_alternative` still shows that the
  consumer interns exactly the interval a local `..0 | 1..` interns. Its
  honest witness arrives with RangePattern role transport (item 5b).
- **The scaling client.** `Opt<base::R005>` reaches the same niche fence, so
  the fresh client is the `R005` witness. Packet bytes still grow exactly
  linearly, and the client's LowIR stays within 64 bytes.
- **Cold interning work.** The lib invariant `imported_interval_work_grows_linearly`
  was declared as exactly linear consumer work. Re-proving a warm interval
  costs exactly 14 steps at each width. A cold interval also pays the store's
  table growth, which copies at geometric thresholds: 127, 576 and 2,348 steps
  at 6, 24 and 96. That is linear within a bound: each increase is at most
  four times the previous one, and each interval costs at most 25 steps. The
  test asserts the measured shape.

Two harness fixes need no amendment. The provider exports its `pattern_type`
macro, so its packet is a declarative macro envelope (artifact tag 14,
version 2)
around the function artifact. The version check reads the inner artifact, and
the duplicate and deleted diagnostic forgeries rewrite the envelope's inner
length. Separately, `o` is the entry's first slot, `$s0`.

## Evidence

Every run used the tree rebased onto `ad05f4d61`. Receipts are under
`obj/qualification/fast/`, and each passed. The candidate CLI is
`obj/parallel/nonnull/candidate/rustgm` (sha256 `c2cdd746`), the fastcheck
build of this tree. The test-only edits after it do not change the CLI. The
baseline is the same build of `ad05f4d61` (`obj/parallel/nonnull/baseline/rustgm`,
sha256 `355d3f77`).

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| this slice's lib and crate_linking cases | 8 + 7 | `1794136470630979690` |
| supplied-packet codec cases | 7 | `1794136523618733378` |
| linking suite, part 1 | 47 | `1794136762954690263` |
| linking suite, part 2 | 47 | `1794136850774291772` |
| linking suite, part 3 | 47 | `1794137093646893617` |
| linking suite, part 4 | 47 | `1794137162695876226` |
| linking suite, part 5 | 46 | `1794137297014468054` |
| conversion_inference downstream | 1 | `1794137433949498919` |
| lib cases of the touched modules | 82 | `1794137539631609147` |
| private source-client cases, part 1 | 85 | `1794137574099067124` |
| private source-client cases, part 2 | 84 | `1794137611585825772` |
| course text pa1-pa12 | passed | `1794137853984769048` |
| course text pa13-pa19, pa21-pa24 | passed | `1794137920007627716` |
| course text pa20 | passed | `1794137998034244013` |

The five linking parts are the suite inventory's 234 non-ignored tests:
160 crate_linking tests, `nonnull_patterns` included, and the suite's
`metadata_codec`, `compiler_recipes` and other modules. The 82 lib cases are
every test under `compiler_items::binding_views`, `crates::authored_export`,
`typed_headers::recipe_cursor`, `codec::ordinary_columns` and
`crates::generics`. `make test-upstream` verified 128 pinned fixtures and 126
Reference files. `make test-harness` ran 304 script tests, found 464 roots in
13 binaries and no orphan fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| linking suite native, part 1 | 24 | `1794137647467240584` |
| linking suite native, part 2 | 24 | `1794137707007431750` |
| conversion_inference native | 1 | `1794137783997938378` |
| packet-envelope native client | 1 | `1794137818104715065` |

The two linking parts are the inventory's 48 ignored tests, the four accepted
clients' `pattern_type_clients_execute_natively` included. With the
non-native parts, that is all 282.

(c) holds: no version 10 or 12 export flips. The identity run
(`obj/parallel/nonnull/identity/`) compiled every pa29 spec provider with both
CLIs under versions 10, 12 and 13, then every fixture client against each
export: 30 packets and 1,002 runs. Only the `nonnull-patterns` packet under
version 13 differs:

- the provider exports, where the baseline stops at "source type form requires
  an explicitly supported transport row";
- `nonnull`, `nonnull-str`, `holder` and `nonzero-u8` now exit 0;
- the four pending range clients and the two opponents are rejected under
  their declared messages instead of the baseline's missing artifact.

Every other run's status, stderr and LowIR are byte-identical. Versions 10 and
12 accept 6 and 14 runs both before and after. Version 13 accepts 173 before
and 178 after.

The selection reduction was re-run with both CLIs:
`need::<pattern_type!(u32 is 0..=5)>()` without `Copy` for `u32` gives "trait
obligation requires an unambiguous matching assumption or implementation" with
the baseline and the candidate, and is accepted with the impl.

Oracles, run before the candidate:

- `obj/parallel/nonnull/oracle/oracle.log`: rustc 1.98.0 gives the ten declared
  outcomes, and the eight accepted clients exit 42 natively;
- `obj/parallel/nonnull/oracle/sizes-std.out`: the std replica's sizes;
- `obj/parallel/nonnull/oracle/witness/oracle.log`: rustc accepts the five
  layout witnesses, each exits 42, and the bare sizes are 4/4, 4/4, 4/4, 1/1
  and 1/1.

Scaling, all asserted by the tests:

- `pattern_packets_and_fresh_consumers_grow_linearly` uses 6, 24 and 96
  pattern records. It asserts exactly linear packet growth, and the R005
  witness client's LowIR stays within 64 bytes.
- `imported_interval_work_grows_linearly` asserts the interval re-proof's work
  at 6, 24 and 96 intervals: exactly 14 steps per warm interval, and the
  bounded-linear cold shape recorded above.

## Files outside transport

For owner review:

- `dev/src/sema/compiler_items/binding_views.rs`: `diagnostic_roles`,
  called first in `reserve_roles`;
- `dev/src/support/crates/generics/ordinary_profile.rs`: `diagnostic_bindings`,
  called from `bindings`;
- `dev/src/sema/crates/typed_headers/demanded_types.rs`: `Worklist::pattern`,
  the free `scalar_range`, the `Pattern` child in `next`, and the test mount
  of `dev/tests/invariants/wrapped_intervals.rs`;
- `dev/src/support/crates/generics/schema_rows.rs`: the `Pattern` type row,
  `PatternConstraint`, `PatternRange`, `DiagnosticRole` and
  `DiagnosticBinding`. `ColdColumns` gains the `pattern_ranges` and
  `diagnostics` columns. No test asserts its size, and the `size_of` hits
  under `dev/tests/invariants` are other types.

## Remaining gaps

- **Local layout: niches.** Frontend layout work, with an owner assigned when
  bodies reach it.
  - Scalar-range niches other than zero-excluded unsigned, such as `Option` of
    the signed `NonZero` types and the four pending clients.
  - Niche placement through non-transparent struct fields, which rustc uses
    but does not guarantee (holder).

  Where a runtime size is compared, `size_of` differs from rustc's without any
  diagnostic, and core's const size asserts would fail loudly at const-eval
  once bodies are entered. A read-only grep of the pinned core found no
  const or static size assertion over `Option`, or another niche-using enum,
  of a non-transparent struct with a niche field. The only `Option` and
  `Result` size asserts are `io/error/repr_bitpacked.rs:348-351` over `Repr`,
  which is `repr(transparent)` over `NonNull<()>` and inside the transparent
  walk. `mem/alignment.rs:25-26` assert `Alignment`'s size and alignment
  against `usize`, with no `Option`.
- **Fn-item-to-fn-pointer reification in exported bodies.** Version-13
  module-body export rejects it ("source rvalue requires a complete wider
  transport row"). Core's bodies reify function items in many places
  (fn-pointer fields and callbacks). Queued for Lane C after item 5b.
- **Range helper roles and module const items**: item 5b, from the two
  fences above.
- **`!null` over a type parameter**, fenced at export (b), until an export
  reaches `pat.rs:85`.
- **Next prerequisites** of the caller-location chain, in the integrator's
  order: auto traits with negative impls; item 5b; `Location` and the caller
  ABI; `PanicInfo`/`panic_impl`; arrays (tag 14) with `CoerceUnsized`; the
  multi-packet proposal.
