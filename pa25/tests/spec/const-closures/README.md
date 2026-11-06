# Const closure contracts

The original four sources and `contract.json` retain their independently observed
creation, feature, constant-call and ordinary-call outcomes. The semantics
successor adds actual captures, Fn/FnMut/FnOnce, conditional caller and cleanup
premises, const arithmetic, source loans/moves, lifetime escape and real Drop.
All predictions were hashed before each independent observation.

`conditional-call.rs` disproved its positive prediction with genuine E0493:
its ordinary generic cleanup prerequisite was missing. The separately frozen
v2 adds the actual conditional Destruct premise and accepts. This is an ordinary
negative and correction, not an infrastructure failure.

`source-execution.rs` preserves the exact authored support prefix of
PA31's `inherited-overflow/const-closure-v3.rs`. Its successive missing
const_destruct/drop_glue dependencies are retained. The v3 adds only the already
vendored original `cstr-source/original-drop-glue.rs` compiler-owned helper.
Metadata accepts; object generation requires panic_const_add_overflow. There is
no no-core native claim, empty authored replacement or disabled overflow check.
The original Iterator count excerpt remains in PA31's inherited-overflow packet.

`semantics-native.rs` executes through installed pinned Rust and std. Runtime
input 23 yields `closure 23 26 23 1 25`: checked CTFE shared/mutable/consuming
captures, one actual const Drop, and ordinary execution of a const-fn-produced
closure. This does not establish the no-core panic family or RustGM execution.

`semantics-manifest.json` lists all source and auxiliary bytes.
`semantics-reference-v1.json` binds the immutable predictions, raw receipts,
diagnostics and native execution. No historical source or generated oracle was
changed. These reference-only successors do not activate an unfinished course
suite or complete PA25, PA31, closure transport or original library compilation.
