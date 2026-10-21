# Closed scalar named constant metadata

Schema one profile zero keeps its original exact function-only encoding. Profile
one retains that function table and appends a nonempty canonical named-constant
table. Target/language IDs remain Linux x86-64 and Rust 1.98.0/edition 2024. An
empty profile-one table, unknown profile or scalar tag, noncanonical order,
truncation and trailing bytes are rejected before inference.

The added table starts with a little-endian u32 count. Each entry contains owner
ordinal zero (u32), Value namespace tag one (u8), Constant kind tag two (u8), scalar
tag (u8), the existing bounded path encoding (u16 count and u32-length UTF-8 names)
and a little-endian 128-bit payload. These tag assignments are fixed. A constant
has no runtime linkage, symbol-pool ID, source node, initializer body or callable
permission. External identity is the validated crate identity, semantic path and
constant kind. Module prefixes remain explicit resolver identities derived from
those paths.

Integer payloads use the type width's two's-complement bits and zero upper
padding. Unit requires zero; bool permits only zero/one; char requires a Unicode
scalar value. Floating values retain every bit, including signed zero and NaN
payloads. Both constructors and decoding validate these rules. Function and
constant tables share the 100,000-row quota, including the complete dependency
graph, and existing artifact/atom/name budgets. Cross-kind value-namespace name
collisions and foreign owner identities fail before remapping. Canonical sorting
is bounded; subsequent remapping and query dispatch do not scan an initializer
table per imported value.

The producer consumes completed public closed scalar constants from the existing
value reader after required evaluation. Unsupported exposed arrays, references,
nominals and function-valued constants are explicit profile boundaries. Public
const/generic function bodies remain unsupported exports. Metadata is published
only after the ordinary checked pipeline succeeds.

Import remapping installs source-free `DefKind::Const` identities, completed
types and validated values. Imported membership is explicit; no fake initializer,
AST or runtime global is created. Ordinary expressions, patterns, discriminants
and const arguments read the same completed value. Borrowing a constant does not
turn it into imported static storage. A scalar value cannot authorize a function
call, expose a private producer body or grant const execution to an imported
ordinary function.

The three `pa29/tests/spec/values/` inputs are declared in the maintainer outcome
inventory. Two separately compiled libraries and their caller independently build
with pinned Rust and execute through RustGM's reviewed object bridge. Controls
cover every integer width, unit/bool/Unicode char, runtime signed-zero/NaN
observations, imported aliases and patterns, a discriminant, const-generic bounds
and reified runtime calls. Reordered dependency arguments preserve LowIR bytes;
repeated production preserves metadata and LowIR. Four independent negative
consumers reject nonconst calls, calling a value, a type mismatch and private-body
access. Codec-only cases additionally retain arbitrary NaN payloads exactly and
reject invalid bool/char/padding bits, quotas and malformed wire forms.

At 8/32/128 imported values and ordinary consumer roots, each imported identity
has one ready value; only the N source roots have initializers. Validation visits
and required demands are N, expression CFG execution is zero and indexed lookup
work is at most `3*N+4`. The test makes no claim that canonical sorting is linear.

Formatting, 25 focused tests including native controls, strict all-target/all-feature
Clippy, the 1270-file zero-finding audit, layout and upstream checks pass. Evidence
is `obj/parallel/pa29-scalar-values/`, final numbered logs and resource receipt
`93e648af827e4f379c91aef741ae8e61`. The dev-source manifest SHA-256 is
`b3cb9ef2f4eac5b03b5bc8da34eff0647aab10b0ae5e532d47e62afbea10f9a5`.
Native evidence is under `obj/parallel/pa29-crate-linking/values-*`. The initial
pattern-path failure, two filename ownership findings and lint repairs remain in
the logs. The inherited combined scheduler lint was repaired by integrator
commit d510d0e1; node-query dispatch was separated naturally in this checkpoint.
No worker references were generated or changed.

The earlier immutable function-profile performance inputs and binaries remain
pinned under `obj/parallel/pa29-performance/`; the exclusive run was canceled
before admission at integrator request and produced zero observations. Changed-path
timing for this slice remains pending coordination. Previous unexplained scalar
and ordinary RSS observations (+594 KiB and +264 KiB) remain unresolved.

PA25 and PA29 remain incomplete and planned. Checked generic/const-function bodies,
nominal/lifetime/closure metadata, aggregate/reference allocation graphs and
library packed-pointer startup initialization still require implementation and
qualification. PA25's deterministic constant view and inline-const source bodies
are the next implementation priority.
