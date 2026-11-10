# Projected receiver methods (PA20)

These reductions retain the original core integer_decode CastInto<i16> lookup
through an associated Int result contract. PA20 owns every feature used here.
The contract declares object/native and complete LowIR expectations before the
compiler implementation and new text oracles. Explicit qualification and direct
generic calls are controls. References, mutation, lifetime results, nested
projections, complete trait/const arguments and output inference exercise the
same ordinary selected-call machinery. Missing parent/result/Copy premises,
wrong outputs, hidden traits, ambiguity and escaping lifetimes remain errors.

Width8/32/128 and128 unrelated types retain opaque generic receivers. Repeated
lookups may scale with the actual call sites; unrelated declarations must add no
inference/canonical/method work. All native positives check actual returned values.
