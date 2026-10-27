# Required unsafe-policy warning diagnostics

LIB-047 is a PA27 unsafe-places implementation/release blocker. Permission and
deny/forbid behavior are qualified, but required warning emission is missing.
The unit and requirement27.1 are therefore partial. No acceptance predicate or
required positive is removed to retain an implemented label.

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

`bodies/safety.rs` currently tests only the policy's denies-operation/reference
queries. Warn permission succeeds without entering a warning sink. The phase
warning vocabulary lacks these lint identities. Existing accepted policy tests
prove acceptance and enforce empty stderr; they do not prove warning behavior.

Implement actual effective Warn/default-warning diagnostics at the existing
typed safety consumers, using the existing bounded semantic warning sink and
actual source/expansion provenance. Keep Allow silent, explicit unsafe-block
satisfaction, inherited priority/forbid rules, source failure retention and
per-original deduplication. Preserve the no-policy allocation contract; do not
add another per-node policy map or reparse attributes. Independently declare
diagnostic and native warning expectations before generating component oracles.

The same inventory found a real prerequisite: the union's canonical
`std::mem::ManuallyDrop<S>` fixture needs the already named PA26 operator-kernel
nominal producer. Operator-kernel has no dependency on unsafe-places; moving it
before unsafe-places and adding that edge is acyclic. The real positive and fake
identity opponent both remain required. Component publication must also await
its declared prerequisites, complete fixture inventory and actual release gates.
