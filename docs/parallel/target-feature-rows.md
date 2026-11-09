# Target-feature rows (B2 F)

Lane D's slice F, 2026-11-09, on `implement/inline-target-features-20261109`,
from F0 (`b2a14ea76`). The integrator approved the F notice and its rulings.
F carries a free `unsafe fn`'s declared target features through version-13
module-body transport, so the real `_mm_stream_ps` body crosses the crate
boundary. It is a transport slice, not a library milestone, and makes no
claim about an original-core artifact or a whole assignment.

## Before

Every function with `#[target_feature]` stopped export under every profile
with "source named/foreign/target-feature function metadata requires its
wider profile". After S, that fence was where the stream witness, the crate
holding the real `_mm_stream_ps` body, stopped.

## Row

- **Flag bit 3.** The function header's flag byte (bit 0 the inline
  contract, bit 1 `#[cold]`, bit 2 `#[track_caller]`) gains bit 3, declared
  target features. After the inline payload come `count u32` and `count`
  names, each `len u32 | bytes`.
  - The names are the explicitly enabled features before implication,
    deduplicated: 1 to 128 names of 1 to 32 bytes, in strictly increasing
    byte order.
  - No closure, gate or table index travels, and the version is not bumped.
    Rows without bit 3 keep their exact bytes. No earlier v13 packet can
    hold one, because the exporter fenced them all.
- **Reader** (`annotation_rows.rs`), in this order:
  1. flags above 15: "unknown transported inline or callable advisory flags";
  2. bit 3 without `body_nodes()`: "source target-feature rows require the
     version-thirteen header profile", before the cold/caller test;
  3. bits 1 or 2 without `body_nodes()`: the existing cold/caller text;
  4. the inline payload;
  5. bit 3 with an always-inline hint: "source target-feature function
     cannot use an always-inline hint";
  6. count 0: "source target-feature set is empty"; above 128: "source
     target-feature set exceeds transport profile limit";
  7. a name above 32 bytes: "source target-feature name exceeds transport
     profile limit"; empty or not UTF-8: "unknown source target feature";
  8. a name not above the previous one: "non-canonical source target-feature
     set".
- **Validation** (`advisory_contracts.rs`). A trait row with bit 3 keeps
  "source trait header cannot carry a callable advisory". A function row
  needs an exported original body ("source target features require an
  exported original body"). Each name costs one work step, and its bytes are
  charged.
- **Width.** `s::HeaderContract` grows from 56 to 64 bytes, as declared, and
  `s::DeclaredFeatures` is 24. `s::NominalHeader` embeds `HeaderContract`,
  so S's pin moves from 96 to 104 (amendment 1). The provider's
  `target_features::Contract` grows by its declared `FeatureSet` (16 bytes),
  from 96 to 112, as declared.

## Exporter

`authored_functions.rs` keeps the old text for a named or foreign function
with features and for every target-feature function outside `body_nodes()`.
Under `body_nodes()` the checks run in this order, and the first failure
wins:

1. a trait or trait-impl member: "source trait member target features
   require their trait implementation transport";
2. a non-Rust ABI or a `const fn`: "source target-feature function form
   requires its wider profile";
3. not declared `unsafe`, read from the source row because the canonical
   signature always is: "source safe target-feature function export
   requires its contextual call policy";
4. no declared name: "unknown source target feature".

`advisory_rows.rs` then writes bit 3 and the declared names from the
contract's declared set, so export replays retained rows and never re-parses
source. F0's local rule rejects `#[inline(always)]` with `#[target_feature]`
before export.

An unsafe inherent method passes F's checks and then keeps the existing
unsafe associated-function fence of the ordinary profile ("source
nonordinary callable effect/safety/convention requires a wider profile",
`ordinary_profile.rs`). F therefore covers free functions only
(amendment 2). The legacy codec keeps "target-feature exports are outside
the generic/scalar metadata profile".

## Consumer

- **Eager re-proof.** The import ledger (`metadata_contracts.rs`) records
  each bit-3 row with one fixed charge and slot. `validate_signatures`
  re-proves every recorded row once the headers are installed, so a forged
  row on a function the client never names is still rejected. The checks
  follow the exporter's order on the consumer's own installed header:
  1. an associated function whose owner is not an imported inherent impl:
     the trait-member text;
  2. a non-Rust ABI or `const`: the form text;
  3. not `unsafe`: "source target-feature function is not declared unsafe";
  4. the names against the consumer's own table, closed:
     "unknown source target feature" for any failure.
- **Calls.** An imported target-feature function is called through its
  ordinary unsafe signature: no new call-site code, and `LowIR` carries no
  target-feature attribute.
- **CPU policy.** An imported asm body's register-class check
  (`encoded_assembly.rs`) uses the consumer's baseline plus the imported
  closure, read through `target_features::imported`.
- **Inherent admission.** No unsafe inherent method can be imported, so only
  a forged row reaches the inherent branch of the member check. A named
  private test forges bit 3 onto a safe inherent `Lane::get` and gets "not
  declared unsafe".

Accepted deviations from the notice:

- The notice's `Policy::Imported(FeatureSet)` variant was dropped. Nothing
  read it, and the asm CPU check reads `target_features::imported`.
- The dead `Contract::declared()` accessor was removed. The cpu_policy test
  reads the field directly.

## Known integrity limit (beside S4)

A forged addition to an imported function's declared features (bit 3 on a
function that had none, or an extra name) widens that body's asm CPU check
to the baseline plus the claimed closure. This is harmless while every
admitted register class needs only baseline features: `reg`, and `xmm_reg`
with sse. Any later item that admits `ymm_reg`, `zmm_reg` or `kreg` must
first cover forged additions, by inventory identity or an authenticated
feature set. spec.md names this condition. A forged removal, or `avx` added
to `bare`, cannot be told apart on this target: the client's outcome and
`LowIR` do not change.

