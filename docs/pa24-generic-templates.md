# Generic closure body proofs

Closure CFG construction now shares one constructor between ordinary bodies and
templates. A closure in a generic owner retains its source closure identity and
shares the owner's immutable permission environment. Every original template,
including unused closures and nested closures, passes the existing ownership,
loan, promotion, lifetime and destruction checks. The checked template token
keeps a separate closure index; function and initializer lookup remain unchanged.

Formal closure signatures restore inference aliases to their declared body
universals before those signatures contribute assumptions. The original equality
obligations remain required. Already closed signatures take a constant-time
property check and bypass equation construction. This fixes the generic
`|value: &T| -> T { *value }` proof for `T: Copy` without weakening the
corresponding move and escaping-reference rejections.

Executing a closure defined in a generic owner still requires instance demand
and lowering integration. Captures and dynamic callable adapters remain separate
unfinished work. PA24 is not active.

The focused `closure-templates-v2` batch passed 30 tests across generic closure
templates, source closures, closure lifetimes/pointers and template loan/region/
cleanup groups. Fourteen declared source outcomes were independently checked
with pinned rustc. The initial borrowed-copy proof failure remains in the v1
logs. A fifteenth contextual-borrow control and a formal-signature invariant
were added afterwards, together with the closed-signature fast path.

The v3 gate stopped at a test-only Clippy spelling error, which was corrected.
The v4 formatter and strict all-target/all-feature release Clippy command passed;
the guard was interrupted during its historical cleanup scan before audit,
focused and native stages. That incomplete receipt is not a passing gate.
The next integration batch must run those stages with scoped cleanup. No new
performance or assignment release claim is made by this local checkpoint.
Evidence remains under `obj/parallel/closure-templates-v*/` and the earlier
v1/v2 logs. Ordinary native and active text coverage for the preceding compiler
snapshot is recorded separately in the publication checkpoint.
