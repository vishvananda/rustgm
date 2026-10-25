# Wide literals and the next original-core boundary

Isolated integration candidate `cd8405bd` passes eight exact private controls
in [CI 40196564678](https://github.com/vishvananda/rustgm/actions/runs/40196564678):
six wide-literal controls, the genuine Functions codec/client control and the
source attribute publication guard. Metadata, formatting, architecture, strict
private Clippy, build and execution take 170.27 seconds. Public integration
and reference-effect review remain pending.

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

All 139 old PA2 cases are byte-identical; PA9's 91 negatives preserve status
and diagnostics. Its 109 positive outputs require explicit typed-field mappings
for new types/primitive definitions, later IDs and table totals. A read-only
effect run completes all 8478 active text cases in 51.79 seconds, reusing the
authenticated earlier 339. Every exit status and all 4902 LowIR outputs are
byte-identical. Other semantic/name output changes remain under review. Six
changed stderr streams introduce actual must-use warnings and require comparison
with the immutable pre-wide compiler; they cannot be approved as ID shifts.
No references have been regenerated or approved. Full release/native
qualification remains separate.