## Fixture and outcomes

[`pa41/tests/spec/target-feature-transport`](../../pa41/tests/spec/target-feature-transport/README.md)
holds the provider, seven ordinary clients and the stream witness's provider
and client. Every outcome was declared in
`pa41/tests/target-feature-transport-outcomes.json` at 2026-11-09T12:30:08Z
(sha `644bb438305380c14a7a392df14794724eacbea28157228379e41bb54e6216e7`),
before the rustc oracle and before any F code. The four amendments are in
`obj/parallel/target-feature-transport/f-amendments`; the outcomes file
records each one beside the outcome it changes.

| Amendment | Declared | Content |
| --- | --- | --- |
| 1 | 13:30:02Z, after the rustc oracle, before any RustGM run | twin-keep's named fence (withdrawn by 4), twin texts, the untargeted bounded-work comparison, the NominalHeader pin, the forgery client and mechanism |
| 2 | 14:07:40Z, the integrator's option A | `Lane::get` and `inherent.rs` withdrawn after the first run showed the provider fenced; `opponent-inherent` and the safe-inherent forgery added |
| 3 | 14:23:30Z, after the first integration run | safe-pointer's text, pa29's unknown-flags wire control moved from byte 8 to 16, the witness's packet name |
| 4 | 14:28:00Z, after the rerun | twin-keep's declared parity restored, the witness's stop at step 3, one diagnostic probe |

Failed predictions, each recorded and not reblessed:

- **Inherent methods.** The notice's "(an inherent method, admitted)" was
  wrong. Unsafe inherent methods are fenced at export under every profile,
  which pa29's `const-method-outcomes.json` already declares ("unsafe
  inherent methods stay fenced") and `evaluated_methods.rs` tests. Target
  features on an inherent method join the queued associated-unsafe widening
  item, ahead of HR/C.
- **safe-pointer.** The declared "incompatible types" was wrong for the
  client and its local twin alike. Both give "incompatible callable
  convention or safety" (rustc E0308), as cpu_policy.rs's local
  `safe-pointer-missing` control does.
- **twin-keep.** Amendment 1 predicted a named fence for its `no_mangle`
  entry. The twin exports, so the declared parity against
  `generated/twin-keep.rs` stands.
- **pa29's wire control.** Byte 8 is now bit 3. Byte 16 takes over the
  unknown-flags text, and byte 8 gives "source target-feature name exceeds
  transport profile limit".

