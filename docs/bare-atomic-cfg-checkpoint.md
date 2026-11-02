# Bare atomic target predicate

LIB-117's missing atomic_swap comes from a target-model defect. RustGM supplied
atomic width values and bare target_has_atomic_load_store, but omitted bare
target_has_atomic. Original core guards its atomic_swap helper with the bare
flag, while its AtomicBool caller uses the width8 predicate. The helper was
therefore removed while the caller remained active.

The pinned compiler's [target configuration](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_session/src/config/cfg.rs#L268)
publishes the bare atomic flag for a bootstrap/nightly CAS target. Its
[cfg gate table](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_feature/src/builtin_attrs.rs#L15)
gates target_has_atomic_load_store, independently of the ungated bare atomic
predicate. Both primary files were read locally and hashed before execution.
Eighteen declared stable/bootstrap metadata outcomes match pinned Rust; three
raw cfg listings retain the actual target facts. Stable and bootstrap outputs
differ as predicted. The course already uses bootstrap-capable load/store facts.

The fixed target now includes bare target_has_atomic. Width values, primitive
alignment facts, source feature/macro authority and override rejection remain
intact. The library does not read the host bootstrap environment. This adds one
constant fact to the existing bounded lookup, with no new field, cache, parsing
pass or semantic permission. Selected source functions type-check normally;
direct and macro-generated missing load/store permission remain hard errors,
including under capAllow.

## Qualification

Three [PA3 predicates](../pa3/tests/spec/bare-atomic/README.md) and six
[PA8 source/macro cases](../pa8/tests/spec/bare-atomic/README.md) declare outcomes
before reference generation. Four source positives retain one real helper and
complete ordinary name resolution and type/body analysis in both supported
editions. Two negatives require the actual load/store feature. Existing captured
literal, false-predicate, removed-owner and repeated macro-allowance controls pass.

| Gate | Actual result | Receipt SHA-256 |
| --- | --- | --- |
| Independent Rust | 18 metadata outcomes, stable/bootstrap, plus raw cfg/version | `f69455d7419a227c051094a19b66b72d299806172913e43507903fb8da60f200` |
| Private/strict Clippy/static | 7 exact checks,128.34s | `36543aeef12f5ad5cc1deea86a26f3b7b3f1bfd3331924b5ad81fa9cd1ed652b` |
| Production/CLI strict checks | 36 checks,91.54s | `c3562fe9d1941c22a8902bf157932dc61347e7f0592373fb4db6e61f7e36ba60` |
| Joined CLI/text/harness/core | Passed,40.14s | `28b7e0f9ddcf069282425d01b91824b7caf5ba40361af92be984adb780d95808` |
| Unchanged-binary call-path observation | Passed,17.96s | `328fb673972d4a86095a3ccd3fa23cf40bc7ba3b4d13014c1294e394ca83c494` |

Evidence is under obj/parallel/bare-atomic-cfg-v1/reference-v1 and core-v1,
and obj/qualification/fast/1793643551065094141 and1793643772251203864.
The final compiler/tool fingerprints match across gates. Sixteen additional CLI
observations check predicate values, ordinary types, both lint-cap modes and
explicit host RUSTC_BOOTSTRAP=0. Source bytes stay frozen throughout.
The generator writes only the nine new oracle triplets and two scoped batches.
All PA3 93/93 and PA8 126/126 cases pass. Upstream24/Reference126,275 harness
tests,386 maintainer roots and44-stage discovery pass.

Widths1/8/32/128 retain one metadata capture and one parser callback, no gated-use
storage, and11/32/104/392 syntax nodes. Existing load/store captures retain their
real paid gate records and nine actual primitive implementations. These are
structural scaling observations, not wall-time speedup claims. Strict Clippy
initially rejected an unnecessary Result on the pure fact test; it now returns
unit without an allowance. No local check exceeds300s or uses a release build.

## Explicit configuration provenance and next requirement

The derived original-core manifest records exactly one implementation hash
change: dev/src/expand/config.rs, from
6dd46a1eaea77b801d2456ce14b55be3c0f909757523e17d696bc801e68d8734 to
c111892059dd18b420ee63d29d35cb2a6eb0d051df235cf2dbe176a70ab2ef23.
Its configuration_binding_changes records the false→true bare predicate,
independent receipt and primary-source hashes. The historical base manifest
is preserved. This is an explicit target-model correction, not a claim that
configuration implementation or effective predicate truth was unchanged.

Immutable CLI
b39b10120f92741306d430d41272c54c1036761b3d0b9b45cb4654239988d408 passes
the earlier atomic name failure. The18.89s attempt keeps all722 original source
hashes,375 opens, target hardware, CLI arguments, capAllow and finite limits;
no custom --cfg is added. It reaches file14:6456's
`source-core mode has no bootstrap library fallback`. Resolver::bodies now reaches
post-traversal iteration reservation, whose bootstrap-only builder requests
synthetic library metadata. The first recorded for-loop is the open-order
bignum.rs150 candidate using iter::zip. PA26/31 must consume actual source-owned
Iterator/IntoIterator/Option declarations and calls, retaining the no-fallback
guard and ordinary source/selection/borrow proofs. Name-only admission or enabling
synthetic core identities does not close that requirement.

A guarded debugger observation of the same immutable binary confirms the stack:
Resolver::bodies → reserve_iteration → prepare_iteration → prepare_external_traits.
It verifies all722 source hashes before/after, preserves the configuration binding
and changes neither source files nor the executable. The raw stack and receipt
are retained under iteration-stack-v1. The guard is the current blocker, rather
than a missing explicit iter::zip path. Iterator, next and into_iter source roles
are absent from the current72-role source bank; actual core declares them on
the Iterator trait and its next method and on IntoIterator::into_iter.

Full CI41494273651 on the earlier storage checkpoint and41499430614 on the
namespace checkpoint both pass every gate. This target correction has its own
focused evidence above; its full successor is tracked separately.

No original core/alloc/std artifact or whole assignment is complete. Successful
generated metadata is scoped-pruned. Raw inputs, diagnostics and receipts remain;
only the current qualified CLI and latest warm caches stay pinned.
