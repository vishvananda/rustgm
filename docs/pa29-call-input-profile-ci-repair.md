# Imported call-input protocol repair

Source `fbf4242c7` repairs the twelve private failures reported by full
[CI41054298940](https://github.com/vishvananda/rustgm/actions/runs/41054298940).
All twelve were reproduced with the unchanged saved test binary before repair.
Four were real Provided-consumer failures; eight were stale cold role-size
assertions after adding From. No source fixture, wire expectation, language
outcome or unsupported positive was changed to hide a failure.

The conversion implementation collected inherited call inputs whenever any
crate linkage existed. Ordinary source headers split their clauses across
actual parent rows, but selected Provided consumers retain their installed
scope-input reader. Walking those rows as ordinary headers requested a source
implementation header that did not own the imported declaration, failing with
`missing impl header owner`.

Call checking now chooses the protocol from the actual typed imported owner.
Ordinary source metadata retains the paid parent walk and current-owner checks;
selected and ordinary local consumers use the established declaration-input
view. This does not grant a trait premise or call permission. Actual body,
const/unsafe, lifetime, source identity, environment and selection proofs remain
required. No new cache, owner ledger, hot field or metadata profile is introduced.

From remains appended at language-role index 71, extending only cold tables.
The explicit guards now expect 72 roles, Reservations 2328, Catalog 1632, imported
role storage 1224 and the declaration array 1440 bytes. Existing role ordinals,
old wire fences, Node 36, Body 456, Package 256 and Resolution 2032 remain guarded.

## Qualification

All three compiler gates bind the same 3,295 source/tool fingerprints:

| Gate | Result | Receipt SHA-256 |
| --- | --- | --- |
| Focused private/static | All 152, including every original CI failure and prior 140 controls; 90.87s | `ae98f5954820e02b469ea1a063365bc8a26c867204562bd03494a5f8a78d80d6` |
| Production and text | All 69 exact checks and 4,460 text cases across PA9/18/19/20/21/23; 274.12s | `a17567ec05f85afa5e26360cdf3d858f7565a0ca4f9e5af5e4898b611ea9a016` |
| Conversion native | All 24 standalone positives and both fresh clients; 47.78s | `038b228709d3dfa601dc603ffbb0f08720b03818154cf7817435d713a65baf73` |

The receipts are under obj/qualification/fast/1793598475182043344,
1793598703328603256 and 1793599385983934009. The original failed run and the
intermediate 151/152 result remain retained; the latter exposed the second cold
imported-table assertion in the same range-contract test.

Fourteen additional existing controls pass on that exact test binary: Provided
wrong-type/arity/trait/dead-borrow and contract/source-owner failures, actual
default/self-premise checks, defining environments and ordinary zero-work views.
The original full-family provider/caller compiles freshly with pinned Rust and
executes native 7. The current candidate executes the same observer result 7.
Receipt obj/parallel/resume-20261102/consumers-v2/receipt.json has SHA
`05a3359d7aa03cec968d6463c38b9bf5fa8ba64b8ba0d41615561ee4e219ba9a`.

The initial test driver linked an independent C main with the candidate object,
which already contains the original entry point. Its failed link is retained.
The corrected candidate uses its real entry; the Rust library object uses the
independent C driver. Successful opponent, harness, Rust and object-generation
steps were reused only after checking every recorded source/tool/binary/object
hash; the corrected candidate link and execution actually ran. There is no
source change or failed-run-to-passing-run substitution.

The unchanged 722-source core attempt still reaches LIB-111 at ffi/va_list.rs:416,
file 121, byte 16662, in 22.01s with 375 loading opens. Its target, cap Allow and finite
limits are unchanged. Current immutable CLI SHA is
`71054639471c114cc6d265a4d8873eceb38a4282977a983af3b32df2f51ddcd7`;
receipt obj/parallel/resume-20261102/core-v2/receipt.json SHA is
`4345b183af0b2b3dc17db5b700bfa0b18d37f3d881f35f7257aca7fb1b5242e4`.
This repairs a regression; it does not advance the core frontier, finish another
assignment or produce an original library artifact. Full CI remains a separate
qualification. Resume the actual [forced indirect ABI work](pa30-forced-indirect-abi-plan.md).

## Scratch relocation

The root filesystem fell below the 150 GiB reserve during the pause. Obsolete
stopped-worker build caches were retired, then the user authorized btrfs scratch.
The same existing worktree now lives at
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work`; its former home path is an alias.
Source/tool 3,295 and both saved binary hashes match before and after the move.
The obj directory itself is regular. Git registration and the shared controller
use the actual new path, with unchanged two-job/two-thread, 16 GiB absolute output,
150 GiB reserve and 300s development limits. No new worktree or cache was created.
Logs, fixtures, raw diagnostics, current pins and the warm integration cache are
retained. Routine commands should use the actual btrfs worktree path.
