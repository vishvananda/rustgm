# Const inherent methods of lifetime-capturing impls

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/struct-constructors-20261108`. It sits on the task-1 checkpoint
`2964dc468`, which is based on pushed main `4a9a3441f`. It is a transport
slice, not a library milestone, and makes no claim about an original-core
artifact or a whole assignment.

It is the second task of the second Lane C batch. Core 1.98 declares const
methods in inherent impls that capture lifetimes, such as
`impl<'a> Arguments<'a> { pub const fn from_str(s: &'static str) -> Self }`,
and in impls that capture none, such as `NonZero::get`. Before this slice,
every const inherent method stopped export at "source nonordinary callable
effect/safety/convention requires a wider profile". Version 13 now carries
them with the same const checks and evaluation as a const free original. The
impl's captured regions are the leading early parameters, and no const
evidence is transported.

## Wire rule (version 13)

There is no new tag, column or row shape. The method's function header
carries the existing const constness byte (1). Its original carries the
existing const-definition context byte (1), which the exporter already wrote
for every const function. The rule is in spec.md's version-13 const
inherent-method paragraph. The integrator was told before commit and did not
object.

- **Admission.** `ordinary_profile::inherent_const` admits constness `Const`
  only on a method whose parent header is an implementation with no trait
  application, and only under version 13. `const unsafe` stays fenced by the
  unchanged safety clause. Early lifetimes stay fenced by the unchanged
  frame check ("source function early/captured callable frame transport is
  not implemented"). Trait-impl const methods stay fenced because their
  parent has a trait application.
- **Method families.** `conversion_profile::method` (no captures) and
  `frame_captures::method` (captures) now reject only conditional constness,
  so a const method joins the same family as an ordinary one.
- **Context re-proof.** `free_protocols::original_context` expects the
  const-definition context for every const header, free or inherent. It runs
  the same `constant_calls` check: every call names a const target of the
  same artifact.
- **Consumer.** `formal_headers` imports the header as const when its parent
  is an inherent impl whose captures equal the method's
  (`scheme_ports::inherent_captures`, already used for captured methods).
  `decoded_original` accepts the matching const-definition original. The
  consumer's own const checker then evaluates each call in a `const` item or
  a client const function, as it does an imported const free function.

## Temporary field reads

The fixture's `relay` returns `Self::new(text, 42).n`, a read of a field of a
call's temporary. Main rejects every such read at export with "source direct
temporary writes require wider proof", even for an ordinary free function
(`mp().n`). That is because the projection access table asked whether the
temporary was writable. Under version 13,
`evaluation_ports::projections` now records a field of a temporary as unique
but not writable. A write or mutable borrow through it is still rejected by
name ("source Store writes through an immutable dereference").

That write rejection is a **fenced deviation** from rustc, not a match.
rustc 1.98.0 compiles `pub fn w() -> u8 { mp().n = 5; 42 }` with no
diagnostic, and its client exits 42 (`obj/parallel/const-methods/oracle-temporary2/`).
Older profiles keep main's "source operation omitted a required nontrivial
formation input" for the read.

## Fixtures and outcomes

`pa29/tests/spec/const-methods/` holds one provider and fourteen clients. The
outcomes were declared in `pa29/tests/const-method-outcomes.json` at
2026-11-08T01:44:21Z, before any rustc or RustGM run of these files.

- Ten accepted clients exit 42. They cover runtime calls, `const` items, a
  client const function, the captured `Held<'a>`, the uncaptured `Plain` and
  the const free `make`, which calls a const method.
- Four opponents are rejected as rustc rejects them:
  - E0015, as "non-const calls are not allowed in constant initializers";
  - E0435, as "unresolved value name";
  - E0308, as "incompatible types";
  - `lifetime may not live long enough`, as "required lifetime does not
    outlive its target".

Amendments, each recorded with what was observed first:

- **`static-held` was withdrawn.** rustc rejects a static whose crate graph
  has no `sync` lang item. A client-local `Sync` is refused by RustGM, because
  a source client takes every lang item from its source core. A provider
  `Sync` cannot be exported, because it is an unsafe auto trait. The provider
  therefore keeps its declared bytes.
- **`text-leak` was made private.** Its declared `pub fn leak` exposed `&str`
  in the client's own interface ("str is outside the scalar metadata
  profile"). The private form reaches the declared lifetime error.
- **Older-profile controls.** Their declared messages were wrong. Main
  rejects the `Plain` provider under versions 10 and 12 with "source impl
  owner has no completed canonical conformance record", even with an
  ordinary `get`. The candidate gives the same bytes.
- **Forgeries.** Setting only `get`'s header to ordinary, or only `slow`'s to
  const, is rejected by the earlier context check ("source
  contextual/borrow/call original requires a wider explicit profile"). This
  follows integrator decision 3. Coherent rows were added:
  - `get` ordinary in both its header and its original's context: the client
    fails with "non-const calls are not allowed in constant initializers";
  - `slow` const in both: accepted, because the consumer re-checks `self.n`;
  - the same forgery over a `slow` that calls an ordinary `helper`: rejected
    with "source const-definition call requires its actual const target".

  Declared constness is therefore never evidence.

## Evidence

- **rustc 1.98.0.** `obj/parallel/const-methods/oracle-final.log` covers the
  committed bytes: all fourteen as declared.
- **Candidate.** All fourteen are as declared, and the ten accepted clients
  run natively and exit 42.
- **Generated controls.**
  - `Plain`'s const `get` exports under version 13, and a const client is
    accepted. Older profiles reject it by name, as main does.
  - `const unsafe` and early-lifetime const methods stay fenced by name.
  - The temporary read exports under version 13, and its client is accepted.
    Its write is fenced.
- **Scaling.** Packets are 43,801, 61,153 and 130,561 bytes for 6, 24 and 96
  const methods of one `impl<'a> Wide<'a>`. That is 964 bytes per method,
  exactly linear. The client LowIR is 351 bytes at every width.
- **Byte identity.** The baseline is the task-1 checkpoint pin (sha
  `22fe9718`, built from the same source as `2964dc468`). The candidate is
  sha `8424709d`. They were compared over every pa29 spec provider under
  versions 10, 12 and 13, each with every fresh consumer. That covers 3,621
  metadata, LowIR, status and diagnostic files, including all 25 earlier
  version-13 packets. The 36 files that differ are all in this packet's
  version-13 run: the baseline rejects its provider with "source nonordinary
  callable effect/safety/convention requires a wider profile", and the
  candidate exports it. Versions 10 and 12 are identical. No earlier packet
  changes, so no earlier fixture read a temporary's field or declared a const
  method.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `evaluated_methods` integration (4), `lifetime_methods` (4, with the superseded const gate now an accepted control), `struct_constructors` (4) and the task-1 checkpoint's neighbouring transport tests | 78/78 | `1794105609962241680` |
| native `--ignored`: this packet, struct constructors, captured methods, lifetime nominals, inherent methods, source nominals, constructors, enum match, str-never, unique writes | 10/10 | `1794105011915845838` |
| native `--ignored`: body nodes, cold, guards, branches, fn roles, spelled roles, direct calls, instantiable, normalized results | 9/9 | `1794105088247229327` |
| private selection (38): the task-1 selection plus seven const-original effect invariants (`rebuilt_operations::effect_tests`) and two supplied-column call-context tests | 37/38 | `1794105194356821017` |
| PA1-PA12 | pass | `1794105423255132264` |
| PA13-PA19, PA21-PA24 | pass | `1794105474528434903` |
| PA20 | 1,451/1,451 | `1794105548582368046` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/const-methods/test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/const-methods/test-harness.log` |

All rows ran on the final code. The one private failure is main's stale
`domain_budgets.rs:42` assertion (784 against main's actual 792), which the
task-1 checkpoint records. No record layout changed, and no field was added to
any retained row. `resolve` and expansion are untouched, so the macro-metadata
check does not apply.

The receipts are:

- the generated forgery observations: `obj/parallel/const-methods/forge-observe/`;
- the identity receipt: `obj/parallel/const-methods/identity/receipt.txt`;
- the pinned binaries: `obj/parallel/const-methods/candidate/` and
  `obj/parallel/struct-ctors/{baseline2,candidate2}/`.

## Remaining gaps

- **`Sync`.** It is an unsafe auto trait and does not export ("source
  const/auto/unsafe traits require a wider explicit profile"), so no
  imported or client static can be proved `Sync` against a source core. Main
  also accepts a source-client static when its source core declares no
  `sync` lang item, where rustc rejects it ("requires `sync` lang_item").
  That divergence predates this checkpoint and is an integrator decision.
- **Writes through a temporary's field.** These remain fenced, a recorded
  deviation from rustc.
- **Other const methods.** Const trait-impl methods (`const impl`), const
  methods with early lifetimes or lifetime bounds, and `const unsafe`
  methods stay fenced by name. A const method's calls must name const targets
  of its own artifact, as a free const original's must.

## Next task: blocked

Lane C's next task, region arguments of another provider's nominal in a
signature, is **blocked on multi-packet source export** (integrator decision).

- The exporter writes artifact 0 only (`contract_rows::address`,
  `scoped_parameters`), so no exported row names another artifact's nominal.
- `--source-profile` requires `--source-core`, and a source core that imports
  another version-13 packet fails at "imported Self declaration requires its
  authenticated source client", even with scalar signatures.
- Clients of core already accept core's lifetime nominals with region
  arguments: in their own signatures, as generic payloads (`Slot::new`,
  `Opt::Some`) and through `get(new_ref(&held))`.

std still needs alloc, a source client, to publish rows that name core's
artifacts, so multi-packet export is required later. A design proposal comes
first.

The two tasks after it are also **blocked**, each on prerequisite transport
slices (integrator decision):

- **The caller-location ABI.** Calls to `#[track_caller]` imports and
  `Location::caller()` need core's real `Location`. Local sema admits only
  `filename: NonNull<str>, line: u32, col: u32, _filename: PhantomData<&'a
  str>`; the stand-in's three-field form fails with "source panic nominal
  requires its actual four fields". Each part is fenced at export:
  - raw pointers ("source type form requires an explicitly supported
    transport row");
  - `#[repr(transparent)]` ("source nominal representation requires its wider
    complete contract");
  - `PhantomData` ("source Pointee contract transport is not implemented");
  - NonNull's diagnostic-item role.

  The prerequisites, in order, are raw-pointer and slice type rows,
  `repr(transparent)`, PhantomData's Pointee role and the NonNull role.
- **The `panic_impl` extern and the unsizing fence.** `panic_impl` fails with
  "panic callable requires completed actual Location", so it waits on the
  same chain plus `PanicInfo` and `Arguments`. `&[T]` has no type row, and
  unsizing `&[&str; 1]` needs `CoerceUnsized` in the stand-in.

