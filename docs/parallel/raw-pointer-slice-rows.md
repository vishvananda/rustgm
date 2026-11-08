# Raw-pointer and slice type rows

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/struct-constructors-20261108`. It sits on the task-2 checkpoint
`d9c73a69c`, which is based on pushed main `4a9a3441f`. It is a transport
slice, not a library milestone, and makes no claim about an original-core
artifact or a whole assignment.

It is the first prerequisite of the caller-location chain. Core's real
`Location<'a>` holds `filename: NonNull<str>`, and `NonNull<T>` holds a
`*const T`. `PanicInfo`, `Arguments` and every `&[T]` API need slices. Before
this slice, version 13 stopped export of every raw pointer and slice at
"source type form requires an explicitly supported transport row".

## Wire rule (version 13)

The integrator allocated two type recipe tags. They were reported before
commit.

- **Tag 12, `RawPointer`:** `[u8 12][u8 mutable][id pointee]`. It is the
  reference row with no region. `mutable` must be 0 or 1, and any other byte
  is rejected by name ("invalid source boolean"). The pointee may be unsized:
  `str`, a slice, or a `?Sized` parameter.
- **Tag 13, `Slice`:** `[u8 13][id element]`.
- **Reserved:** tag 14 for arrays and tag 15 for `dyn` trait objects. Neither
  is used.

Both tags are read only under `body_nodes()`. The exporter emits them only
under `body_nodes()` (`recipe_emission::type_row`). Older profiles keep
main's rejection by name and their exact bytes. The rule is in spec.md's
version-13 raw-pointer and slice paragraph.

- **Emission.** A pointer or slice row, like a reference row, is finished
  after its one child (`recipe_emission`, `completed_child`).
- **Validation.** `node_rows` follows the child and summarizes the row as the
  child. A slice whose element row is `str` or a slice is rejected by name
  ("source slice element requires a sized type"). The port plan gains
  `Node::RawPointer` and `Node::Slice`. A raw pointer is formed whenever its
  pointee is. A slice of a parameter needs a formation input, as a reference
  to a parameter does. Signature comparison follows the child and requires
  equal pointer mutability.
- **Consumer.** `demanded_types` interns `Kind::Pointer` and `Kind::Slice` in
  the consumer's own store. Before it builds `[T]`, it proves the element
  sized from its own structural facts (`Store::is_sized`), including a
  parameter's declared bound. No sizedness travels.

## Fixtures and outcomes

`pa29/tests/spec/pointer-slices/` holds one provider and eleven clients. The
outcomes were declared in `pa29/tests/pointer-slice-outcomes.json` at
2026-11-08T02:45Z, before any rustc or RustGM run of these files.

- Seven accepted clients exit 42. They pass `*const u8`, `*mut u8`,
  `*const str` and `*const T` pointers through the provider, build and read a
  record with a raw-pointer field, and relay `&[u8]` and `&[T]` slices,
  including one held by the lifetime-generic `View<'a>`.
- Four opponents are rejected as rustc rejects them:
  - a `*const u16` for `*const u8` (E0308), as "incompatible types";
  - a `*const u8` for `*mut u8` (E0308), as "raw pointer coercion cannot
    strengthen mutability";
  - a dereference outside `unsafe` (E0133), as "raw pointer dereference
    requires an unsafe context";
  - a `&[u16]` for `&[u8]` (E0308), as "incompatible types".

The shared lines have no `CoerceUnsized`, so no client builds a slice value.
The slice clients check signatures only.

One generated control was amended, recording what was observed first. The
declared older-profile gate said "a client of each is accepted". Its first
pointer client dereferenced the returned `*const u8`. It failed with "cannot
move a non-Copy value out of a reference", because `source-nominals` declares
`Copy` but no `impl Copy for u8`. The client now only passes the pointer
through. The declared rejections and exports are unchanged.

## Evidence

- **rustc 1.98.0.** `obj/parallel/pointer-slices/oracle1.log` covers the
  committed bytes: all eleven as declared.
- **Candidate.** All eleven are as declared, and the seven accepted clients
  run natively and exit 42.
- **Generated controls.**
  - The full provider is rejected under versions 10 and 12 with "named,
    associated or generic alias transport requires its wider profile".
  - `source-nominals` plus a raw-pointer function, or plus a slice function,
    is rejected under both older profiles with "source type form requires an
    explicitly supported transport row". Both export under version 13, and a
    client of each is accepted.
- **Forgeries**, on a minimal provider (the shared lines, `same`, `text_ptr`
  and `width`) whose honest client is accepted:
  - mutability byte 2: "invalid source boolean";
  - a pointee or an element id past the type table: "source type reference
    crosses its typed table namespace";
  - `width`'s slice element set to the `str` row: "source slice element
    requires a sized type";
  - relabelled version 12: "unknown source declaration kind";
  - every strict prefix fails to decode.
