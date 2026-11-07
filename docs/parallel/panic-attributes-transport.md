# Cold, caller-tracking and `panic` role rows

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/core-panic-path-20261107`, based on main `687f746fd`. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

It is the first step of task 1, core's real panic path. Core 1.98's
`panicking::panic` carries `#[cold]`, `#[inline(never)]`, `#[track_caller]`,
`#[rustc_const_stable_indirect]` and `#[lang = "panic"]`. Version 13 now
carries the first, third and last of these. `#[inline(never)]` and `const fn`
already crossed. A fresh consumer runs a provider with core's `panic`
attributes and role natively, except `#[track_caller]` on `panic` itself (see
the gaps).

## Header flags (version 13)

There is no new column. The byte that marked an inline contract in a header
contract row becomes a flag set:

| Bit | Meaning |
| --- | --- |
| 0 | inline contract follows |
| 1 | written `#[cold]` |
| 2 | written `#[track_caller]` |

A row without the two advisories keeps its exact earlier bytes. The rule is
in spec.md's version-13 advisory paragraph.

- **Export.** `advisory_rows.rs` sets both flags from the completed header
  (`cold_advisory`, `checked_caller_tracking`) under `body_nodes()` only, and
  charges two steps per function. Older stages keep "source cold advisory
  marker transport is not implemented" and "source caller tracking policy
  transport is not implemented". Under version 13 a tracked trait member is
  rejected with "source trait member caller tracking requires its trait
  implementation transport". Free functions and inherent methods are admitted.
