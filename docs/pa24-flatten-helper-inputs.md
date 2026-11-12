# Helper return callable inputs

Pinned core FlattenCompat::iter_fold fails at flatten.rs:398, file142:10701, on
`self.iter.fold(acc, flatten(&mut fold))`. The helper returns
`impl FnMut(B,T)->B`, while its argument mentions only `T::IntoIter`. Naming T
works in the baseline; the inferred direct, named and free-applier forms fail
because the parent fold's required callable inputs never reach the helper's
opaque result. An associated result must not be inverted to guess its inputs.

The candidate maps the opaque type's formal type, const and lifetime captures
into the owning inference graph as an ephemeral `InferredCaptures` result,
shared with the existing trait-result refinement. Selection then reuses the
indexed declaration templates: one compatible callable clause supplies input
equations through the existing Probe and scoped predicate relation before
projection readiness. Normalization or Unresolved from head inspection continues
to the existing normalizer. No hidden representation is read, no unresolved
caller term is cached, and the original selector still proves the complete
trait, formation and lifetime requirements. No new engine, dense-model field,
backend, storage policy or compiler limit is added.

Fifty-two independently declared Rust1.98 outcomes are36accept/16reject, split
PA24 ordinary27 and PA31 actual-source25. Forty-six complete API outcomes
qualify: 32 body/rule cases, 12 width/noise cases and two real native exports.
Six entire contracts remain required. Implicit enclosing-return-opaque input
context needs one valid borrowed-return proxy and one escaping-borrow rejection
per owner; the earlier unresolved-input error is not qualified as the intended
lifetime rejection. Explicit twins pass and reject with an actual hidden-lifetime
error. Two ordinary native inputs with nominal or `Self::Item` impl where-clauses
stop at the existing callable-output conformance gate before the helper call.
Their bytes remain serialized positives in the packet's pending.json. No valid
source is reclassified as a language rejection. The 623 baseline wrongly accepted
the explicit escaping-borrow twin; the candidate rejects it, agreeing with Rust.

Focused receipt1794465771008351229 passes three exact tests/three targets in
33.57s with full formatting, architecture and strict selected production/CLI/
test Clippy; it supersedes1794463509909508691 only for two revised fixture bytes.
The closing review passes both native groups: new ordinary and source programs
print `flatten-helper 7 9 1234`, matching independent Rust/C. Left and Right
share Bucket as their associated result but perform distinct conversions and
callbacks. Old ordinary/source match programs still print `match-capture 7 9 12314`.

Widths8/32/128 scale with calls; adding128 unrelated types changes no counter.
PA24 inference/traits/canonical are23636/90/4302,90894/354/16902 and
359912/1410/67302; PA31 are33799/289/11611,128153/1129/41971 and
505555/4489/163411. These bounded-work observations make no optimized or
original-core speed claim.

Twenty-three new ordinary course outcomes (16accept/7reject) complete the CLI and
normal text contract. Before generation, all sixteen positives cross the
independent Rust/CPPGM object bridge, and the fifteen declared empty objects
contain no defined code or data in either compiler. All seven rejections are
independently rejected by Rust. Native execution is separately established for
the one real exported program. All prior PA24 (661) and PA21 (867) references,
and every other active stage, reproduce byte for byte with the candidate before
normal scoped generation produces23 reviewed triples. All28,204 prior reference
hashes remain unchanged. PA24 now has684 cases,490accept/194reject,396 runtime
comparisons,5 codegen-only controls and89 object controls. No planned assignment
is activated.

The first course declaration of `shared-output-explicit/inferred` as empty
objects was wrong: Rust conservatively code-generates the non-generic private
impl methods `Left/Right::into_iter`, while RustGM emits an empty unit, so neither
object classification existed for those bytes. That attempt produced no oracles.
Revised, still unpublished fixtures use generic `Left<T>/Right<T>` carriers and
keep both non-injective outputs and the identical helper/trait text. Their
outcomes were declared before execution; Rust and both CLI/CPPGM objects are then
empty, and the 623 baseline rejects the inferred form. PA31 counterparts are unchanged.

Upstream, maintainer harness and layout checks pass.

Clean sourceee8c11abb freezes CLIa08ab1a4 with3907 Rust hashes and an empty
tracked patch. Forty exclusive A/A plus ABBA accepting observations preserve
complete outputs against preceding623 for the import, capture-free source try,
priority-graph and explicit-helper library LowIR controls. Wall ratios
1.138/1.035/.977/.989 (CPU1.138/1.036/.976/.989) against A/A.933/1.223/1.009/1.001
are dominated by20-70ms process noise; no optimized speed, neutrality or
original-core throughput is claimed. A first attempt stopped on its own
output-equality guard: explicit helper type inspection gains one interned opaque
instance before existing entries, shifting later identities. Acceptance and all
published references are unchanged; that attempt's raw outputs are retained.

The unchanged120s original observer ends at its deadline after121.213146s with
62 partial records and14,659 attempts; last9491, alloc/mod.rs257:13060, line281.
FlattenCompat::iter_fold4718/body142:10360 is reached without a record, as are
iter_rfold and the four Flatten/FlatMap fold/rfold callers, alongside all fifteen
earlier required bodies. No new failure appears among previously passing reached
bodies. The first remaining is iter_try_fold4720/body142:11329, error142:11801,
flatten.rs:431: an associated type projection requires normalization at
`self.iter.try_fold(acc, flatten(&mut self.frontiter, &mut fold))?`. The helper's
first input `&mut Option<T::IntoIter>` must wait for T from the parent callable
bound before normalizing through `I::Item: IntoIterator<IntoIter = U>`.
Postcheck verifies722 source hashes,375 opens, six configuration bindings,3907
current/immutable Rust hashes and unchanged profile/limits. ProgressSHA
9eb0304af922a286599a69f0622e69663091bc45afaed7dac1c7ec8feda3d9ee.
This is neither a complete census nor a core/alloc/std artifact.

The [machine record](pa24-flatten-helper-inputs-evidence.json) binds the raw checkpoints.
