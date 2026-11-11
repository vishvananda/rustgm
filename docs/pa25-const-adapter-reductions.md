# Const adapter reductions before implementation

The current original-core prefix first fails in
`NeverShortCircuit::wrap_mut_1/2`, `ops/try_trait.rs` file95 byte16817/17146.
Both const functions return a conditional const `FnMut` and `Destruct` opaque
result whose actual `const move` closure captures a conditional callable.
The original 722 input hashes, bindings, profile and limits remain unchanged.
The last 120-second observer still timed out; no core artifact exists.

The [PA25 packet](../pa25/tests/spec/const-adapters/README.md) owns ten ordinary
language reductions, including six required executable positives. The
[PA31 counterpart](../pa31/tests/spec/const-adapters/README.md) owns ten actual
source-role contrasts. Both manifests declare six positives and four negative
outcomes before a compiler correction. Pinned Rust object compilation agrees
with all twenty declarations; all six ordinary positives execute with status
zero and empty output. These are draft requirements, not activated course tests.

Ordinary and const-factory controls pass current RustGM LowIR. Conditional
ordinary forms stop earlier at bootstrap Destruct lookup or const declaration
preparation. Keeping those valid forms in PA25 avoids assigning the ordinary
language requirement to a later PA merely because a source-mode probe is easier
to construct.

Actual source self, binary and forwarding adapters reproduce the original
canonical selection failure. Diagnostic-only CLI
`ab8a0041ef393f95ce5678a4745dfb0fda4ed6d00ec4ff406ade257449a97da3`
confirms the explicit const-closure defining-obligation continuation fence.
Direct conditional invocation and ordinary/const-factory source controls pass
type analysis. No compiler correction or permission shortcut has been made.
The temporary instrumentation is removed, and its producer patch and raw
observations remain in `obj/parallel/const-adapter/trace/` and the trace pin's
producer directory. Its completed diagnostic executable was retired with scoped
cleanup1794366792736214583; the current available compiler remains8b73f15a.

The next implementation should reuse the existing iterative scoped clause
frontier. It must bind each selected closure's actual owner parameters and
inherited lifetimes, instantiate its defining requirements with the goal's use
effect, and retain every child proof before publishing callable evidence.
Signature equality or a const keyword cannot pay those requirements. Preserve
ordinary call performance, capture and lifetime checks, source role authority,
unpaid conditional opponents and explicit retained-storage/work charges.
Generic body construction, native adapter execution, general conditional
function-item proofs and imported transport remain separately required.

Evidence lives under `obj/parallel/const-adapter/`: predeclared probe plans,
independent object/native observations, immutable CLI comparisons and the
diagnostic-only trace. The initial no-Destruct direct-call prediction failed
with Rust E0493 and is retained honestly; it is not one of the packet's required
positives. Source runtime-creation refinement preserves the genuine role prefix
and reaches the intended creation-context rejection. Committed independent
receipts bind the final fixture hashes rather than those exploratory inputs.
