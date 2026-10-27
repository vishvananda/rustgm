# Required unsafe-policy warning diagnostics

LIB-047's required warning emission is implemented and development-qualified.
The unsafe-places component and requirement27.1 now include default and scoped
warnings. Component publication, complete fixture inventory and release checks
remain pending; whole PA27 is still planned.

Two original, independently declared controls isolate the gap:

```rust
#![allow(dead_code)]
#[warn(unsafe_op_in_unsafe_fn)]
unsafe fn read(p: *const u8) -> u8 { *p }
fn main() {}
```

```rust
static mut V: u8 = 1;
#[warn(static_mut_refs)]
fn main() { unsafe { let _ = &V; } }
```

Pinned Rust1.98.0 and the immutable integrated compiler both accept each input.
Rust emits the corresponding warning; the candidate emits empty stderr in both
LowIR runs. No unsafe input program was executed. The four commands take0.63s.
The actual raw input bytes, Rust JSON diagnostics, candidate LowIR, predeclared
plan and receipt remain under the root-owned
`obj/parallel/unsafe-warning-gap-v1/` directory. Candidate SHA-256:
`ebd0d4d09f53b2bde0d216be5e5d041ae7f4039ee97786eee3219bb520c781ff`.
Receipt SHA-256:
`c8f3c7aa8461d6926bbba848b327ee2e4ff7c1453dd9a619e57c5187708f79f8`.
The post-observation binding also records actual toolchain88d9e12ae and both
unchanged source hashes; it is not a claim of a new release gate.

The original `bodies/safety.rs` tested only the policy's denies-operation/reference
queries. Warn permission succeeds without entering a warning sink. The phase
warning vocabulary lacks these lint identities. Existing accepted policy tests
prove acceptance and enforce empty stderr; they do not prove warning behavior.

The repair consumes effective Warn/default-warning levels at the existing typed
safety consumers and uses the existing bounded semantic warning sink. Allow is
silent; explicit blocks, safe-context errors, inherited priority/forbid rules,
source failure retention and per-original deduplication remain intact. There is
no new field, policy map, warning owner or attribute reparse. Pinned rustc's
unsafety checker also confirms that allow_internal_unsafe does not mask these
two lint policies; its separate unsafe-code suppression grants no permission.

Twenty-seven frozen ordinary controls independently match pinned Rust acceptance
and required lint counts; six additional Rust controls cover distinct operations,
implicit references, macros, generic reuse and warnings before errors. The seven
affected successful CLI cases now declare exact full warning bytes and counts;
all unlisted successes still require empty stderr. Native declarations also need
explicit reviewed warning codes and exact stderr. No generated oracle was blessed.

Private11 passes in159.45s, receipt
`obj/qualification/fast/1793133945107042994/receipt.json`, SHA-256
`1ff2fee2fe0da2606557220162cbb54fee45fa9b30f2ce32f8d734a99fbd19da`.
Affected unsafe-context/policy/static/volatile CLI13 passes in110.62s, receipt
`obj/qualification/fast/1793134113149020041/receipt.json`, SHA-256
`8dbeac9a90b0bb018c20ca945f35df87800cd9eda970335bb281690ec9afd5ed`.
Their complete source fingerprints match. The tests exercise actual macro
provenance, runtime instances, warning retention, deduplication and failed-work
accounting. A separate same-binary observer measures widths1/8/32/128 at exactly
2 steps and405 retained bytes per warning; Body456 and WarningCode1 are unchanged.
The full harness passes in14.85s. Observer receipt
`obj/parallel/lib047-qualified-v1/receipt.json`, SHA-256
`05f7703315d6555867f897a88f229119fb8427f561e03406f7b0f53bc20ea293`.
Its first captured-index probe used an invalid CLI option and stopped at usage;
the corrected probe alone establishes the separate real implementation failure.

The same inventory found a real prerequisite: the union's canonical
`std::mem::ManuallyDrop<S>` fixture needs the already named PA26 operator-kernel
nominal producer. Operator-kernel has no dependency on unsafe-places; moving it
before unsafe-places and adding that edge is acyclic. The real positive and fake
identity opponent both remain required. Component publication must also await
its declared prerequisites, complete fixture inventory and actual release gates.
