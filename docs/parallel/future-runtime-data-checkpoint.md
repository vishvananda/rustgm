# Authored future storage prerequisite

This intermediate checkpoint implements source-authenticated type/storage data,
not an executable future. The actual constructor, checked deferred AsyncBody,
generated poll/drop entries, Ready/completion and native consumers remain required.
The existing executable state-machine fence remains active.

Actual Poll/Ready/Pending/Context declarations complete through the ordinary
nominal/member producers. Independently valid unused alternatives remain accepted;
ordinary invalid bodies still fail. Exact demanded polling ABI validation is the
next consumer, distinct from declaration admission.

`Kind::Future` retains the existing actual opaque identity and complete arguments.
Its prepaid shape stays in the existing opaque row, with one derived application
memo. A shared capture folder transforms all raw input fields and declared Output
once per complete application. Logical declared Output remains distinct from the
checked authored body's internal return, including nested opaque results. Neither
the poll Context nor Output is an owned capture. Layout uses a genuine anonymous
state/input sequence; it does not invent a tuple, closure, AST node or DefId.

The data queries preserve !Unpin/!Copy/!Clone, conditional capture Send/Sync and
owned capture liveness/destruction requirements. They authenticate applications
on warm reads and reject forged arity/kinds. They grant no Future trait method,
checked original, generated family or executable readiness. Compiler-owned
AsyncDropInPlace remains excluded from authored shape publication.

Gate `1793875308394627846` passes architecture, formatting, strict Clippy, build and
14 exact public tests. It includes the existing22 async signature cases,14 Future
source cases,4 polling source cases, real layouts, full generic cold/warm captures
and argument forgery controls. Actual Kind24/Owner20/Body456 sizes are unchanged.
The old opaque row is actual Declaration4; the optional shape handle changes the
row to16. An earlier expectation3 failed and is retained, corrected from the real
type measurement. Metadata allocation accounting charges the actual new row.

The frozen standard-core packet retains13 static outcomes and8 independent native
traces, with Pending/cancellation traces preserving later requirements. A further
exact-body shape companion passes pinned Rust1.98.0. Its raw no_core attempt stopped
at the missing genuine ResumeTy prerequisite; that is not semantic parity. Raw
observations remain under `obj/parallel/future-runtime-v1/`; exact fingerprints and
gate paths are in `future-runtime-data-evidence.json`. There is no candidate native
future evidence at this checkpoint.

Next approved joins preserve single owned body preparation: distinct original
AsyncBody and transient AsyncApplication owners/keys, actual internal Output
signatures, cold CallerInstance::AsyncBody for the selected complete capture frame,
and genuine generated FutureEntry poll/drop operations. Real checked source
publication reuses inferred/hidden plus close/wake; factories still require the
actual Checked template and contract. No authored async marker export is permitted.
Full await, saved locals, self references, panic/cancellation, recursive async
cleanup, generic/runtime qualification and real body/region metadata remain work.
