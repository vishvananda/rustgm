# Scoped execution and physical semantic privacy

Component `80f9e069` executes scoped declarations through their retained matcher
and actual invocation owner. Three source/native failures from the first real
execution gate are repaired: private function-item types cannot acquire access
from definition-site lookup, generated modules retain primitive-prelude access,
and an exact opaque introduced item is selected before rewinding to its parent
context. Legacy macro lookup retains its earlier boundary order.

The compiler consumes solved body types, selected calls, associated constants
and adjustment facts after publication. It checks nominal, function signature,
trait and associated-item identities from the physical module. Public function
pointers may hide their private target; opaque interfaces expose declared bounds
and captured arguments while retaining their hidden representation privately.
No additional field is added to the hot Checker, Body, Value, Type or Package.

Each body shares an iterative visited type graph and lazily builds its physical
module ancestry once. Primitive-only checks allocate no graph storage. Attempts,
edges and errors consume the bounded budget. Actual source signatures with
shared tuple/pointer DAGs at depths 8, 32 and 128 require 29, 101 and 389 steps,
visiting 19, 67 and 259 unique nodes. Another 128 uses cost exactly 128 steps
at every scale. This invariant measures the privacy traversal; the combined
checkpoint below separately qualifies total frontend costs.

## Independent and RustGM evidence

The immutable ten-case PA31 source predictions and three ordinary companions
match pinned Rust 1.98.0: 26 metadata/object observations and five native answers.
Source contracts, original predictions and raw diagnostic streams are retained
under `pa31/tests/spec/declarative-macros/semantic-privacy/` and the root worker's
`obj/parallel/semantic-privacy-oracle-v1/`. Results SHA-256:
`664c0a1360406d5607d71801b55041cd9b050b009c9440ad6914757cf3a74cbc`.

The ordinary companions were independently observed again as actual standalone
PA12 programs, removing later C ABI/linkage wrapper features. All six
metadata/object observations and the one native program pass. Results SHA-256:
`8b4c327ad4cf70d3381db56ba017319e7f40746da8ac5e14004b061efbf5fb32`.
Their PA12 manifest outcomes were declared before generating only those three
text references through a scoped course receipt. Complete PA12 qualification
passes 111 independent source checks, 99 native checks with 52 executable
comparisons, and all text cases. The public function-pointer result retains the
private helper as a real function and returns 7 through an indirect call.

Formatting, architecture/file audit and all-target strict Clippy pass. The
component runtime executes all 1,498 nonignored private tests, 319 frontend
cases, five ignored native test functions, and all 8,382 preexisting active
text cases. The new three PA12 cases bring the active text count to 8,385.
No preexisting reference changed. Runtime receipt:
`obj/parallel/semantic-privacy-runtime-v2/complete.json`, checks SHA-256
`a41d8a86256ab8a755f2217c164aeadcdcbb8fbdbec4a044c121dd8be12b0dbb`.
The first runtime attempt was interrupted for the user's authentication pause;
the recovery authenticates the unchanged successful static, harness and upstream
checks, then completes the previously unrun runtime inventory. It does not count
an interrupted build as a test pass.

## Remaining acceptance

The integrator has merged this component with the separately qualified source
concat_bytes implementation. Their shared eager seam preserves the actual
formatting builtin's opaque definition context and normal builtin transparency.
The [combined compiler checkpoint](scoped-bytes-checkpoint.md) now completes
release/native coverage, affected source checks and the 125-input performance
matrix, retaining all seven reviewed RSS costs. This does not complete foreign
macro export, nonempty formatting, attribute transformation or a core artifact.
PA31 remains planned and its scoped fixtures do not enter active PA totals.
