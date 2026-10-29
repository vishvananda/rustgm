# Calls in selected derived field types

The copier retains the real Call callee and ordered argument list, including generic
type applications through existing typed traversal. This grants no const, unsafe,
API, body or target permission. Semantic checking and actual source execution remain
necessary. Closures, statement-bearing blocks and stored decoded payloads retain their
explicit wider-profile boundary.

The PA26 input independently checks real standard `size_of` and ordinary const function
counts, with nonconst/arity/result opponents. The PA31 source consumer uses actual
ordinary const function bodies, generic arguments and source Clone/Copy through CTFE
and LowIR, then a separate C object. It does not replace the original size_of/Div bodies.
The unchanged TypeId declaration is preserved as an attributed syntax observer, not
an invented standalone lang-item provider or runtime TypeId implementation.
