# Caller contexts, constant memory and C variadic calls

Compiler cutoff `c566bed4ee844832c166257646e0f89f1b949765` preserves checked
associated-constant caller contexts through nested instantiation, transports
symbolic pointer words through constant memory, implements canonical pointer
byte fills, and lowers direct/indirect C variadic calls using actual argument
classes. The detailed contracts remain in the PA25 handout,
[relocation design](pa28-relocation-design.md), [byte fills](pa28-byte-fill.md)
and [variadic calls](pa30-variadic-calls.md). PA25–PA30 remain planned and
incomplete. This checkpoint does not activate an assignment.

## Qualification

The combined production candidate passes 2,491 regular release maintainer tests,
251 explicitly selected native tests, all 8,360 PA1–24 text cases, strict
all-target/all-feature release Clippy, formatting, the architecture/file audit
(1,659 files, zero findings), 90 harness tests, suite registration, layout,
upstream verification and both pinned backend handshakes. No text references
or existing fixture expectations change.

The first full regular run passed 2,489 tests and failed two associated-declaration
controls that invoked the old unprepared semantic/lowering test entrypoint.
Both retained their exact source and expected outcome and now invoke the
existing prepared pipeline helper. The complete affected `suite_projections`
binary then passed all 126 tests. Qualification combines that rerun with the
unchanged other full-suite results; the earlier failures remain in the record.
Production sources are identical before and after the test-only repair.
The complete native suite and all active text cases passed after that repair.

Default and all-feature production builds reuse the identical executable.
Candidate SHA-256:
`e25d01ba2b6fbd352880e4397f4fe2b0aebeb8fe9816d5c2b5353e4ebc6d4963`.
Qualification lives in
`rustgm-validation-work/obj/parallel/memory-caller-variadic-checkpoint-v2/`,
with the original full run retained in the sibling `-v1/` directory.
The passing wrapper reclaimed 12.11 GiB of disposable output.

## Frozen performance comparison

The baseline is the previous public candidate with SHA-256
`f46b8699a71cfb949e217f286f0acf1fc60b4a4af959d5c51dc8361a90667a6d`.
The 67 frozen inputs cover caller uses/identities/instances, observation graph
width and contexts, byte-fill type width/path depth/extent/repeats, symbolic
relocations, C variadic tails/calls and ordinary controls. Forty-two newly added
inputs have independent complete Rust-object qualification; 25 reuse the prior
qualified variadic matrix. Ten inputs succeed under both binaries with identical
stdout. The 57 newly accepted inputs carry no speedup claim.

Each shared case retains six A/A observations, five ABBA blocks with three
executions per observation, eight RSS samples and both Callgrind profiles.
The exclusive lease excludes registered RustGM work, not unrelated host load.
Shared instruction growth is 0.119%–0.525%. The ordinary512 control changes from
43,095,488 to 43,272,423 instructions (+0.411%); its candidate median is
19.21 ms and paired wall change +1.82%. Other shared wall changes range from
+0.97% to +10.94%. All observations remain recorded; these results do not
establish a general speedup or wall-time neutrality.

Three ordinary array controls cross the declared memory trigger: median RSS
increases by 748/754/756 KiB (about 7.3%–7.4%). One fill series crosses the
incremental instruction-ratio trigger. The original results and thresholds
remain unchanged. A single follow-up was declared before running it, using
these controls, ordinary512 and three extended fill inputs.

## Attribution and limits

Massif reports identical total peak heap plus allocator overhead for both
binaries: 191,752 bytes at each of the three flagged array extents and
2,117,520 bytes for ordinary512. Requested heap differs by just eight bytes in
the array controls, offset by allocator overhead. Pages-as-heap increases by
274,432 bytes (268 KiB) for every control. The binary text section grows by
272,296 bytes. This identifies bounded mapped-image growth and finds no growing
heap cost on those axes. It does not account for every byte of the original
resident-memory difference: mapped pages and resident pages are different
measurements. The roughly 750 KiB RSS increase remains a documented cost.

The fill series originally costs 5,420,484 / 5,422,215 / 5,435,779 instructions
at extents 8/32/128, producing an incremental ratio of 7.84 from small
whole-program differences. Independently qualified extents 512/2048/8192 cost
5,487,186 / 5,681,161 / 6,472,417 instructions. Their successive increments
scale by 3.77 and 4.08. Exclusive function costs identify the actual fill work:
`repeat_byte_span` increments 31,104 / 124,416 / 497,664 and `edit_fragments`
increments 11,136 / 44,544 / 178,176, exactly fourfold per fourfold extent.
The original small increment also includes allocator and name-table variation.
These measurements support linear work in bytes materialized, without a
whole-compiler complexity claim. The previously documented generic cleanup
quadratic remains a separate required PA25 follow-up.

The measured instruction costs and bounded footprint are accepted for this
checkpoint with the RSS limitation above. No compiler change or favorable
measurement selection was used to remove a trigger. Original observations,
profiles, declarations and the one attribution pass live in
`obj/parallel/memory-caller-performance-v1/` and `attribution-v1/` below it.
The attribution declaration SHA-256 is
`20c515d680a0e06257a7ff7b7d0da9366ed4436f138f5310c08231963556d21d`.
