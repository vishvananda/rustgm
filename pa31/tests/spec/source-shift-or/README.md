# Source shift and bitwise-or hypotheses

These sources are frozen before observation. No candidate build or independent
generation has run. The immutable discriminant checkpoint remains `645fa92b7`.

The no_core inputs retain the complete existing sizing/Clone/Copy foundation and
the actual pinned Shl/BitOr trait declaration blocks from core/src/ops/bit.rs.
Primitive implementations are explicitly authored reducers. They do not replace
the original library's implementations or erase its stability/const contracts.
The actual consumer is Arguments::from_str in core/src/fmt/mod.rs815–821.

The protocol needs full Self/Rhs/Output and the actual owned shl/bitor method.
Pinned rustc checks overloaded trait operations first, then builtin scalar type
and const rules. Shift syntax retains the left result type without forcing the
RHS to that type; bitwise-or scalar operands/result have the same type. Missing
traits/impls, wrong Output/RHS and negative/overflowing constant shifts remain
opponents. Ordinary explicit and nonprimitive method calls retain their actual
const and body permissions.

The custom primitive implementation deliberately returns99/42: the predicted
native output is7 for builtin syntax and99/42 for explicit calls. The separate
overloaded struct/generic source predicts7/12/13, proving source method behavior
is retained for nonprimitive inputs. A standard-library inference observer
predicts an i32 RHS and result7; a checked-runtime observer predicts a real shift
panic. Neither standard-library observer claims a new candidate library route.

`predictions.json` binds every complete source/observer byte and declares outcomes
before execution. Error codes and warning hypotheses may be wrong; retain all
actual raw results and disagreements before correcting any final contract.
The source-only implementation plan reuses existing source Not contracts, the
ordinary inference/solver/selected call, scalar operations and backend. It adds
no Body/Value/Rvalue/Report field, proof owner, cache or fabricated method.
