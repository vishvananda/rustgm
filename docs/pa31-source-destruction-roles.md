# Source Drop and Destruct header prerequisite

This checkpoint extends the actual-source binding catalog on combined base
`5af6c837`. It does not provide executable destruction or the NeedsDrop kernel.
Worker C owns exhaustive destructor registration and that kernel; the header
boundary intentionally rejects execution until the exact registration exists.

The pinned source is Rust 1.98 revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, shared read-only under
`/home/vishvananda/rustgm-sources/rust-1.98.0-88d9e12ae178fab0fb5cc050a94da85685d449ea`.
The minimal Drop declaration is an independently valid provider profile. The
unchanged full `library/core/src/ops/drop.rs` declaration, including its two
provided methods and Pin receiver, remains required. Its genuine Deref/Target,
Pin, Destruct eligibility and drop_glue consumers are separate pending work.

## Identity, header and selection boundaries

`LangItem::Drop` and `Destruct` append at indices 14 and 15. The private validated
role mask becomes u32 without reusing an existing bit. Reservations remain
possible identities, completed actual headers become validated identities, and
neither status establishes registered implementations or const-drop eligibility.
No bootstrap drop-trait alias is installed.

Drop's first profile requires the exact const trait with no extra defining
constraints and one ordinary safe Rust method, `drop(&mut Self) -> ()`. Its single
receiver binder, actual Self type, member owner, generics and signature are
checked after the ordinary definition scheduler drains that real method. Header
validation does not manufacture a body or implementation. Destruct requires its
actual const marker header, exact ordinary PointeeSized prerequisite, no extra
constraints or members, and its actual deny-explicit-implementation attribute.

Only an ordinary application of the exact validated Destruct owner can use the
existing Builtin proof route. Source candidates, pending headers and scoped
assumptions keep their existing priority. The ordinary formation check runs;
nonempty returned formation prerequisites explicitly remain unsupported rather
than being discarded. Const and Conditional applications receive no fallback.
The semantic Builtin consumer validates the same exact ordinary application.
There are no new goal/evidence/IR variants or general marker-to-method bridge.

## Fail-closed execution prerequisite

`Catalog::require_source_destructors_prepared` is a temporary explicit boundary.
No reserved actual Drop means immediate success, preserving the existing
fourteen-role providers. Reserved but unvalidated Drop is an error; validated
Drop without exhaustive registration is explicitly unsupported. Copy carrier
preparation calls this before deriving decisions, and reached constant calls call
it before their effect and code-cache decisions. Neither a warm Code snapshot
nor a shape-based Copy answer substitutes for registration.

Worker C replaces only the facade's implementation with a cold exact-DefId
registration marker after exhaustive collection/preparation succeeds, including a
completed empty registry. Actual role validation and later implementation
conformance remain mandatory. The private adversarial tests deliberately
withhold that readiness; future successful registration must not be confused
with the header-only checkpoint's rejection expectation.

## Independent source observations

`obj/parallel/pa31/source-destruction-v1/plan.json` freezes ten source inputs and
their SHA-256s before qualification. `independent/commands.json` retains exact
pinned rustc object commands and statuses; `independent/summary.json` records all
ten matching outcomes. The ordinary Destruct generic-call program additionally
links and executes with answer 7.

Rust rejects the local lookalike bound, explicit Destruct implementation,
forged Drop struct, duplicate Drop role and wrong implementation receiver. Rust
accepts unused Drop declarations with a wrong receiver/result and accepts a
Destruct declaration without the deny attribute. Those last three are explicitly
unsupported provider profiles, not Rust-invalid programs. Their original
accepted source observations are retained unchanged.

Candidate qualification is complete for this header-only scope. The
`development-v3` receipt preserves nine frontend passes and four private passes;
its fifth private test initially failed because its fixture requested only the
batch of generic templates, which correctly omitted both nongeneric const
functions. `development-v4` explicitly requests those same actual functions
through `Prepared::function` and `Collector::function`: the corrected Copy plus
cold/warm CTFE gate passes. No production behavior changed for that test repair.
The union is nine frontend and five private controls, with no missing cases.

`development-v4/native` records five successful executable answers: ordinary
Destruct 7 and the unchanged Copy 7, Clone 11, function-item FnOnce 23 and unused
conditional runtime 7. They use the ordinary prepared original/instance proof
and native lowering route; this does not claim public-trait metadata export.
The first audit found a seventh `destructor_*` filename; `test-path-move.json`
records moving its byte-identical test to `registration_seal.rs`. The existing
successful test binary measures layout without recompilation after that purely
mechanical move. `closing-v1` passes final formatting, all-target/all-feature
strict Clippy, that exact layout control and the 1,957-file audit (zero findings).
It retains the full final development source inventory with SHA-256s.

Measured on this combined base: private Validated is 28 bytes, source Catalog
416, Reservations 432, Declarations 7040 and Resolution 2032. The previous
actual-source Catalog receipt is 376 bytes at
`source-traits-v1/closing-tail-v2/current-catalog-layout.stdout`; two additional
20-byte optional declaration slots account for the 40-byte cold owner growth.
The resolver's cold reservation array likewise gains two slots. The u32 mask
fits the existing private record's padding, and this patch adds no fields to
ordinary Declarations, Resolution, Body or Contract records.

The original failed compile/lint, missing-fixture and filename-audit receipts
remain retained. Root owns combined active-text/full-release qualification; this
worker did not repeat those suites or generate references. No whole-core,
whole-crate artifact export, runtime destructor or paid CTFE acceptance is
claimed by this header implementation. Generic Copy/FnOnce body
inspectors, source Freeze, original Pin/Deref/drop_glue and conditional const
payment remain required follow-ups from the prerequisite record.


The immediate semantic follow-up before actual Drop execution is enabled must
also reject actual trait-member UFCS/function-value uses, not just collected
implementation-method IDs. That identity-based source control remains separate
from this fail-closed header checkpoint and the full original Pin/Deref family.
The parent integration must preserve the pre-proof gate when adapting
`body/copy_requirements.rs` to A's BodyOrigin/ProofSite migration; no borrowed
proof fields, neutral source requirement Arcs or actual caller identities change
here.
