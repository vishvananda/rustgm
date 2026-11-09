# SIMD header rows (B2 S)

Lane D's slice S, 2026-11-09, on `implement/simd-records-20261109`, stacked
on T (`008593ab5`). The integrator approved the S notice, which puts S after
T. S carries `#[repr(simd)]` structs through version-13 module-body
transport and makes the `xmm_reg` re-proof of an imported asm body reachable.
It is a transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

## Before

No vector type row exists. `__m128`'s type is an ordinary nominal type row,
and its lanes are its one field's type, `[f32; 4]`, which is T's tag-14 row.
Before S, version 13 rejected every SIMD struct at export with "source nominal
representation requires its wider complete contract". That fence also stopped
the stream witness, the crate that holds the real `_mm_stream_ps` body.

## Row

- **Representation byte 3, `Simd`.** It is the nominal header's last byte,
  beside 0 = Rust, 1 = C and 2 = Transparent. It is read only under
  `body_nodes()`. Bytes from 4 up stay "unknown source nominal
  representation", as does byte 3 in an older profile.
- `support::crates::RecordRepr` gains `Simd`. It stays one byte, and
  `s::NominalHeader` keeps its 96 bytes (`vector_headers.rs`).
- No value code, type tag, lane, count or layout column is added, and the
  version is not bumped. A header written by the pre-v13 encoder with `Simd`
  fails with "source nominal representation requires its wider complete
  contract", as `Transparent` does.

## Exporter

`record_headers::record_repr` writes `Simd` for exactly `#[repr(simd)]` with
no packing, alignment or integer tag on a struct, and only under
`body_nodes()`. Everything else falls through to the existing fence:

- `#[repr(simd, packed)]` and `#[repr(simd, align(32))]` keep "source nominal
  representation requires its wider complete contract". The writer never
  writes byte 3 with a modifier dropped (S2).
- A SIMD struct with any type, const or lifetime parameter fails with "source
  generic SIMD record requires its wider complete contract"
  (`concrete_simd`). It is stricter than rustc for now (S3), so no consumer
  sees an imported generic SIMD definition.
- `rustc_simd_monomorphize_lane_limit` keeps "SIMD lane limits require
  authenticated representation metadata transport", which is checked before
  any header is written.
- `#[repr(simd, C)]` keeps its local "conflicting representation hints".

## Consumer

- `mapped_records` maps byte 3 to the consumer's own
  `aggregate::Representation::Simd`. The legacy record import does the same
  for exhaustiveness, but the legacy codec cannot carry it.
- Each consumer re-validates from its own facts. `complete_layouts` runs
  `validate_simd_declaration` and `validate_repr` over every aggregate,
  imports included, and `simd_shape` and `simd_layout` run on demand.
- An imported asm body's `xmm_reg` operand is re-proved by the consumer's own
  `simd_operand`:
  - 128 bits of supported lanes, then the SSE baseline;
  - "assembly SIMD operand requires its completed genuine representation"
    when the operand's header is not SIMD.
- The legacy codec refuses a SIMD record with "SIMD record metadata is not
  implemented", the legacy exporter's own text. Without that guard, its
  `u8::from(repr == C)` would write `Simd` as Rust.

## Known integrity limit (S4)

Rewriting byte 3 to byte 0 changes ABI-visible facts: alignment 16 versus 4,
and vector-register passing at C boundaries.

- **With asm.** The consumer's re-proof rejects the rewrite by name, as the
  `rust` forgery below shows.
- **Without asm.** It is a silent runtime mismatch. Like the transparent
  rewrite (byte 2 to 0), it is authenticated only by the packet's inventory
  identity and is left for a later ABI digest. F's forged-bit-3 limit,
  which widens one body's asm CPU policy and is not ABI-changing, joins
  these two when F lands.

## Fixture and outcomes

[`pa41/tests/spec/simd-transport`](../../pa41/tests/spec/simd-transport/README.md)
holds one provider and two clients. The provider is the pinned `native.rs`
foundation through `__m128`'s impls, then `V(pub [u32; 4])` and
`put(p: *mut V, a: V)` with `movups [{p}],{a}`.

Every outcome was declared in `pa41/tests/simd-transport-outcomes.json` at
2026-11-09T10:54:35Z, before the rustc oracle and before any S code. The
pre-S "before" texts came from the S baseline `ca3ba92f`, which is T's
candidate rebuilt from `008593ab5` with identical bytes.

