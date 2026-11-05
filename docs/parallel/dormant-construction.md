# Dormant construction and genuine future cleanup prerequisites

This checkpoint does not qualify executable future scope exit or complete async
runtime. The Rust-positive construction fixture deliberately remains an explicit
generated-Drop gap. Poll, capture transfer and state-specific cleanup are required.

The actual public async function can now produce a distinct generated constructor
without materializing or running its deferred source body. The first factory
profile accepts only zero inputs, empty complete captures and zero public binders.
It initializes the real future's state byte, returns that owned future, and keeps
the checked AsyncBody and its Output contract in their original owner. A private,
non-Clone/non-Copy permit authenticates the real FunctionItem, source, current
arena, full future, public/execution signatures, exact storage/CFG and owned return.
Existing region, Copy, initialization and loan proof runs before its permit moves
once through the existing forwarding map. Unsupported captures, binders and
formation requirements still fence. Actual Permit32, outer factory Permit16,
Bridge16 and Recipe32/Method8 compile against the real definitions; Owner20,
Body456, Kind24 and Builder1776 are unchanged.

Testing exposed a genuine cleanup bug: ordinary collection activated NeedsDrop
only when a synchronous destructor existed. A parameterless unpolled future could
therefore reach native code without a cleanup plan. That earlier executable
returned0 and preserved its divergent deferred body, but is infrastructure evidence
only, not correct construction/scope-exit parity. Its exact source, LowIR/backend
hashes and observation receipt are retained. No output or source was reblessed.

The repaired path completes opaque storage only from the existing checked async
owner/template, source region proof and current arena. It uses the producer's
actual last ordinal and existing infer/close operations, walking the existing
owner index once with paid temporary storage; generic layout remains deferred.
Ordinary empty-index entry remains constant time. This publishes storage data,
never a body/factory permit. Concrete cleanup activation now accepts a genuine
authored future anchor independently of synchronous Drop definitions, preserving
ordinary body construction, ownership scopes and the existing Pending/plan path.

A future recipe records Method::Future(full TypeId) under the existing structural
key, preserving complete capture/lifetime identity. Its generated Drop operation
owns capture cleanup; no parallel field recursion duplicates it. Cold/warm tests
prove separate applications retain separate recipes. Dependency selection and
native cleanup both explicitly require the actual checked generated Drop entry.
The previously accepted construction now fails at that requirement before native
publication. No empty destructor, invented Instance or source feature gate was added.

Gate1793910638041086958 passes23 exact public tests across4 targets in125.91s,
including all stateframe tests, six compiler recipes/legacy byte controls, scalar
dispatch and three atomic controls. Metadata, formatting, architecture and strict
selected Clippy pass. Upstream verification passes29 pinned fixtures and126
Reference files. Earlier gate1793910094371881903 passes14/128.31s; its subsequent
native0/3.36s is the explicitly unqualified pre-cleanup observation. Independent
unchanged standard-core companions pass0/native0 and1/E0308. The earlier routing,
test-reader, layout-estimate, lint and storage-admission failures remain in obj.
No new cold private link or performance matrix was run. Adjacent evidence binds
the exact source, fixture and receipt hashes.

Next work must consume actual selected caller/formation and checked application
proof, retain its application-to-execution coordinates and specialized environment,
then prove a real generated Drop/poll family. Constructor inputs move once; body
and parameter-pattern effects wait for poll. Poisoned-before-transfer, exact
FutureField/FutureTake provenance, source Copy obligations, actual Ready/Output
move, cleanup-before-Complete, repeat/completed poll, unpolled/completed Drop,
borrow/capture order and native observers remain required. Full await, suspension,
saved locals/self references, panic/unwind, cancellation, recursive async cleanup,
compiler async_drop_in_place execution and authored-body metadata remain required.
Symbolic recipes grant no runtime authority, and core need not request async_drop.
