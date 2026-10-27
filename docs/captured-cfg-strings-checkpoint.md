# Captured cfg strings and atomic target gate

LIB-044 now consumes captured cfg string values from the existing typed literal
root. Stored strings borrow their actual literal pool; genuine deferred literals
use the existing interpreter. Literal-root expressions and metadata forwarding
are supported. Parenthesized/arbitrary expressions, byte strings and nonstring
literals remain rejected. No rendered source is reparsed.

The fixed target has bare `target_has_atomic_load_store` and values
8/16/32/64/ptr, but not 128. The actual `cfg_target_has_atomic` feature is slot32
in the existing cold `Features(u64)` bank. `MacroFeatures(u8)` is unchanged.
The existing source/ordinary macro owners retain the request and authenticate
per-gate request or allowance. Truth never grants feature permission. Demanded
operands, false predicates and removed-owner provenance retain their existing
ordering; unselected nested attributes are not evaluated.

Before implementation, 26 frozen programs in editions2021/2024 produced52/52
predicted outcomes with pinned Rust1.98.0. Stable `--print cfg` omitted the
unstable fact; actual bootstrap `-Zunstable-options --print cfg` supplied the
bare fact and five widths. Source feature permission remains separate from this
independent oracle environment. Evidence is under
`obj/parallel/lib044-captured-cfg-v1/oracle-v2`; the initial runner failure before
fixture observations remains in `oracle-v1`.

The d841 gate passed format, metadata, global architecture audit, production and
private strict Clippy, build, ten lexer controls and six of seven private
controls. The remaining reducer failed on its actual retained `#[unstable]`
without `staged_api`; the failed overall receipt is retained. A fixture-only
correction supplied pinned core's actual request and stable crate annotation.
The missing-only continuation authenticated the same2923 source fingerprints
and binary and passed that one selector. No compiler/test code was changed or
rebuilt for the continuation. The exact core macro and first11 invocations now
retain nine active impls and remove the two128-bit impls. This is an expansion
reducer, not a core artifact or atomic runtime proof.

Generated gate widths1/8/32/128 retain one capture parse and produce exactly
1/8/32/128 uses,12/33/105/393 nodes and16/128/512/2048 use bytes. Existing
allowance widths8/32/128 with1 versus32 calls add1131 work for UB and1379 for
the atomic gate independently of owner width. A thousand stored-string reads
borrow without pool growth; deferred strings and invalid root/range opponents
also pass. These are bounded structural/work observations, not timing claims.

Measured layouts: `Features`4→8, `MacroFeatures`1 unchanged, `CfgGate`0→1,
`CfgUse`8→12 and `ConfiguredUse`12→16 bytes. The cold source client grows80→88.
Source184, Local128, Contracts104, Core176, Package256, Report3040,
Definition136, Session16 and Context16 remain unchanged. No new hot token/IR
field, cache or proof owner was added.

Exactly25 PA8 and two PA3 reference cases remain pending root-owned generation.
PA3 computes facts before PA8 gate authorization, so its two paired cases
intentionally accept while their independent Rust program and PA8 cases deny.
The manifest preserves that distinction. No references, fresh core probe,
release build or broad sweep were run by this checkpoint.
