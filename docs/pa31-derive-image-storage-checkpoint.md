# Source derive image storage checkpoint (LIB079)

The unchanged pinned core reached `iter/sources/from_coroutine.rs:38`, byte 1098,
while expanding `#[derive(Clone)] pub struct FromCoroutine<G>(G);`. The prior
qualified CLI was `1b626052c160ccc0f3b485b0b4cb91520f013e21ec4dc5448a8ccfff84ffe814`.
A separately identified diagnostic build reproduced that boundary with all 722
input hashes unchanged and 373 source opens. Opens are not compiled files.

The failing request charged 34 work units and 10,336 attempted bytes in
`derive_output::Output::text`: 304 bytes for each character, even within an
identifier or when vectors had spare capacity. The attempted totals were
644,164 work and67,116,347 bytes. The unchanged limits are 1,000,000 work,
67,108,864 attempted bytes and67,108,864 retained payload bytes. The actual
failure was attempted storage, not work or retained payload. The diagnostic
record had 387 macro declarations, 362 edition rows and 53,959 witness bytes;
witness bytes are not a measurement of the whole retained compiler heap.

The implementation uses existing `retained::Growth` before actual source,
origin, token and copied-field vector growth. Image copy work, lexical scanning,
row initialization and growth copying remain distinct attempted charges. Spare
capacity incurs no repeated allocation charge. Parser scratch is prepaid using
actual token and typed-splice counts; existing owned syntax/output and source
installation charges remain. Typed subtree copying, origin checks, source role
and current-owner authentication are unchanged. There are no new record fields,
permission grants, budget increases, metadata rows or diagnostic exemptions.
The accounting policy remains conservative deterministic payload accounting,
not an allocator heap-size measurement.

The focused private four gate passed in 206.76s on committed-source-compatible bytes,
without Root's temporary Debug array emitter. Its receipt is
`1793257178494829080` (SHA256
`856fadd945735ba5abc2e9bbb5a41e1a1e219e30de226859e241d32f8d70e207`).
It checks spare capacity and failed growth before publication, repeated derives,
long identifiers, sticky exhausted budgets and the earlier genuine source
Default/generic body producer. Node 36 and Body 456 are unchanged.

Fixed provider/import setup is 5,003 work and 263,141 attempted bytes. The deltas
for independently named small Clone derives are:

| Derives | Work | Attempted bytes |
| ---: | ---: | ---: |
| 8 | 11,937 | 690,272 |
| 32 | 47,629 | 2,760,864 |
| 128 | 190,949 | 11,043,808 |

The 128-derive cold total is195,952 work /11,306,949 bytes; authenticated warm
receipts raise work to 222,621 and allocate no additional charged bytes. The
witness charge stays 2,785 bytes across those warm reads. Identifier lengths
64/256/1024 produce deltas 2,284/3,859/10,771 work and 89,708/95,404/123,052 bytes.
These checks separate fixed setup from changed-input costs.

The exact original declaration interval and limits were frozen in
`pa31/tests/spec/derive-image-storage/contract.json` before observations. Pinned
Rust 1.98 expanded the unchanged declaration successfully. Upstream verification
and the exact source interval check pass. Raw diagnostics, failed predecessors,
source maps and receipts are under `obj/parallel/source-macro-budget-v1`.
The first static predecessor had one test-only unused import; the first semantic
predecessor exposed a missing actual Clone macro import in the new test scope.
Both failures are retained. The final test module imports the genuine source
derive and Clone declarations before using the unchanged declaration fragment.

Production/CLI qualification and an unchanged whole-core observation of the
combined current compiler remain Root integration work. A legitimate next work
or storage boundary must remain visible. Complete original coroutine/runtime,
core/alloc/std artifacts, metadata and whole assignment releases remain required.

Root integration passes48 private controls in148.85s and normal production/CLI2
in109.78s. All7 source-native program inputs are byte-identical to their passing
native observations. The resulting CLI8ee48985 preserves original722 hashes,
target flags and all limits, but reports another source work/storage boundary at
StepBy, iter/adapters/step_by.rs512. This observation does not establish that the
whole original macro budget is now sufficient.

The next diagnosis isolates work: 1,000,004 attempted steps exceed the unchanged
1,000,000 limit. Attempted storage is 60,099,511 bytes, below 64 MiB; retained
storage has not failed. The failing seven-step charge scans StepBy's Debug
header. The measured work is 532,351 for generation, 225,626 for panic delegation
and 242,027 for other header/provenance/receipt work. Actual copying, capacity
relocation, token initialization, lexing and parsing remain distinct linear
costs; this diagnosis does not establish another duplicate charge to remove.
Raw diagnosis is B's `obj/parallel/source-macro-budget-v2/diagnosis.json`
(SHA256 `74f1557864d4be8d3c1d3ccdfac0237c375c3d0e940323924bc5410bd113470d`).

The next compilation experiment uses explicit finite resource inputs, retaining
the old defaults and charging every attempted operation. The selected work
ceiling is 14,500,624, equal to the physical bytes of the 373 source files opened
by the predecessor. This is an input-derived experimental allowance, not the
measured total work of a completed crate. Attempted and retained limits stay at
64 MiB. Bind all three effective limits in the manifest, producer and result;
never compare this changed-resource observation as an unchanged-default speedup.

The integrated resource API passes eight private and three CLI controls together
in193.73s, with strict Clippy in both modes. Receipt1793261815925571764 qualifies
CLI `213a3c5711dd99431e49e03236743cf1c0174f3b59afbd798da8e356a3e8ce33`.
The work14,500,624/attempted64MiB/retained64MiB observation takes10.49s and
stops at SocketAddrV4's derives (`core/src/net/socket_addr.rs:79`) on attempted
storage. Its receipt SHA256 is
`7fd06b938bffb51916a6bd35c6f68b8143357d654b98e87c8db1cc1ca073a6b6`.

A separate finite attempted-volume allowance of1GiB, with the same work and
retained limits, takes10.85s and reaches `core/src/escape.rs:161`:
`#[derive(Clone, Copy)] union MaybeEscapedCharacter<const N: usize>` lacks its
actual Clone recipe. Receipt SHA256 is
`2da97d75af6b4c4b51f9a7b701d6bd61249f0787044ef809fc998917c2174899`.
Both observations preserve all722 source hashes and target settings;373 opens
still do not establish semantic completion or an original artifact. Both use
the same immutable compiler and retain their distinct resource bindings. No
default or accounting charge was removed. These are pre-metadata-migration
observations; subsequent compiler checkpoints need their own source binding.
