# Actual CStr source, constant storage and byte-word integration

CStr literals now use the actual completed source nominal type, static immutable
references and one NUL-terminated canonical byte arena. Runtime, promotion and
CTFE share the same typed producer. Physical storage is a real sized array of the
actual character element; its nominal logical view retains genuine ABI length
and field-offset facts. Equal byte/C content does not replace typed storage.
No new dense node, value variant, wire field or metadata solver is introduced.

The original CStr/slice `as_ptr` and `to_bytes_with_nul` bodies remain unchanged.
Ordinary privacy, type, const, unsafe, region and loan checks remain authoritative.
Raw literal writes and end-of-allocation reads reject for actual memory causes.
Unused alternate extra-field/generic role headers retain pinned Rust legality.
Whole modules, derives, constructors/from_ptr/strlen, broader CTFE and authenticated
transport remain required; this is not whole PA31 or an original library artifact.

Eleven exact public controls across three targets, strict selected Clippy, format
and architecture checks pass in107.29s. All356 PA22 and142 PA2 text cases pass in
that same guard. Four real native observers pass in38.79s: `CStr 65 2 0 195 169 255
92 1`, `storage 8 65 0`, `roundtrip 255 0`, and `byte-word 255 0 17 23`.
The last reconstructs actual CStr, ordinary nominal and generic nominal references
from initialized provenance-bearing byte fragments. It fails before the focused
three-file decoder fix and passes unchanged after it. The simpler whole-slot
roundtrip already passed before that fix and is not evidence of the decoder gap.

Fresh immutable CLIa8b8d578 advances unchanged original core to **LIB185 Reborrow**,
`library/core/src/marker.rs:1356`, file81 byte51821. Source admission exits1 after
31.0827s/375 opens; all722 original inputs, fifteen cfg/profile values, target,
options, capAllow and finite work/allocation budgets remain unchanged. This is an
observed declaration frontier, not a throughput comparison or built core artifact.
Five tracked configuration-source files stay identical; const_eval/values.rs is
explicitly rebound only for replacing duplicate Rvalue::Text classification with
the shared typed storage producer. No configuration evaluator or value changes.

Forty immutable A/A+ABBA ordinary-text observations preserve exact output equality.
Median paired candidate/baseline ratios are1.0172 at512 functions and1.0062 at2048;
A/A ranges are0.1074–0.1326s and0.4295–0.4579s. These are noisy ordinary controls,
not a broad throughput or speedup claim. The first --types usage-error attempt
is retained separately and excluded from qualified performance observations.

[Machine evidence](source-cstr-main-join.json) binds the exact source snapshot,
qualification receipts, immutable CLI, original inputs and reviewed source delta.
The [independent reference packet](source-cstr-reference.md), local-reference.json
and byte-word-reference-v1.json preserve every prediction/setup discrepancy.
In particular, the pinned no_core UTF8 hypotheses accept through object generation;
they were never executed and are not candidate negative controls. An empty worker
selection is explicitly invalid and excluded from qualification.

Prior full [42255186438](https://github.com/vishvananda/rustgm/actions/runs/42255186438)
passes all private, release/course, native, debug and independent gates on the
preceding IntoFuture/range compiler. Initial CStr full42265080175 exposed actual
byte-slice pattern regressions. The [typed slice repair](text-slice-repair.md)
passes full successor42273932679 on main14fe4f9c5, including all private/release,
course, native, debug and independent gates. Original core remains last observed
at LIB185; the repair's full success is not a fresh original-input probe.
Keep current/predecessor immutable pins, existing warm btrfs trees, two heavy jobs,
16GiB caps,150GiB reserve and five-minute local guards. No whole PA after24 is
completed by this checkpoint.