- **Codec.** `annotation_rows.rs` rejects bits above 2 ("unknown transported
  inline or callable advisory flags") and rejects bits 1 and 2 under older
  profiles by name.
- **Validation.** `advisory_contracts.rs` admits the advisories only on
  version-13 function rows. Caller tracking also needs an exported original
  body. Trait and nominal rows reject both.
- **Consumer.** `metadata_contracts::caller_tracking` reads the flag from the
  row and re-proves it against the consumer's own installed header. The
  header must be a Rust-ABI function, free or owned by an imported inherent
  impl. `recorded_caller_tracking` and `checked_caller_tracking` return this
  result for imported definitions. Cold stays a header fact with no
  behavioral effect.

A tracked call passes its caller's `Location`, and the caller location ABI
does not transport yet. So a consumer that calls a tracked imported function
is rejected with "imported caller tracking requires its provider's
transported Location" (`site_locations.rs`). A rebuilt provider body that
calls one is rejected by the consumer's own lowering ("tracked call lacks its
actual sealed caller site"). No honest provider exports such a body, because
local lowering of a call to a tracked function also needs `Location`.

## The `panic` role (version 13)

The role row is the existing spelled-role row (escape 255 and the spelling
`panic`) with the declaration address.

- **Local completion.** `runtime_panics::complete` runs after the value
  headers. It completes the role only for a free, defined, const, safe
  Rust-ABI `fn(&'static str) -> !` with no generics. Any other form stays
  pending, so local checking is unchanged and export keeps rejecting it with
  the earlier "source trait role requires completed declaration validation".
- **Export.** The role loop admits a `panic` function only under version 13.
  Older profiles reject it with "source panic role requires the
  version-thirteen role profile". Provider validation admits a `panic` row
  naming a function header under the language-binding profile.
- **Consumer.** `admitted_headers` re-proves the role with
  `runtime_panics::imported`. `Graph::declared_panic_role` checks that the row
  names this declaration of this provider and that its header is installed,
  paying one step per role row. The function must be free, and its installed
  function item and signature must pass the same `signature` check as the
  local completion. The provider's completion is never trusted.

`panic_fmt` and `format_arguments` were part of the plan and are not in this
checkpoint. A nominal with a lifetime parameter does not export: every
provider declaring `pub struct Arguments<'a> { .. }`, with or without the
role, stops at "source lifetime binding has no actual original slot"
(`scoped_parameters::region_slots` reserves region slots only for function
schemes). Core's `Arguments<'a>`, `Location<'a>` and `PanicInfo<'a>` all need
this.

## Forgeries

`forged_advisory_and_role_rows_reject_in_fresh_consumers` uses a generated
minimal provider: lines 1-126, `panic` with only its role, `#[cold] rare`,
`relay` calling `rare`, and `#[track_caller] traced`.

| Mutation | Fresh consumer |
| --- | --- |
| `rare`'s flag byte set to 8 | "unknown transported inline or callable advisory flags" |
| `rare` marked cold and tracked, `relay` client | "tracked call lacks its actual sealed caller site" |
| the same packet, `rare` client | "imported caller tracking requires its provider's transported Location" |
| the `panic` row retargeted to `rare` | "source panic role requires a const declaration with never result" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |
| `traced`'s tracking flag cleared | accepted (sound) |

The cleared flag is sound because the consumer re-lowers `traced` itself, and
no transported body can read the location it drops.

**Rework, recorded in the outcomes file.** The second mutation was declared
with a new reconstruction check, "transported call to a tracked function
requires its caller-location rows". The packet was rejected on the first
candidate run, but by the lowering guard above, so that check was never
reached. The unreached check was removed and the observed diagnostic
recorded.

## Evidence

The fixtures are in `pa29/tests/spec/panic-attributes/`, and their outcomes,
declared before the oracle and the candidate ran, are in
`pa29/tests/panic-attribute-outcomes.json`. The integration tests are in
`crate_linking::cold_transport`.

- **Oracle.** The first rustc 1.98.0 round rejected the provider (E0428,
  E0152): the authored section repeated the oracle-only `panic_location` shim
  that lines 1-126 already carry. The repeat was removed before any candidate
  run, and no outcome changed. In the second round the provider compiles,
  check, role, rare, traced and method exit 42, and the opponents fail with
  E0308 and E0061.
- **Native.** The three accepted clients exit 42 natively, as rustc's do:
  check, role and rare.
- **Fenced.** rustc accepts traced and method. RustGM rejects both by name, as
  declared.
- **Opponents.** Both are rejected, as rustc rejects them:
  - panic-type: E0308 / "incompatible types";
  - rare-arity: E0061 / "call argument count mismatch".
- **Older profiles.** Under versions 12 and 10, cold, caller tracking and a
  conforming `panic` role on the source-nominals provider are rejected by
  name, and all three export under version 13. A nonconforming role stays
  pending. The full provider stops at the shared inherent-methods gate.
- **Scaling.** Packets are 33,558, 45,150 and 91,518 bytes for 6, 24 and 96
  cold, tracked functions. That is 644 bytes per function, exactly linear.
  The client LowIR is 624 bytes at every width.
- **Byte identity.** The baseline is the pinned static-str-never candidate
  (sha `eaf69733`, main's code). The candidate is sha `89ece36e`. They were
  compared over the v10, v12 and v13 fixtures, the static-str-never v13
  provider and this packet's older-profile exports. Across 667 metadata,
  LowIR, status and diagnostic files, none differ.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `cold_transport` integration (4) | 4/4 | `1794081963643460777` |
| native `--ignored` (this packet, static-str-never) | 2/2 | `1794082244009705019` |
| native `--ignored` (this packet, static-str-never, spelled roles, fn roles), before the last refactor | 4/4 | `1794081172003656602` |
| neighbouring v13 suites (guard, inherent, constructor, match, node, branch, fn roles, statics, str-never, spelled roles) | 36/36 | `1794081221810881179` |
| private selection (16), including the hot-record invariant | 16/16 | `1794081739649775294` |
| PA1-PA12 | pass | `1794082363383428175` |
| PA13-PA19, PA21-PA24 | pass | `1794082398911019580` |
| PA20 | 1,451/1,451 | `1794082288169727487` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/core-panic-path/test-upstream.log` |

The last refactor kept the two-argument older-profile advisory boundary that
private invariants call. Everything after it was rerun except the 36
neighbouring tests and two of the four native tests.

The rustc 1.98.0 receipts are `obj/parallel/core-panic-path/rustc/receipt.txt`
and `receipt-round1.txt`. The identity receipt is
`obj/parallel/core-panic-path/identity/receipt.txt`. The pinned binaries are
`obj/parallel/core-panic-path/{baseline,candidate}/`.

## Amended controls

None. No earlier outcome changed.

## Edits outside crate transport, for owner review

- `sema/compiler_items/runtime_panics.rs`: `signature` now takes the
  definition, its signature type and the `format_arguments` identity instead
  of a local `Function`. It gains the `panic` branch, the local completion and
  the consumer re-proof. `runtime_body` checks the same conditions as before.
- `sema/compiler_items/roles.rs`: `complete_panic_role`.
- `sema/declarations.rs`: one call after `site_records::complete`.
- `sema/bodies/effect_facts/site_locations.rs`: the caller-tracking fence for
  imported callees.
- `sema/header_attributes/{implicit_receivers,tracking}.rs`: the profiled
  advisory boundary, `cold_advisory`, and imported caller-tracking reads.

## Remaining gaps toward core's real `panic`, `assert_failed` and `Option::map`

In dependency order:

1. **Lifetime-parameterized nominals.** These block `Arguments<'a>`,
   `Location<'a>` and `PanicInfo<'a>`, and with them the `panic_fmt` and
   `format_arguments` roles. The region slots and their consumer variances
   and outlives requirements need to transport.
2. **`Location`.** Core's form needs `NonNull<str>` (a pattern-typed
   transparent struct) and `PhantomData`. Locally, `Location` needs these
   fields and the `PhantomData` role.
3. **The caller ABI.** This needs caller-site rows in rebuilt bodies, a
   consumer static `Location` per site, and tracked to tracked forwarding.
   Locally, even a provider call to a tracked function needs `Location`
   ("tracked call lacks its prepared actual caller location").
4. **`panic_fmt`'s body.** It needs `extern "Rust" { #[lang = "panic_impl"]
   fn panic_impl(..) -> !; }` in a provider, `PanicInfo::new`,
   `Location::caller()`, and `Arguments::from_str`, which uses transmute and
   `impl str`.
5. **`assert_failed`.** This needs `fmt::Debug` and `Option<fmt::Arguments>`.
6. **`Option::map`.** The earlier gaps remain: const callable bounds, the
   remaining lang and diagnostic items, stability and derive attributes,
   closures in provider bodies, callable bounds written by the consumer, and
   trait impls with methods (task 2).

Tasks 2 to 4 were not started in this checkpoint.
