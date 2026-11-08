# Transparent record headers

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/static-sync-20261108`. It sits on the shared-static checkpoint
`588fcd8bc`, which is based on pushed main `f98c5d7c8`. It is a transport
slice, not a library milestone, and makes no claim about an original-core
artifact or a whole assignment.

It is the second prerequisite of the caller-location chain. Core's `NonNull<T>`
is `#[repr(transparent)]`, and so are many other wrappers alloc consumes.
Before this slice, version 13 stopped export of every transparent struct with
"source nominal representation requires its wider complete contract".

## Wire rule (version 13)

The integrator approved this shape before it was used.

- **Representation byte.** The nominal header's representation byte, the last
  byte of the row, gains 2 = Transparent beside 0 = Rust and 1 = C. Only a
  version-13 reader accepts 2 (`record_wire`, under `body_nodes()`), and any
  other value stays rejected by name ("unknown source nominal
  representation").
- **Kind.** The exporter (`record_headers::record_repr`) writes 2 only for a
  struct with no packing, alignment or tag, and only under version 13. An enum
  header carrying 2 is rejected by the existing enum header check ("source
  enum nominal header carries record fields or representation").
- **Older writers.** Versions 10 and 12 keep main's rejection by name, both in
  the exporter and in the row encoder. `RecordRepr` gains `Transparent`. The
  older non-source record writer (`codec/nominals.rs`), which writes `repr(C)`
  as a bool, rejects it explicitly ("transparent record metadata is not
  implemented").
- **Consumer.** `mapped_records` interns the struct as
  `Representation::Transparent`. No layout or ABI fact travels. The
  consumer's own `complete_layouts` pass runs `abi::layout::validate_repr` and
  the `Templates::declaration` path for every imported definition. That path
  requires at most one nontrivial field over the consumer's own field
  layouts, so no extra check is needed. A forged transparent header over two
  `u8` fields is rejected there with "transparent aggregate has more than one
  nontrivial field".

The rule is in spec.md's version-13 transparent paragraph.

**Known integrity limit.** Rewriting byte 2 to byte 0 relabels a transparent
struct as `repr(Rust)` without detection. A consumer given `Wrap`'s header with
byte 0 accepts its client (`obj/parallel/transparent/flip-0.rgcrate`). The
representation is authenticated by the packet's inventory identity, not
re-proved. A later ABI digest may close the gap, and that is out of scope.

## Fixtures and outcomes

`pa29/tests/spec/transparent-records/` holds one provider and nine clients.
The outcomes were declared in `pa29/tests/transparent-record-outcomes.json` at
2026-11-08T05:02:15Z, before any rustc or RustGM run of these files.

The provider declares four `#[repr(transparent)]` structs:

- `Wrap(pub u32)`;
- `Tagged { v: u8, unit: () }`, whose second field is trivial;
- the generic `Cell<T>(pub T)`;
- `Ptr(pub *const u8)`.

Seven accepted clients exit 42. They build each struct with its constructor or
a record literal, pass it through the provider, and relay a `Wrap` through
`Cell<T>`. `outer` declares its own `#[repr(transparent)] struct
Outer(base::Wrap, ())`, which needs the imported `Wrap` to be its only
nontrivial field.

Two opponents are rejected as rustc rejects them:

- `mismatch` passes a `Wrap` where `u32` is expected (E0308): "incompatible
  types";
- `pair` declares `#[repr(transparent)] struct Pair(base::Wrap, base::Wrap)`
  (E0690): "transparent aggregate has more than one nontrivial field".

Three amendments were recorded before any RustGM run of what they changed.
The integrator accepted the first.

- **The provider.** The first rustc run stopped with an internal compiler
  error ("error performing operation: fully_perform", a delayed bug) on
  `wrap`, `wrap-call`, `cell-wrap` and `outer`, each of which matches a `u32`
  call result. The shared lines declare `Copy` only for `u8` and `bool`. A
  scratch provider with `Clone` and `Copy` for `u32` let all four exit 42, and
  the provider now ends with those two impls. No client or outcome changed.
- **The older-profile gate.** The packet's shared lines are already rejected
  under versions 10 and 12 by their alias transport, so they cannot witness
  the representation gate. The gate uses `spec/source-nominals/provider.rs`,
  as the earlier gates do.
- **The enum forgery.** An integration test can find a header's
  representation byte only through a provider pair whose bytes differ in
  exactly that byte, and an enum header has none. So the wire control
  declares its own `pub enum Mode { A, B }` directly after `Two`, whose header
  row it follows.

## Evidence

- **rustc 1.98.0.** `obj/parallel/transparent/oracle2.log` covers the
  committed bytes: all nine as declared. The first run is
  `obj/parallel/transparent/oracle1.log`, and the scratch provider's run is
  `obj/parallel/transparent/explore1/`.
- **Candidate.** All nine are as declared, and the seven accepted clients run
  natively and exit 42.
- **Generated controls**, in `crate_linking::newtype_records`:
  - the gate provider is rejected under versions 10 and 12 with "source
    nominal representation requires its wider complete contract". It exports
    under version 13, and its client is accepted.
  - `#[repr(transparent)] pub struct Two(pub u8, pub u8)` is rejected before
    export with "transparent aggregate has more than one nontrivial field".
- **Forgeries**, on the wire control (the shared lines, `Wrap`, a `#[repr(C)]`
  record `Two` of two `u8` fields, and `Mode`), whose honest client is
  accepted:
  - `Wrap`'s byte set to 3: "unknown source nominal representation";
  - `Mode`'s byte set to 2: "source enum nominal header carries record fields
    or representation";
  - `Two`'s byte set to 2: the fresh consumer's own layout pass rejects it
    with "transparent aggregate has more than one nontrivial field";
  - relabelled version 12: "unknown source declaration kind";
  - every strict prefix fails to decode.

  The test finds `Wrap`'s and `Two`'s bytes as the single byte at which a
  provider with `repr(C)` or no representation differs. That confirms that
  the representation is the only fact that travels.
- **Scaling.** Packets are 43,794, 62,442 and 137,034 bytes for 6, 24 and 96
  transparent tuple structs with one constructor function each. That is
  1,036 bytes per struct, exactly linear. The client LowIR is 1,155 bytes at
  every width.
- **Byte identity.** The baseline is the shared-static checkpoint pin (sha
  `dcc9556c`), and the candidate is sha `93651a06`. They were compared over
  every pa29 spec provider under versions 10, 12 and 13, each with every
  fresh consumer: 2,835 metadata, LowIR, status and diagnostic files. The 25
  files that differ are all in this packet's version-13 run. There the
  baseline rejects its provider with "source nominal representation requires
  its wider complete contract", and the candidate exports it. Versions 10 and
  12 are identical. Across the 28 provider directories, the baseline's 25
  version-13 exports and its 2 other rejections are unchanged. No earlier
  provider declares a transparent struct.

All rows ran on the final code, on main `f98c5d7c8`.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `newtype_records` (4) and 68 other non-native `crate_linking` tests | 72/72 | `1794116298950356004` |
| the remaining 65 `crate_linking` tests, `synced_statics` (2) and `crate_linking::exports` (8) | 75/75 | `1794116437382308941` |
| private record, role, codec and descriptor selection | 67/68 | `1794116556920662408` |
| native `--ignored`: this packet and the five earlier Lane C packets | 6/6 | `1794116859912855387` |
| PA1-PA12 | pass | `1794116962594152449` |
| PA13-PA19, PA21-PA24 | pass | `1794117077574421488` |
| PA20 | 1,451/1,451 | `1794117149697469572` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/transparent/test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/transparent/test-harness.log` |

The one private failure is main's stale `domain_budgets.rs:42` assertion
(`size_of::<SourceRows>()` is 792 against its 784), which the earlier
checkpoints record. This change adds no field to any retained row, and
`RecordRepr` stays one byte with its third variant. The identity receipt is
`obj/parallel/transparent/identity/receipt.txt`, and the candidate CLI is
pinned at `obj/parallel/transparent/candidate/`.

## Files outside transport

For owner review:

- `dev/src/support/crates/nominals.rs`: the `Transparent` variant;
- `dev/src/support/crates/codec/nominals.rs`: the older record writer's
  explicit rejection;
- `dev/src/sema/crates/imported_records.rs` and
  `dev/src/sema/crates/typed_headers/staged_headers/mapped_records.rs`: one
  representation arm each.

## Remaining gaps

- **The 2 to 0 flip** above.
- **`repr(transparent)` enums**, and transparent structs with packing or
  alignment (which rustc rejects anyway), stay rejected by name.
- **Next prerequisites** of the caller-location chain, in the integrator's
  order:
  - PhantomData's Pointee role;
  - NonNull's diagnostic role, with pattern types (tag 16) and
    `rustc_nonnull_optimization_guaranteed`;
  - auto traits with negative impls.

  Then come `Location`, the caller ABI and `PanicInfo`/`panic_impl`. Each
  exact shape is reported before it is used. The pattern-type constraint
  encoding is reported first.
