# Overflow check query execution

Original core reaches the actual registered overflow_checks intrinsic from pow.
Pinned Rust1.98 specifies CTFE true independently of runtime overflow flags.
The five PA31 overflow-query sources and C observer freeze independent outcomes
before implementation. Four accept and one invalid-result declaration rejects
normally under both enabled and disabled checks. An ordinary same-name function
still returns its authored false value. Three actual intrinsic cases reproduce
the oldc52c555a unsupported constant-intrinsic diagnostic.

The independent native observer prints1 1 1 with enabled checks and1 0 0 with
disabled checks, separating constant from runtime policy. RustGM's driver uses a
fixed checked arithmetic profile, already documented by the inherited-overflow
attribute owner. Its runtime answer is therefore true. A future selectable
profile must provide and transport its own policy; no such profile is inferred.
The provided false body does not supply intrinsic CTFE or runtime semantics.

Reuse the existing zero-input runtime-check selection and actual reached Call
proof. The selection authenticates the completed registered declaration, full
empty argument list, safe Rust signature, no formals/predicates and constness.
Both CTFE and lowering retain actual caller body, destination, callee and source
provenance. Dependencies omit ordinary body demand only for this exact typed
direct operation. Reification and imported policy provenance retain their guards.
The existing UB configuration/frozen/deferred decisions are preserved; only the
new authenticated overflow operation binds true for the current driver. No dense
catalogue fields, intrinsic enum/transport version, backend bridge or budgets change.

Production1794289740510319548 passes five exact groups, including the new
five-case packet, six static-knowledge cases and existing UB both-profile,
header/feature and indirect-call guards. Private1794290079406487644 passes eight
checks in192.80s with strict private Clippy; the earlier attempt caught one new
test-helper borrowing lint, corrected without production semantic changes.
Native1794290285447742973 passes two groups/seven actual programs: the new
1 1 1 observer and six existing UB configuration combinations. Strict selected
Clippy, full format and architecture pass. Harness313/466 roots/44 stages and
upstream128/Reference126 pass. All five frozen CLI outcomes match the contract.

Committed compiler9cf816e81 binds immutable dev-fast CLI
9af9f1877ce5e71ec9335a80503f683506adba15585e34a6c4b6ad1b87ae4552,
all compiler/test Rust hashes and an empty patch. Twenty same-profile A/A and
two paired ABBA observations preserve complete output equality for the ordinary
name and74 existing inputs. Ratios0.883/1.007, A/A0.910/0.985; no speed claim.
The three intentional rejections change to acceptance outside equal-output controls.

The unchanged722-input census opens375 sources and checks42310 declarations,
none failed/blocked and bodies0, in73.81s. It clears pow's overflow query and
reaches overflowing_mul at num/uint_macros.rs:3171, byte137546, with the
unsupported constant-intrinsic diagnostic. The actual call is mul_with_overflow;
the uint_impl expansion remains num/mod.rs byte45951. TSV SHA256
3874ec484a5efa1dd1126347df8b957ce8c6be2f1ba298ec243b84accdd25e46.
Inputs, profile and limits remain unchanged. No original artifact is produced.

Full slice predecessor42760129302 succeeded in every stage. Full qualification
for this combined static-knowledge/overflow checkpoint is the next dispatch;
do not cancel an active successor to publish subsequent development changes.

The first negative reference probe forced CTFE on an invalid intrinsic signature
and triggered a reference compiler ICE. It is retained under obj and is not a
language rejection. The frozen negative removes evaluation and yields ordinary
E0308 under both independent policies. Raw probes, source-bound immutable binaries,
receipts, independent outputs and paired observations remain under pa26-work's
obj/parallel/overflow-query. Original artifacts, full assignment release, broader
profiles, optimized nested-proof measurement, HR transport and client exports
remain open.