| Control | Result |
| --- | --- |
| provider | exports under `module-bodies-v13` in Lane B's `[14, 2]` macro envelope around the `[13, 0]` core. The bit-3 rows are `second` (`sse2`), `keep` (`sse`, `sse2`) and `wide` (`avx`), each unique in the packet. Re-encoding the decoded packet gives the same bytes. |
| `call`, `generic`, `pointer` | accepted, and each exits 42 natively with empty stdout, as rustc's does. `generic`'s `keep::<u8>` block equals the local `twin-keep`'s once each symbol is one placeholder. |
| `wide` | accepted, `LowIR` only |
| `gated` | the generated `x87` provider exports and the client, which declares no gate, is accepted |
| `unsafe-free` and its twin | "call to unsafe function requires an unsafe context" (rustc E0133) |
| `safe-pointer` and its twin | "incompatible callable convention or safety" (rustc E0308) |
| export opponents | `safe`, `trait-impl`, `trait-default`, `extern-c`, `const`, `no-mangle` and `inline-always` give their declared texts, and `inherent` gives the nonordinary text. None writes a packet. rustc accepts all of them except `inline-always`. |
| older gate | v12 and v10 keep the old text; v13 exports |
| codec rows | the thirteen declared reader and validation forgeries fail by name; rows without bit 3 keep their bytes |
| consumer forgeries | `ssx` and `soft-float` on `second`: "unknown source target feature". The u32 `clone`: the trait-member text. `zero`: the form text. `plain` and the safe inherent `get`: "not declared unsafe" |
| declared-accepted forgeries | `avx` added to `bare` and bit 3 stripped from `keep`: client outcome and `LowIR` unchanged |
| bounded work | one fixed ledger charge per bit-3 row: t(D) - u(D) > 0 and t(8) - u(8) = 8(t(1) - u(1)) |
| cpu policy | the imported-asm union admits `xmm_reg` with an imported `{sse}` over a baseline without sse, and rejects it with no import |
| stream witness, step 1 | `native.rs` before its observers exports: `_mm_stream_ps` carries its inline hint, `[sse]` and the one template `movntps [{p}],{a}` |
| stream witness, step 2 | `stream-provider.rs` exports; its lines 1-95 equal `native.rs`'s |
| stream witness, step 3 | stopped: see below |

Receipts are in `obj/parallel/target-feature-transport/receipts`. The base
is the F0 candidate `d9883c84` (from `b2a14ea76`), and the F candidate is
`a0176251`.

| Receipt | Result |
| --- | --- |
| old readers | S's candidate `868d91e1` and S's base `ca3ba92f` reject F's provider packet and the stream packet with "unknown transported inline or callable advisory flags", as declared |
| before | `d9883c84` rejects the provider, the stream provider and `native.rs`'s stream body with the old target-feature text |
| byte identity | `d9883c84` and `a0176251` publish identical v13 packets and `LowIR` for pa29 transparent-records, panic-attributes, macro-contexts, const-parameters, pointer-slices, named-constants and array-types, pa41 asm-transport and simd-transport, and the native.rs prelude before `#[repr(simd)]` |
| flip survey | 530 runs per binary over 69 spec directories (64 exports, 466 clients). The only 10 differences are this packet: both providers export; `call`, `generic`, `pointer` and `wide` are accepted; `unsafe-free` and `safe-pointer` give their texts. `gated` and `stream-client`, which the survey pairs with `provider.rs` and not their own providers, give "unresolved value name" |
| rustc oracle | `obj/parallel/target-feature-transport/rustc` (every prediction matched) and `rustc-a2`, rerun on the amended provider: `call`, `generic` and `pointer` exit 42, `wide` builds, `unsafe-free` gives E0133, `safe-pointer` E0308, and `opponent-inherent` and the inherent forgery provider build |

Focused checks:

- `make fastcheck` passed 11 private tests
  (`obj/qualification/fast/1794233743398829091`): F's five codec and ledger
  tests, the inherent forgery, the cpu-policy union, F0's hint rule, S's
  codec unit, the assembly-baseline union and the amended step-1 witness.
- It passed 9 linking tests (`obj/qualification/fast/1794234648834594509`):
  F's packet, export-control and witness tests, plus S's, A's and
  panic-attributes' packet and forgery tests as controls, including the
  amended wire control.
- The native root passed 4 (`obj/qualification/fast/1794234728100327619`):
  F's clients and S's, A's and the cold clients.

## The stream witness's stop

The fresh stream-client consumer is rejected with "nominal shape or
representation is outside the metadata profile"
(`sema/crates/export/records.rs`). This is the third named risk: an
imported SIMD type in a client `extern "C"` signature.

- A fresh client is a non-source library. Its own legacy interface discovery
  walks every public signature and admits named-field records only.
  `rg_stream_ps(p, a: __m128)`, `rg_stream_default` and `rg_generic` name
  the tuple struct `__m128`.
- Under the stop rule, the integration test records that text,
  `stream-client.rs` is unchanged and the native witness does not run.
- **Probe (declared, diagnostic only).** The client without those three
  functions stops at "exposed type is outside the scalar metadata profile"
  (`sema/crates/export.rs`), from the `*mut f32` parameters of the
  remaining exports.
  - Neither of the other two named risks is reached.
  - Both fences belong to the client's own library export and are
    independent of F's row.
- The next step is the integrator's decision.

## Remaining

- The stream witness's client fences above.
- Target features on unsafe inherent methods, with the associated-unsafe
  widening item.
- Safe target-feature functions, trait members, and non-Rust-ABI or `const`
  target-feature functions stay fenced, stricter than rustc.
