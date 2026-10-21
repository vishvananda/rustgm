# Mutable references reconstructed through symbolic raw pointers

This bounded PA27 slice implements constant `&mut *raw` for live, initialized,
ordinary nonstatic symbolic storage when the existing pointer value is writable.
Success returns the identical full pointer. Shared reconstruction still restricts
only its returned value; neither mode changes an original alias. Raw constness
and casts to `*mut` grant no new authority. The private `raw_reference(place,
mutable)` routine shares the existing type, layout, bounds, provenance, storage
liveness, initialization and actual leaf alignment checks. Shared and mutable
wrappers select the result authority. No public enum, pointer representation,
source lifetime/variance proof, ownership rule or backend change occurs.

Independent qualification disproved five initial rejection hypotheses: pinned
Rust accepts unused shared-derived (direct and call/product), expired, null and
unaligned mutable-reference formations. All original bytes and hypotheses remain.
Separately demanded shared-derived writes and expired/null/unaligned reads reject.
A shared-derived mutable-reference read is independently accepted too. The initial
Writable-only, initialized/live profile conservatively leaves these accepted forms
unsupported. An eager formation check is not proof that Rust requires reading the
contents merely to form a reference. Shared-derived write rejects at the explicit
profile gate, so this test does not qualify a later write-validity check. Existing
ordinary raw-store permission tests remain independent.

Twenty-five full-object outcomes distinguish nine supported acceptances, seven
independently rejected controls, and nine accepted-Rust unsupported cases. Source
uninitialized-place and packed-reference rules remain distinct from CTFE validity.
A physically aligned packed-field raw pointer can form a mutable reference, while
source `&mut (*packed_raw).field` retains its executable packed-reference rejection;
its type inspection succeeds. A packed whole-object mutable reference uses its
own alignment. Unaligned leaf access and actual expired/null dereferences reject.
General reinterpretation, union references and static mutable-reference formation
remain explicit unsupported cases. Transient mutable local references succeed;
references escaping their local allocation reject.

Three ignored-by-default native groups compare constant values for local/raw
constness casts, call/product/copy and alias propagation, packed fields/whole
objects and symbolic ZSTs with pinned rustc. Four private invariants pass,
including exact pointer identity, writable authority preservation, shared-input
denial, unchanged original alias and expired storage rejection. Nine affected
source/scaling regression tests pass. The original mutable-reference fixture
becomes supported without changing its bytes.

At N=8/32/128, ordinary loops use 538/1882/7258 steps and 49/73/169 slots; raw
reconstruction uses 610/2170/8410 steps and 48/72/168 slots. Both grow linearly;
the observed raw slot delta is -1 at every size. A shared test helper owns the
paired qualification and bounds, while the reference-cell and mutable suites
supply separate source programs. These observations make no wall-time neutrality
claim. Strict all-target/all-feature Clippy and the 1,368-file audit pass.

The retained failed gates are test-only: assertions requiring Constant Debug,
an unused wrapper, duplicated scaling code, a helper count type mismatch, a
seventh reference filename stem, and an imprecise packed diagnostic expectation.
No production fix was needed after the initial implementation. Corrected source
checks and regressions run after the final wording correction; native/private
passes are retained rather than repeated. The final receipt records fresh object
and all-view hashes, build, upstream, lint and audit results.

Remaining PA27 boundaries include the accepted forms above, deferred mutable/
foreign/cyclic-static reference formation, union-field references, numeric nonnull
ZST provenance, general reinterpretation and offsets, pointer-bearing union Images,
shared-derived interior-mutable writes, the default runtime invalid-reference-
casting lint and full packed/pattern/release closure. PA28's unimplemented families
and release requirements remain as recorded in its reviews. No active references,
assignment activation or full-release claim is made here.

The later [union-reference review](pa27-union-reference-review.md) supersedes
its raw-derived scalar-Image union-field boundary. Direct source union borrows
and other deferred profiles remain explicit. Historical receipts are unchanged.
