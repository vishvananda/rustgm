# Source Copy signatures across specialization

A selected function and its opened body can have different callable binders.
For example, an unused `T = &u8` argument leaves the public `fn() -> u8`
signature without a bound region while the actual specialized contract retains
the introduced region. Additional caller premises may subsequently enlarge the
contract's universal namespace. Comparing the public signature directly with
the contract rejected valid source-mode programs, including the retained
`needs_drop::<&D>()` control.

Function owners now retain the actual specialized execution signature alongside
the selected public signature. Original bodies have no separate execution view;
specialization validates their exact original signature before transporting it.
Source Copy decisions retain both signatures together with the full instance,
body identity, requirements and environment. Dispatch updates the owned signature
with its contract. No signature is reconstructed from a later universal count.

The field fits the existing owner enum: exact private checks keep Owner at
20 bytes, Body at 448, Contract at 184 and Value at 24. Preparation and subsequent
identity checks add constant work and no body traversal. Ordinary dispatch reuses
its existing memoized type fold. The cold prepared source-Copy carrier grows
from 96 to 104 bytes; before/after debugger receipts retain executable hashes.

Development evidence is under `obj/parallel/copy-signatures-v1/` in the isolated
`rustgm-pa29-work` checkout. The independent plan declares answer 7 before running
five pinned Rust 1.98.0 object/native controls. All pass. Four exercise unused
reference and reference-pair arguments, nested callable binders, and early plus
external lifetimes; the fifth checks a source Copy read. Four matching candidate
native programs pass. The private read invariant uses an ordinary local entry
instead of the independent C export, retaining the same generic read body.

`development-v3` passes 12 source integration tests, six exact private invariants,
strict all-target Clippy, the 1,978-file audit and 152 harness checks. The private
controls reject public-only and owned-only signature changes, a missing full
instance, foreign contracts and specialized owners presented as originals.
Earlier wiring and function-size lint failures remain in their original receipts.
`closing-v1` passes all 8,382 active text cases in 39.54 seconds under shared load
and confirms unchanged compiler/test source hashes. No text references or
assignment activation changed. Combined release execution,
performance qualification and the intrinsic worker's unchanged original reference
control remain integration requirements.
