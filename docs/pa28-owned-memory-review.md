# PA28 Sized pointer transfers and constant static reads

The current checkpoint supports the real Sized-only signatures of canonical
`core::ptr` / `std::ptr` `read` and `write`, including concrete non-Copy values.
It supersedes the concrete Copy-only profile in `pa28-memory-review.md` without
changing the frozen CPPGM vocabulary or backend. No assignment is activated.

Runtime transfers reuse ordinary owned arguments/results and typed layout spans.
`ptr::write` consumes its argument and does not drop overwritten storage. A read
produces an owned snapshot without marking the original source place moved.
Generic and reified calls use the existing ABI and full-instance shim identity.
Unsafe callers retain their obligations for initialization, alignment and valid
ownership; the compiler does not invent physical move-byte invalidation. The
ignored-read destructor counter control holds only raw/scalar fields, rather
than asserting that duplicating arbitrary resource ownership is valid.

CTFE reuses exact raw storage type, provenance, bounds, raw-anchor alignment,
initialization, liveness and per-pointer write authority. Copying a scalar union
Image preserves its initialization mask and does not validate unused alternative
fields. Ordinary source moved/uninitialized-place checks remain separate from
physical byte initialization. Actual destructor execution is still const-invalid.
A returned or static-owned Drop value need not execute a destructor during CTFE.

The former Copy-only machinery is removed after confirming no other consumers:
the bounded Copy API and optional hot-loop branch, reader helper, success cache,
copy_steps counters and four private capability tests. No fictitious source Copy
bound or replacement capability walk is introduced.

Typed owners distinguish constant evaluation from static initialization. Frame
context survives suspension, const functions inherit it, and named, associated
and inline constants establish constant context. Exact initializer roots use the
existing execution_target owner seam. A direct Const block initializer retains
its declaration as semantic owner, just like a root closure, so ordinary inline
registration supplies its real anonymous boundary. This also works through the
bounded parsed wrapper walk; the interpreter never classifies source syntax.

A nonzero by-value read in constant context rejects an interior-mutable static
ROOT type, including reads of its ordinary field or inactive enum variant.
Static initializer reads remain accepted. Zero-byte reads and eager reference
validity checks are separate; neither bypasses raw bounds/alignment/init/liveness.
Only completed Ordinary root-type facts are cached. Four retained logical slots
per cached root conservatively cover hash capacity; access permission is checked
at each read. Exhausted cache storage publishes no fact and preserves unrelated
completed entries. Mutable/external static profiles remain explicitly separate.

`pa28/tests/owned-memory-contract.json` declares 52 object outcomes and all three
frontend views: 33 Rust-positive and 19 Rust-negative programs. Two ordinary
runtime ownership negatives deliberately defer from type inspection to checked
lowering. Three Rust-positive profiles remain unsupported: nominal aggregate
mutable-reference lifetime differences in two cases, and mutable-static reads
inside static initialization. Direct mutable-reference cells work. Two scalar
owner controls live byte-identically in PA25's `inline-owner-context` lane; the
Atomic/memory cross-feature controls remain in PA28.

Fourteen new native comparisons pass, including overwritten versus assignment
destructor counts, forget/replacement, moved source storage, generic/reified and
nested transfers, ZST destruction, CTFE results and static-context/reference
controls. Eleven existing memory comparisons also pass, including the separately
expected bounds-abort diagnostic/status distinction. Eight private tests pass;
ten affected initializer/drop groups pass. Strict all-target/all-feature Clippy,
syntax audit (1,443 files, zero findings) and 24+126 upstream checks pass.
All active PA1–PA23 text comparisons pass with zero differences. The ordinary
scalar control is byte-identical to 30bc44cd in all three views.

The 36-point matrix varies Copy tuples versus non-Copy wrappers, widths 2/16/64,
repetitions 1/8/32 and unrelated aliases 0/64. Four selected instances always
produce four shims. Logical retained slots are exactly
`4*width + 18*repeats + H`, where H is 19 for tuples and 23 for nominal wrappers.
Unrelated types change neither execution work nor storage. Work grows linearly
with width and repetition; these are logical counters, not timing/RSS claims.
The independent static-root property invariant costs 9/37/133 steps on its first
walk and 64 steps for 32 repeated checks at every width. The N=8/32/128 ordinary
loop records 344/1160/4424 steps and 30 slots; pointer transfers record
504/1776/6864 and 36 slots. The old Copy-only observations remain unchanged in the
historical review; they are explicitly superseded rather than rewritten.

Original hypothesis disagreements are retained: rustc accepts three initially
predicted moved/aliased cases, and rejects the initially predicted ordinary-field
and inactive-variant static reads. The original wide slice-index/.len consumers
also remain retained and require integrator qualification on combined main.
Numeric/expired ZST pointer profiles, general reinterpretation, pointer-bearing
union Images and the declared lifetime/static profiles remain open; this is no
PA27/PA28 completion claim.

The first context check failed Rust borrowing, then passed after shortening a
reader borrow. A subsequent Clippy gate found push over the function limit; the
context-selection helper reduced it without a lint allowance. Audit rejected
unreviewed include macros, so native fixtures now use ordinary runtime file
reads. All failed receipts remain. The final receipt pass completed every check
but its optional trailing cleanup used an invalid keyword; a successful bounded
closure verified unchanged source/compiler hashes and corrected cleanup without
repeating passed checks. No resource-limit failure or language rejection was
inferred from those setup failures. Local layout was not repeated against the
previously identified receipt boundary; combined layout belongs to integration.

Base: `4b099b49`. Source fingerprint (sorted dev/**/*.rs path, NUL, bytes, NUL):
`e9c97690eb31d494e7b0827c045ba7c73589f49599fa6a179ad977b7ac531307`.
Compiler SHA256: `40e3d3acd9c7e9ebbbf66b975632879a01860e99af9a8eb6c9de61aa30ceb5a2`.
Final functional evidence: `obj/parallel/pa28/owned-memory-final`.
Final input/output/text receipts and successful closure:
`obj/parallel/pa28/owned-memory-finish`.
Tracked outcomes bind the exact fixture, contract and output hashes.