- **Consumer facts.** The integrator asked for this control after the shapes
  were reported, and its outcome was declared at 2026-11-08T03:03:42Z, before
  any run. A generated provider declares `pub fn pair<T, U: ?Sized>(_s: &[T],
  _p: *const U) -> u8 { 42 }`. The honest packet and a client relaying
  `pair::<u8, str>` are accepted. The forgery sets the `[T]` row's element to
  `U`'s parameter row. Wire validation sees only a parameter element and
  passes it. The fresh consumer rejects it from its own facts: "source slice
  element requires a sized type".
- **Scaling.** Packets are 46,088, 71,594 and 173,618 bytes for 6, 24 and 96
  pairs of a `*const u8` and a `&[u8]` pass-through. That is 1,417 bytes per
  pair, exactly linear. The client LowIR is 751 bytes at every width.
- **Byte identity.** The baseline is the task-2 checkpoint pin (sha
  `8424709d`, built from the same source as `d9c73a69c`). The candidate is
  sha `f43c79d8`. They were compared over every pa29 spec provider under
  versions 10, 12 and 13, each with every fresh consumer. That covers 3,730
  metadata, LowIR, status and diagnostic files. The 27 files that differ are
  all in this packet's version-13 run: the baseline rejects its provider with
  "source type form requires an explicitly supported transport row", and the
  candidate exports it. Versions 10 and 12 are identical. All 26 earlier
  version-13 provider exports, and the 12 other version-13 rejections, are
  unchanged. No earlier provider that exports carries a raw pointer or a
  slice.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `addressed_views` integration (5), `evaluated_methods` (4), `lifetime_methods` (4), `struct_constructors` (4) and the earlier checkpoints' neighbouring transport tests | 83/83 | `1794107333333716129` |
| native `--ignored`: this packet, const methods, struct constructors, captured methods, lifetime nominals, inherent methods, source nominals, constructors, enum match, str-never, unique writes | 11/11 | `1794107709958069766` |
| native `--ignored`: body nodes, cold, guards, branches, fn roles, spelled roles, direct calls, instantiable, normalized results | 9/9 | `1794107751491201561` |
| private selection (38), as at the task-2 checkpoint | 37/38 | `1794107791723763210` |
| PA1-PA12 | pass | `1794108206240332425` |
| PA13-PA19, PA21-PA24 | pass | `1794108266559342358` |
| PA20 | 1,451/1,451 | `1794108337342789042` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/pointer-slices/test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/pointer-slices/test-harness.log` |

All rows ran on the final code. The one private failure is main's stale
`domain_budgets.rs:42` assertion (784 against main's actual 792), which the
earlier checkpoints record. No retained record layout changed. The port
plan's node stays 20 bytes (`source planner node=20` in the private
selection's observations), and both new `s::Type` variants fit the existing
row. `resolve` and expansion are untouched, so the macro-metadata check does
not apply.

The receipts are:

- the rustc oracle: `obj/parallel/pointer-slices/oracle1.log`;
- the identity receipt: `obj/parallel/pointer-slices/identity/receipt.txt`;
- the pinned binaries: `obj/parallel/pointer-slices/candidate/` and the
  task-2 pin `obj/parallel/const-methods/candidate/` as the baseline.

## Remaining gaps

- **Slice values.** No client or provider can build a slice yet. Unsizing
  `&[T; N]` to `&[T]` needs arrays (tag 14) and `CoerceUnsized`, which the
  shared lines lack, and the export fence "reference unsizing requires its
  actual source CoerceUnsized declaration" is unchanged. Indexing, `len` and
  slice patterns are also not transported.
- **Raw-pointer operations in provider bodies.** Probes with the candidate
  pin (`obj/parallel/pointer-slices/ops-probe/`) found three fences, each by
  name:
  - `unsafe { *p }` is rejected with "source projection transport requires
    its wider explicit contract";
  - `p as *const u16` is rejected with "source cast operand requires a
    closed primitive type";
  - the `&u8` to `*const u8` coercion is rejected the same way.

  Moves of pointers and slices, and the reborrow `&*s` of a slice, export.
  Clients build and dereference pointers in their own bodies.
- **Next prerequisites** of the caller-location chain, in the integrator's
  order: `repr(transparent)`, PhantomData's Pointee contract role and
  NonNull's diagnostic-item role. Then come `Location`, the caller ABI and
  `PanicInfo`/`panic_impl`. Each exact shape is reported before it is used.
