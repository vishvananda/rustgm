# Compiler language-item declaration contracts

PA31 owns source declaration identities and targets. Their builtin consumers
remain with the actual trait, layout, runtime, constant, debug and metadata
assignments. PA31 remains planned: this packet is a scoped specification and
maintainer contract, not an activated grading lane or a library artifact.

`contract-v5.json` freezes 100 additional raw-source names against Rust 1.98.0
revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`. These are names found in
opened original-source files, not a count of cfg-active blockers. Existing
language ordinals 0–86 remain unchanged; the appended inventory follows the
pinned table's order. Diagnostic-owned Send still has no language spelling.

The 135 independent source cases check genuine declaration targets, written
generic counts (including associated owners and lifetimes), required versus
provided trait methods, foreign-only memory hooks, duplicates, features and cfg.
GenericRequirement::None imposes no count restriction. The six weak-only memory
hooks require foreign declarations; compiler_move/compiler_copy require ordinary
functions with exactly two written formals. Declaration recognition grants no
builtin implementation or completed trait proof.

The profiling hooks retain ordinary source bodies. The native observer prints
`9 11` by actually invoking both hooks. Treating either invocation as a no-op
would fail. The current backend adds no profiling annotations; authenticated
debug/metadata transport and any future instrumentation remain required.
Other newly inventoried roles explicitly stop semantic analysis until their
actual builtin contracts exist, including UnsafeCell's Freeze interaction,
AsyncDrop, pinning, field reflection and runtime memory hooks. Old metadata
profiles reject unencoded contracts; source recognition cannot bypass them.

The private test lane consumes cases-v5.tsv and roles-v5.tsv, whose hashes are
bound by the JSON contract and qualification. No JSON library is needed by the
frontend or test harness. The resolver reads existing configured AST items,
resolved kinds, foreign ownership and formal lists once; it neither reparses
source nor adds data to ordinary nodes, bodies or semantic records.

Earlier contracts and authored setup sources remain intact. v1 exposed a pinned
rustc ICE for a malformed UnsafeCell setup, not a language rejection. v2 repaired
that setup but exposed a case-folded Range/range filename collision. v3 assigned
unique ordinal paths and observed seven weak-policy prediction differences.
v4 records the reviewed weak-only/nonweak rules and passes all 126 observations.
v5 adds nine focused controls and passes all 135. The independent diagnostics,
commands, hashes and failed predecessor observations remain under
obj/parallel/compiler-language-claims-v1/oracle-v1 through oracle-v5.

Original core/alloc/std artifacts, builtin semantic completion and full assignment
release checks remain required. A declaration test or an opened source file is
not a compiled library.
