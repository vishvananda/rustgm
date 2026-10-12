# PA8 implementation and validation

PA8 completes configured crate expansion and ordinary name resolution. Its 87
complete-program fixtures have independently declared success/rejection outcomes
and generated stdout/stderr/status sidecars. Every fixture agrees with pinned
Rust 1.98.0; no phase-boundary acceptance differences are declared. PA1–PA8 now
contain 1,106 active fixtures. PA9 types, PA10 bodies and PA11 Rust-to-LowIR
lowering remain necessary for the active frontend goal.

## Owned architecture

| Responsibility | Implementation |
| --- | --- |
| Expansion worklist, scopes, import dependencies, generated modules | `resolve/expansion`, `expand/textual` |
| Source/template/capture origins and persistent opaque fragments | `expand/stream`, `expand/registry`, `syntax/interpolation` |
| Mixed-site local/label identity and final ordinary names | `resolve/hygiene`, `resolve/index`, `resolve/expansion/paths` |
| Typed scalar builtins and iterative token stringification | `expand/builtin`, `expand/stringify` |
| Eager argument parsing and dependency completion | `syntax/expression/fragment`, `resolve/expansion/eager` |
| Environment/resources/source paths through explicit providers | `resolve/load`, `resolve/expansion/resources`, `dev/bin` |
| Cached included fragments and transactional typed installation | `resolve/unit`, `resolve/unit/expansion`, `literal/storage` |
| Physical diagnostics with separate invocation ancestry | `resolve/expansion/diagnostics` |
| Deterministic text views, never semantic transport | `resolve/render` |

Included files use the ordinary bounded source parser and normalized source
storage. Cache keys contain canonical identity and grammar context; requested
aliases retain their own names. Repeated insertions borrow the cached syntax,
copy typed nodes and decoded bytes once into the receiving unit, and expand
independently. Cached templates are marked in the view and are not additional
module bodies. Module directory ownership follows the included subtree and
then restores the caller's directory. Includes preserve source hygiene; cold
error projection separately recovers their invocation ancestry.

The production library remains safe Rust with no ambient filesystem/environment
reads or frontend subprocess fallback. Strict Clippy all/pedantic and the
107-file architecture audit pass. Literal storage now accepts borrowed values;
its narrow `ptr_arg` expectation documents why the Cow ownership discriminant
must distinguish original source ranges from decoded storage. No dependencies
or broad lint allowances were added. Function and file limits remain enforced.

## Independent and structural evidence

The final debug suite has 183 Rust invariants, including 57 crate-expansion
checks. They cover capture reuse, hygiene identities, worklist dependencies,
configuration exclusion, source locations, decoded values, scope restoration,
read/parse counts, recursive inclusion and ordinary-name error traces. Release
invariants pass. The 16 harness checks validate discovery, exact bytes, declared
outcomes, atomic blessing, infrastructure failures, file paths and explicit
child-process environment values.

All 87 PA8 cases pass `scripts/check_syntax.py pa8`. The new source-include probe
matrix additionally checks 19 contexts against both default and suppressed
rustc lints. A trailing expression include is a default-deny lint in rustc;
that fixture explicitly requests default lints. Trait/impl/foreign-item, type,
pattern and statement-list includes are rejected; statement-position includes
accept one expression. Shebangs are removed only for module-item includes.
The included-source hygiene cases independently confirm that included names
can see caller locals but cannot see macro-template locals.

The earlier detailed PA8 investigation and independently evaluated stringify,
concat, environment and resource values are recorded in `pa8-progress.md`.
During suite assembly, rustc rejected an initially misclassified captured-name
fixture; its expected outcome was corrected before any reference was generated.
An oracle review also exposed a missing include trace on final ordinary-name
errors; the implementation and a typed regression were fixed before publication.
Earlier PA1–PA7 fixture/oracle bytes remain unchanged. Their refreshed receipts
contain only compiler-source fingerprint changes. Upstream checks retain all
24 imported inputs and 126 vendored Reference-file hashes.

Local evidence is retained under `obj/pa8-work/`: `pa8-independent.log`,
`pa8-full-validation.log`, `pa8-reference-validation.log`, `pa8-release-tests.log`,
`include-diagnostic-tests.log`, `include-diagnostic-audits.log`,
`pa8-final-references.log`, `pa8-final-tests.log` and
`pa8-final-release-expansion.log`. Per-fixture independent diagnostics are under
`obj/rustc-syntax/pa8`. Raw measurements and immutable binaries remain in `obj/perf`.

## Performance observations

Measurements ran sequentially on CPU 10 with frozen binaries/inputs, complete
output equality, A/A calibration and three ABBA blocks. Baseline `9c177c7`
only registers the unused include builtin name so existing full-view IDs match;
its exact patch is `obj/pa8-work/include-baseline-prelude.patch`. No benchmark
asks the baseline to implement the new builtin. All observations are retained.

| Workload / calls | Baseline ms | Candidate ms | Baseline / candidate peak KiB |
| --- | ---: | ---: | ---: |
| Full expansion / 4,000 | 83.09 | 82.01 | 14,940 / 14,920 |
| Full expansion / 16,000 | 359.56 | 344.92 | 49,484 / 49,956 |
| Full expansion / 64,000 | 905.30 | 943.05 | 188,972 / 190,040 |
| Nested concat / 1,000 | 32.03 | 32.16 | 9,312 / 9,296 |
| Nested concat / 4,000 | 140.32 | 139.53 | 29,652 / 29,688 |
| Nested concat / 16,000 | 567.07 | 582.87 | 108,472 / 109,468 |

Receipts are `1791793915721914731/samples.json` and
`1791793948626401962/samples.json`. At the largest full-expansion scale the
candidate median is about 4.2% slower and retains about 1 MiB more; A/A ranged
882.52–919.38 ms. Include directory ownership adds state to worklist visits.
The concat difference is about 2.8%, with A/A spanning 547.94–612.55 ms. These
bounded shared-host observations establish neither universal improvement nor a
claim of zero overhead.

The new repeated-source-include workload establishes an A/A scaling baseline
in `1791793964917222573/samples.json`: 28.29/28.09, 116.60/119.21 and
549.35/556.56 ms at 1,000/4,000/16,000 calls. The cached expression contains
a decoded string and nested eager expansion. Largest peak RSS is about 102 MiB;
A/A samples span 443.31–557.28 ms. The final subsequent compiler change affects
only error projection and does not run on these successful workloads.
