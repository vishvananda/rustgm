# Wide literals and the next original-core boundary

Isolated integration candidate `cd8405bd` passes eight exact private controls
in [CI 40196564678](https://github.com/vishvananda/rustgm/actions/runs/40196564678):
six wide-literal controls, the genuine Functions codec/client control and the
source attribute publication guard. Metadata, formatting, architecture, strict
private Clippy, build and execution take 170.27 seconds. Reference-effect review
and text qualification are complete. Long release/native qualification remains
separate; this checkpoint does not activate an unfinished assignment.

The qualified dev-fast CLI has SHA-256
`da5d43430556c8b9dc451de3ebe179005b4b1b843246e54af100c8ab833a6f76`.
Its [unchanged-core observation](https://github.com/vishvananda/rustgm/actions/runs/40197046689)
verifies all 722 pinned inputs, opens 42 original Rust files and fails at
`library/core/src/num/nonzero.rs:34`, byte 1279, on
`pub impl(self) unsafe trait ZeroablePrimitive`. The observation takes 7.116
seconds; source bytes and compiler remain unchanged. No crate artifact is
produced. The preceding qualified compiler opened nine files and stopped at
the `_f128` literal in `num/f128.rs`.

The last opened source has SHA-256
`9fee2d234130bf627fd3ec16cdebe8911653886d54c34dcb0601f7dfc2fb9de3`.
The loader assigns its next real unit FileId before parsing. Error FileId 41,
42 unique Rust source opens and the physical token offset agree. The next
family is `impl_restriction`, enabled by the original root at line 134. Ten core
trait declarations use self/crate and const/unsafe combinations. The pinned
compiler and `tests/ui/impl-restriction` supply grammar and semantic opponents;
[RFC 3323](https://rust-lang.github.io/rfcs/3323-restrictions.html) describes the
distinction between trait visibility and permission to implement it.
Parsing the modifier without enforcing its implementation scope is insufficient.

One bounded exact rational/sticky kernel implements the literals, adapted with
attribution from frozen CPPGM. Existing f32/f64 conversion paths remain intact.
Forty independent language observations and 64 exported-static object-bit
comparisons pass; these are not native execution of wide arithmetic. Genuine
feature and macro-origin checks apply to active literals and resolved primitive
types. Later wide operations, ABI, metadata and runtime retain explicit limits.

Measured hot record sizes remain unchanged: FloatBits is 24 bytes/alignment 8,
Scalar 32, NodeFacts 80, Body 448 and LowIR Value 12. Two canonical primitives
add 56 bytes to initial type/property vectors; hash capacity stays 28. Long
decimal inputs retain at most 2048 scratch words in the measured series.
Ordinary validation work still needs review; layout assertions do not establish
wall-time nonregression.

A bounded ordinary-f64 screen retains 40 observations over 256/2048-function
inputs with immutable pre-wide/current binaries, six A/A observations and three
paired ABBA blocks per input. All outputs are identical. The larger input's
paired baseline means are 0.416/0.422/0.458 seconds versus candidate
0.418/0.451/0.465 seconds; A/A observations range from 0.421 to 0.475 seconds.
These noisy small samples do not establish a regression or a speedup. Raw
observations, inputs, compiler hashes and timings remain under
`obj/parallel/ordinary-wide-screen-v1`; the complete screen takes 10.84 seconds.

All 139 old PA2 cases are byte-identical; PA9's 91 negatives preserve status
and diagnostics. Its 109 positive outputs require explicit typed-field mappings
for new types/primitive definitions, later IDs and table totals. A read-only
effect run completes all 8478 active text cases in 51.79 seconds, reusing the
authenticated earlier 339. Every exit status and all 4902 LowIR outputs are
byte-identical. Review accounts for every changed semantic/name output through
the actual renderer-owned fields; it performs no blanket numeric replacement.

All six changed must-use stderr streams are byte-identical in the immutable
pre-wide compiler; the baseline comparison takes 0.913 seconds. Another 106-case
baseline comparison takes 21.326 seconds and proves every changed pattern-work
counter also predates this feature: 107 match rows and their EOF totals agree
between the binaries. These are genuine earlier attempted-inhabitation charges,
not type IDs. Keep both classes distinct from the wide feature's new rows and
later-ID shifts when reviewing oracles.

Checked generation updates 1813 stdout channels and six stderr channels in
1816 selected cases. Every actual channel must match the frozen reviewed SHA-256
before the standard generator writes that stage. The 106 predecessor-work cases
are a subset of the stdout changes. Explicit unused_must_use manifest codes make
the six previously stale successful-warning cases valid native frontend inputs;
six actual pinned-rustc metadata observations confirm their warnings. Status
channels and all LowIR bytes stay unchanged. The complete guarded generation,
8478-case text pass, metadata, 212 harness tests and upstream checks pass in
73.99 seconds; the text pass itself takes 41.96 seconds. Evidence is retained
under `obj/parallel/wide-reference-publication-v1`.

Six public PA2/PA9 fixtures then add half tie-even rounding, the original quad
PI spelling, integer-spelled half max-finite, enabled widths, a missing gate and
a width mismatch. All six match actual pinned-rustc metadata outcomes; the
literal text bits also match the independently observed exported-static bytes.
Fresh complete PA2/PA9 suites pass 142/203 cases. Their generation/check,
metadata and harness/upstream batch takes 19.31 seconds without a compiler
rebuild. Together with the unchanged 8139 cases retained from the preceding
pass, this qualifies 8484 text cases. This is not another fresh all-stage run
or native execution of wide arithmetic. Evidence is under
`obj/parallel/wide-public-fixtures-v1`; PA2's old runtime float adapter is intact.

Periodic [full CI on `fb6cc056`](https://github.com/vishvananda/rustgm/actions/runs/40204216976)
finds five stale private/integration expectations: the cold feature word is now
four bytes and source types allocated after the two new primitives have later
IDs. The repair changes assertions only. Exact reruns pass all three private
layout controls and both semantic relationship controls, preserving their
owner, lifetime, member-selection and ordinary-allocation checks.

The initial combined cold gate reaches its build cutoff after 275.88 seconds;
metadata, formatting, architecture and both strict Clippy modes pass, but no
tests execute. A source-identical continuation retains those five checks using
all 2572 recorded content fingerprints, completes the missing build in 131.69
seconds and executes the five exact tests in 0.44 seconds. Both receipts remain
under `obj/qualification/fast/1792943923078260997` and
`obj/qualification/fast/1792944422675923364`. The failed periodic result remains
failed; later release/native/debug qualification is pending. Compiler behavior
and committed text oracles are unchanged by this repair.
