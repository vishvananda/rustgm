# Source Deref calls keep their own proof

The reduced original `RefCell::clone` failure was a proof-ownership error. Lookup
selected `T::clone` from the caller's `T: Clone` bound, then marked its syntax node
as requiring caller evidence. Preparing the implicit `Deref` call later reused
that node and incorrectly demanded a caller assumption for `Guard<T>: Deref`,
even though a validated implementation existed. A diagnostic capture shows the
same complete Deref application first using normal selection and later being
incorrectly marked `assumed=true`. The temporary diagnostic binary is retired;
its raw typed obligations, source patch and producer hashes remain in obj/.

The existing sparse lookup index now retains the selected member. Actual protocol
calls also keep a distinct use kind: they pay their complete application through
normal selection, while the explicitly selected member keeps its caller-evidence
restriction. This matters even when two calls use the same Deref member with
different Self applications. Effects still use the actual call context. No new
proof engine, global cache or dense body/node field is introduced.

The preceding checkpoint's full CI43059556041 failed one existing Rust negative,
`result-self-context`; RustGM incorrectly accepted `let a:S=K::make(id)` by using
a result-only Self as an input prefix that selected a function-pointer argument.
The original test bytes and negative outcome are unchanged. General early result
hints now require the unresolved result to occur in the actual formal input
graph. The charged walk visits shared nodes once and excludes non-injective
projection inputs. Ordinary result coercion still runs later; existing function
pointer result context remains. The valid `Clone::clone(&guard)` control retains
its expected result because Self is shared with the direct formal input.

## Qualification

The packets own40 independent Rust1.98.0 object outcomes,34accept/6reject:
PA26ordinary13(10/3), PA30native2(2/0), PA31source25(22/3). Every input and C
observer appears in its owning manifest. All57 prior borrow-coercion input bytes
remain unchanged. The25 source typing outcomes and24 complete prepared-root
outcomes are qualified. One valid borrowed-reference return has successful
pipeline inference but a required complete RootBuilder artifact remains pending;
its last full request reported `generic aggregate requires instantiated fields`.
It stays positive in PA31 pending.json. Ordinary15 still stop at existing
core::ops imports; those errors qualify no language rejection or native artifact.

Final production receipt1794418559800884712 passes15 exact tests across3 targets
in121.07s, including the exact CI regression, old factory contexts, semantic
inspection, receiver and coercion controls. The test/fixture-only scaling
extension passes3 exact tests in63.24s (1794420857250870920). Selected strict
production/CLI/test Clippy, full formatting and architecture pass. Final native
1794418832230328990 passes4 exact tests in83.66s: two new actual RustGM/CPPGM
programs print `deref-clone 23 123` and `deref-member 23 12`; both match independent
Rust/C executions. Existing coercion4, receiver1 and callback3 programs also pass.
Both ordinary native programs remain required positives behind bootstrap imports.

The changed path has frozen widths8/32/128 and unrelated-type noise0/128. It
clones the target once, then repeatedly passes the owned value through a generic
call whose expected result must reach its reference input. Inference work is
6492/23580/91932; trait selections55/199/775, method steps43 and probes1. Every
counter is identical with128 unrelated types. The old coercion counters remain
3653/13253/51653 and56/224/896, with no method probes. The immutable counter
harness copy was explicitly unlinked after observation. This is bounded-work
evidence; no optimized or original-core throughput claim follows.

The final identity/upstream/harness checks verify all40 outcomes and manifests,
all57 original bytes,17 minimal/5 full-Clone/3 module-scoped source prefixes,
128 upstream fixtures,126 Reference files and315 harness checks/468 roots across
13 binaries/44 stages. Four initial full-header drafts lacked const stability
metadata; their original bytes/errors remain and all four corrected inputs retain
required accept outcomes. Eight auxiliary commands initially omitted explicit
profile flags; only those eight were rerun with the declared flags. No Rust-valid
unsupported positive was inverted and no text oracle was generated.

## Core and retained work

Current development CLI SHA is
5d434e3a2eee340faa9172c9e70d6af18b63ecfe4d8251fcaa90451be728ee8c.
A clean source commit and immutable producer, calibrated comparisons and the
unchanged120s original-core observation follow this local checkpoint. The last
real original observation still first fails at RefCell::clone,
cell.rs112:49655,line1449; only a new observation reaching that body can clear it.
No complete census or original core/alloc/std artifact exists. Preserve every
paid-alias, projected-premise, symbolic-index native, depth128, factory, tuple,
bootstrap, transport and optimized requirement in handoff.md.

Backend bridge17b024f69, storage.py, all722 original source hashes, six source
configuration bindings and compiler budgets are unchanged. Disk pressure refused
some launches before execution. Bounded runs use smaller output reservations
within the approved16GiB ceiling after measuring retained output and temporary
copies; the150GiB reserve and compiler/observer limits are unchanged. Raw evidence
is B obj/parallel/refcell-clone; see the machine record beside this document.