| Control | Result |
| --- | --- |
| provider | exports under `module-bodies-v13` in Lane B's `[14, 2]` macro envelope around the `[13, 0]` core, as A's does. The nominal table holds exactly `__m128` and `V`, both byte 3. `[f32; 4]` and `[u32; 4]` are value-term rows of length 4 on one usize row. Each impl's `[T; N]` is a slot-term row (two). |
| `store` | accepted. One template, `text "movups ["`, `use 0`, `text "],"`, `use 1`, over a `v128` operand. Exits 42 natively with empty stdout, as rustc's does. |
| `mistyped` | "incompatible types" (rustc E0308) |
| `module-branches-v12`, `module-nominals-v10` | the provider stops first at "source declaration kind is outside the initial source profile". `source-nominals` plus a SIMD struct gives "source nominal representation requires its wider complete contract" in both, and exports under v13. |
| packed, align | "source nominal representation requires its wider complete contract" (rustc accepts) |
| generic `G<T>`, `N<const L: usize>` | "source generic SIMD record requires its wider complete contract" (rustc accepts) |
| lane limit, `simd, C` | their existing texts (rustc accepts the first and rejects the second with E0566) |
| forged `V` byte 0 | "assembly SIMD operand requires its completed genuine representation" in the fresh `store` consumer |
| forged `V` byte 4 | "unknown source nominal representation" |
| pa29 wire control, amended (S1) | `Wrap` byte 4: "unknown source nominal representation". `Wrap` byte 3: "SIMD representation requires an array field", from the consumer's `complete_layouts`. `Mode` byte 3: "source enum nominal header carries record fields or representation". |
| codec rows | byte 3 reads as `Simd` under `body_nodes()` only, and 4, 5 and 255 are unknown. Bytes 0-2 keep their bytes and gates. |
| legacy codec | `Transparent` and `Simd` records are refused by name |
| old readers | the S baseline `ca3ba92f` rejects S's packet with "unknown source nominal representation". The fence pin `3960ec90` rejects it with "unknown source function type recipe". |
| byte identity | the S base `ca3ba92f` and the candidate `868d91e1` publish identical packets and LowIR for transparent-records, panic-attributes, macro-contexts, const-parameters, pointer-slices, named-constants, array-types, pa41 asm-transport and the native.rs prelude |
| flip survey, base against candidate | 500 runs per binary over 67 spec directories (54 exports, 446 clients). The only 3 differences are this packet: the provider and `store` flip to accepted, and `mistyped` gives its declared text. |
| stream witness | the base: "source nominal representation requires its wider complete contract". The candidate: "source named/foreign/target-feature function metadata requires its wider profile", from `_mm_stream_ps`'s `#[target_feature]`, as declared. |

- **Oracle and the client amendment.** rustc 1.98.0's first run
  (`obj/parallel/simd-transport/rustc`) rejected the declared `store` client
  with "requires `freeze` lang_item". The pinned foundation declares no
  `freeze` item, and the client took `&raw mut` of a local `V`, which asks
  rustc whether `V` is Freeze.
  - The amendment, recorded at 10:58:02Z before any development or candidate
    run, stores into a `[0u32, 0, 0, 0]` local addressed as `*mut base::V`.
    `movups` needs no 16-byte-aligned destination.
  - The rerun gives every declared outcome. The provider object holds
    `movups %xmm0,(%rdi)`.
- **movups.** The pinned encoder's `0F 11` store form ran natively for the
  first time here. It assembled and the client exited 42, so the
  failed-prediction rule (integrator ruling e) did not apply. movntps on `V`
  remains the later alignment control, once `V`'s 16-byte alignment is shown
  natively.
- **Locating `V`.** An integration test cannot reach decoded source rows, so
  the forgeries patch `V`'s representation byte in the honest packet. The
  byte is fixed-width, so this is the packet that re-encoding the rows would
  write. The test finds the nominal table by its count, 2, and two adjacent
  43-byte rows ending in byte 3 and owning fields 0 and 1. The looser
  single-row heuristic matched two zero-filled rows elsewhere in this packet.
- **Focused checks.**
  - `make fastcheck` passed 3 private tests and 9 linking tests
    (`obj/qualification/fast/1794222726787398300`): S's codec unit, export,
    control, forgery and legacy tests, pa29's amended wire control, the
    stream witness, B1's local stream body, and A's and T's packet tests as
    controls.
  - The native root passed 2: S's `store` and A's clients
    (`obj/qualification/fast/1794222940245591129`).
  - `make test-upstream` verified 128 pinned fixtures and 126 Reference
    files.
- **Receipts.** `obj/parallel/simd-transport/receipts`. The pins are
  `obj/parallel/simd-transport/base` (`ca3ba92f`, from `008593ab5`) and
  `candidate` (`868d91e1`).

## Remaining

- **F**, target-feature rows, is next. The stream witness now stops at
  `_mm_stream_ps`'s `#[target_feature]`.
- **S3**: generic SIMD records, with a lane proof at each consumer's
  instantiation (`docs/library-build-blockers.md`).
