# Checked scalar separate compilation

The initial PA29 slice now connects source exports, deterministic binary metadata,
explicit import remapping and native library linking. One parse and ordinary
analysis feed `sema::crates::export`, shared `lower::Roots` and the existing
proof/lowering pipeline. The driver writes the metadata sidecar after lowering
succeeds. A private invalid body prevents publication; no imported function gets
an invented AST node, initializer, CFG or const execution permission.

The library CLI is `--emit-lowir --crate-type=lib --crate-name NAME
--crate-disambiguator ID --metadata-output FILE SOURCE`. `--extern ALIAS=FILE`
supplies each dependency's metadata; the complete graph must be available.
Arguments are bounded to 1024 aliases and 16 MB of artifact input. Repeated aliases
to the same canonical file share its decoded artifact. Duplicate identities from
different artifacts, missing dependencies and cycles fail before resolution.
There is no metadata-only CLI or rustc metadata compatibility claim.

Public source bindings define the export table. Unsupported exposed declarations
fail explicitly: generic/const/async functions, target features, non-Rust ABI,
non-scalar types, public reexports and empty public modules. Private declarations
remain checked; private generic instances still use the existing specialization
pipeline. Scalar tag zero permits a unit result, not unit parameters. The fixed
schema/profile meanings and bounded wire contract are in `pa29-scalar-metadata.md`.

`sema::crates::Linkage` retains external Rust spellings and one explicit local
crate identity. `lower::Context::rust_name` serves both ordinary and imported
Rust naming callbacks; private specialization suffixes remain distinct.
`internal_name` prefixes generated functions, closures, drop/remainder/virtual
helpers, tables and data only when a local identity is installed. Exact symbols
and external runtime names keep their spelling. The ordinary no-identity path
keeps legacy names; `--crate-name` alone produces unchanged output. Canonical
`Program::declare/define` from the PA30 checkpoint shares compatible spellings
and rejects conflicting declarations or repeated definitions.

The pinned executable LowIR reader does not merge a declaration with another
unit's definition: an actual attempted multi-file link diagnosed duplicate
top-level symbols. That failed capability probe is retained. Multi-input
`backend.build_many` therefore requires the existing reviewed optional ELF object
profile: validate each LowIR unit, compile one object per unit, then invoke the
host C linker once. No declarations are stripped or rewritten. Single-input
`build` retains its prior executable commands and validated output path; object
mode still requires exactly one serialized input. Missing/empty inputs fail.

The object compiler was copied from the PA26 immutable pin into this worker's
budgeted `obj/backend/pins/` and verified before installation in backend.local.
Its SHA-256 is
`ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
The reviewed profile remains backend.lock.json optional.object; the live CPPGM
checkout was not reconfigured or changed. `--test-object` passed before native
qualification, and the original executable smoke remains passing.

Two independently compiled libraries deliberately share the crate name `shared`,
public leaf names, private helper names and private data patterns, while using
different disambiguators. Their native caller checks nested exports, two private
generic instances, function-pointer reification and u8/u32/i128/bool/char/f32/f64
values. A second caller imports an ordinary Rust function as main. Both execute
with empty output and exit zero, agreeing with independently built pinned rustc
libraries/callers. Reordered extern arguments preserve LowIR bytes; repeated
library compilation preserves metadata and LowIR bytes. Every scalar tag also
passes checked source export and codec round-trip controls.

Check, 23 focused tests including the native group, strict all-target/all-feature
Clippy, the architecture/file audit and 54 harness tests passed. Eight independently
Rust-valid profile boundaries and one Rust-invalid unused-body control were object
qualified. The inherited PA12 receipt mismatch was resolved by taking integrator
commit 6eb857fa unchanged; layout and upstream verification then passed. Initial
test-style lint findings and the crate-test filename ownership finding were fixed;
failed logs remain available. No references were regenerated in this lane.

Evidence is `obj/parallel/pa29-linking/`, including native outputs under
`obj/parallel/pa29-crate-linking/`, boundary qualification, object-pin provenance,
gate logs and source manifests. The dev-source manifest SHA-256 is
`efbc681636fd6b42d97677610977d5939feacbd74d521d3fac17750671d0950c`.
The final layout/upstream receipt is `aed425a0cab342aa8dd961632f0efe40`, exit zero.
Functional/native/lint/audit/harness passed in the preceding ownership batch;
its exit one was only the subsequently repaired inherited receipt mismatch.

This checkpoint does not complete PA29 or PA25. Generic/nominal/lifetime/closure
metadata, const permissions with checked imported bodies, values/allocation graphs,
library packed-pointer startup initialization and assignment release contracts
remain unfinished. PA25's prior scalar RSS observations (+594 KiB and later
+264 KiB ordinary control) remain unresolved. Changed-path performance measurement
for this slice is pending the integrator's priority initializer correctness repair.
