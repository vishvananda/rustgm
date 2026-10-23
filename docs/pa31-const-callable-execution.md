# Closed source const callable execution

Development successor to `398abf45`, with shared source-call contexts from
`e091db86`. This is a bounded compiler-hook checkpoint. PA31 remains planned;
the original core crate and the broader source callable protocol remain required.

## Contract

The frozen original positive calls a helper with
`F: [const] FnOnce(u8) -> u8`, passing an actual const function item and 29.
The interpreter must execute that callback and produce 29. The defining
Conditional requirement remains in its declaration environment. Each reached
constant call pays its copied Const requirement using the actual caller,
complete instance, checked statement and paired contract. Cached code and
immutable requirement rows supply no per-use permission.

The additional executable argument profile is one complete owned function item
with a safe Rust `fn(u8) -> u8` signature, followed by one `u8` value. The
existing argument-free profile remains. Actual intrinsic targets retain their
separate checked witnesses and argument boundaries. CTFE drop glue remains
unsupported; empty source placeholders are never executed for it.

Defining call-operator goals use the actual existing callee-use effect. A closed
initializer demands Const, runtime use demands Ordinary, and the defining
generic const body retains Conditional. In particular, the same const helper
can be called at runtime with a function pointer, while a constant call with
that pointer must reject. Output equations still retain the full original trait
application proof obligation.

## Retained facts and ownership

The common `Operation::Call.source_use` is the sole defining owner/effect fact.
Its compact handle queries the current source catalog and exact semantic body.
Defining validation also checks the actual function owner, source proof site,
call-trait kind, original signature/result and exact receiver parameter image.
This first read-only mapping supports one directly owned type parameter and
complete closed function-item or pointer images. General inherited, lifetime
and composite receiver substitution remains required.

The interpreter's new Invoke path accepts actual function items only. It checks
the reached typed statement, opened signature and evaluated item/u8 arguments,
then shares ordinary call entry checks. Provider lookup preserves the complete
instance and paired code signatures. A callback with its own Conditional debt
still requires a separate Invoke-bound payment witness and is explicitly
unsupported in this profile.

Copied requirement rows retain `Clause`, including empty quantified binders.
Fresh payment opens those binders using the existing binder-aware fold; it does
not discard a binder or rewrite the defining environment. Nonempty quantified
payment remains unsupported. Requirement copying, folding and retained payload
storage are charged.

Direct interpreter function roots cannot inherit a reached caller's payment.
Source Conditional roots reject before both fresh and warmed code execution.
The evaluator derives provider mode from its actual report, and missing,
pending or foreign provider data cannot regain ordinary fallback. SourceClient
mode needs a future authenticated carrier extension.

## Work and layout observations

Defining validation uses one bounded statement pass. It charges attempted
statement, value, fact and descriptor accesses before lookup, including failures,
and bounds seen-set growth. Read-only template APIs remain unchanged. Mutable
owners retain attempted work in `body_work.canonical_steps`; that field is an
observation, not a global enforced quota. Reached execution charges the existing
cumulative Machine budget. Ordinary bodies take the zero-added-work path.

The v5 debug measurements are retained below. The new callback family repeats
reached calls to the same full helper instance; it is distinct from the earlier
caller/instance/noise families, whose controls also passed.

| Reached calls | Fresh payments, cold/warm | Canonical work, each replay | Execution steps, each replay | Retained row bytes |
| --- | --- | --- | --- | --- |
| 8 | 8 / 8 | 304 | 465 | 11,180 |
| 32 | 32 / 32 | 1,216 | 1,857 | 38,828 |
| 128 | 128 / 128 | 4,864 | 7,425 | 149,420 |

Each case retains two rows and one requirement owner, with no growth on the
second replay. Observed sizes are Engine 1696, Machine 4728, State 448, Body 448,
Code 304, IR Value 24, Invocation 20, Operation 16 and NodeFacts 80 bytes.
Actual Rust `size_of::<Clause>()` is 12; the earlier zero-size DWARF enum result
is excluded from evidence.

## Qualification and retained boundaries

Sources and declarations live in
`pa31/tests/spec/source-const-callable/{contract.json,README.md}`. Evidence lives
under `obj/parallel/pa31/const-callable-execution-v1`.

Independent `independent-v1` settled all twelve frozen rows: twelve metadata
and twelve object observations, nine native positives, and three E0277 source
negatives. The source/tool hashes match before and after; no ICE occurred.
The source-positive Conditional callback target remains required even though
this first candidate profile diagnoses it as unsupported.

The v5 all-target strict pass, five frontend test groups and eight private/work
passes are retained. Three private tests originally searched the pruned final
runtime program for a CTFE-only helper and failed their setup. The correction
observes the actual body after successful Code-purpose region/loan proof,
under a test-only scope bounded to two preselected DefIds. It preserves full
instances, rejects reentry and clears its thread-local state on failure. It
neither forces runtime demand nor reconstructs a substitute body or permission.

The v7 closing receipt passes strict compilation, the three corrected observers,
the new observation reentry/error control, and eight candidate native programs.
The defining check records 36 visits on completion, 19 on the failed lookup,
one on tiny-budget rejection and zero for an ordinary body. The v6 audit is
retained with an exact inverse replacement restoring its helper SHA256; only
the private unit-pattern lint correction changed afterward.

Native execution reuses the immutable v5 frontend because subsequent changes
are production-excluded test observation. Exact inventory selection and
frontend/backend/tool hashes agree before and after. Results are
`development-v7/results.json` (SHA256
`3fd679feebfcae8046c3103136b2a75a21df1b98053186d44e45e7f639f9b683`),
with `native-provenance.json` (SHA256
`5687484d64a30ee85bf8609733d6a0e952176a3ffcc89633817ebd282dc9c65f`).
The final Rust/script/input snapshot is `after-sources.json` (SHA256
`5fbb19bc18337e6faad0ab281e4f13b6bb8eab7d33b5509e200e13e57cdbd0cc`).
Earlier failed naming, layering, lint and observer setup receipts remain.

Candidate source/native checks use `pipeline::analyze_source` followed by the
public `lower::instances::build_roots_prepared` adapter, with its ordinary
original, specialized, ownership, loan and alignment checks. This is distinct
from CLI/artifact acceptance of public source traits and their metadata, which
remains required. The contract's `--emit-lowir` label names the intended output
view; it is not a claim that the complete source crate was accepted through
that CLI command. Combined release/text/performance gates belong to integration.

General tuples and callable signatures, Fn/FnMut, closures and borrowed
receivers, nonempty binders, new region/size proof debts, Conditional callback
payment, const intrinsic callback kernels, CTFE cleanup and complete Pin/Deref
remain required. Existing PA26 collection/derive/formatting and PA30 obligations
also remain; this checkpoint does not replace them with ordinary API factories.
