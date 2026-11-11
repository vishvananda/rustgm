# Implicit Deref method receiver checkpoint

Concrete `Hold<S>.get()` and the wrapped array iterator reductions now use
actual source `Deref`/`Target` selection before calling the selected method.
Ordinary library forms remain PA26 requirements; the actual source-role packet
belongs to PA31. Both whole assignments remain planned.

Receiver search retains a typed absent-candidate outcome and advances one actual
Deref target at a time. Existing field access storage owns those steps. Normal
method priority is checked at every level, and the final consumer chooses shared
or mutable calls. Each real call retains its source, signature, generic bounds,
regions, const and stability obligations. The final method remains in the same
checked plan. CFG construction evaluates the operand once, executes the real
Deref/DerefMut bodies and then evaluates explicit arguments. Capture and constant
consumers authenticate the same retained calls. Generated calls use implicit
source-function instantiation, keeping explicit final-method arguments separate.

The representation retains Operation16, Methods::Call20, Plan64, Calls8, Step40
and the cold place selection12. There are no new dense declaration/body fields,
global proof caches, compiler limits or backend/storage changes.

The [source packet](../pa31/tests/spec/deref-methods/manifest.json) now contains42
independently checked inputs:35 accepts and7 rejects. Forty full checked-body
contracts pass the finite selection:36 functional cases and4 depth/noise cases.
They cover generic method arguments, nested and parameter-bound receivers,
inherent/trait and shared/mutable priority, paid/unpaid generic and lifetime
requirements, captures, actual constant values23/41, and move/borrow/const/cycle
opponents. Independent Rust/C and RustGM/CPPGM native observers agree on
`receiver 23 41` and `receiver-order 23 125 41 134625`. The latter distinguishes
operand count, shared/mutable calls and receiver-before-argument evaluation.
Existing field order/const, raw-access policy, receiver128, arbitrary receivers,
capture and compact-layout controls also pass.

The two128-layer generic-chain inputs remain **required positives**. The tested
noise0 input exhausts the existing type-constraint work budget; noise128 is not
qualified. At depths8/32, canonical work1513/5185 is linear, while total inference
14674/192018 is quadratic. Noise128 changes neither counter. A failed initial
linear-inference hypothesis and the actual128 failure remain in the raw evidence;
no budget increase or language-negative qualification is claimed. An independent
cyclic-wrapper own-method experiment produced RustE0055 rather than its predicted
acceptance and remains outside semantic outcome claims.

Focused evidence is under `obj/parallel/array-receiver/`. Public receipts
1794379018015906478,1794380127522383238,1794380243985419751 and
1794380572512005089 contain the passing named controls and the retained failed
scaling attempts. Final finite scaling passes1794380763860385209. Native
1794380820997909008 passes all3 selected groups in36.80s. Every batch includes
strict selected Clippy, formatting and the architecture audit. No course text
oracle was generated. Manifest/upstream/harness validation and the subsequent
immutable original-core observation are recorded in the handoff.

The seven PA26 ordinary reductions remain required; bootstrap Deref imports
still prevent their full qualification. The existing conditional const adapter
execution/bootstrap/transport requirements, generic-chain inference sharing,
three older tuple-factory cleanup owners and optimized nested-proof measurements
remain open. No original core/alloc/std artifact or whole-library qualification
is implied.

Immutable CLI9109d6559285c010f3589bf28ed1420f734cbceff94cddb4290ca9d7f8a9af6e
binds compiler5f78ed963, empty tracked build diff and3900 Rust source hashes.
The40 frozen A/A+twoABBA control observations preserve complete output equality;
raw unoptimized ratios support no speed or neutrality claim.

Original core's reached array iterator len/is_empty failures are absent from the
new prefix. The unchanged120s observer times out at121.42045s with187 partial
records throughfunction7853,file227:4711. First now each_ref/each_mut,
array/mod.rsfile98:24013/25143, const-stable policy at indexing; next projected
try_from_fn_erased is separate. All722 hashes,375 opens,six bindings and original
profile/limits remain unchanged. ProgressSHA
 aae1f21d89790cb17f1ffa01539b3d90951c8253ec3faf505c1364313791c3c8.
No complete census, body count, throughput or original artifact is claimed.
[Machine evidence](pa26-implicit-deref-methods-evidence.json) binds the producer,
independent packet, actual native outputs, controls and original observation.
