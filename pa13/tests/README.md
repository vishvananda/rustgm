# PA13 fixture ownership

The manifest declares 803 cases, with exact stdout, stderr and exit-status
oracles. Its default mode is `--emit-lowir`; cases explicitly select the
supplementary `--emit-patterns` view when testing selection and pending proofs.

| Cluster | Cases | Responsibility |
| --- | ---: | --- |
| bindings | 100 | Owned products/arrays, borrowed modes and edition-2024 restrictions |
| scalars | 47 | Ordered intervals, irrefutability and scalar matches |
| constructors | 49 | Enum/product decisions, captures and constructor witnesses |
| coverage | 45 | Composite exhaustiveness, usefulness and visibility |
| conditions | 72 | Let-else, if/while-let chains and storage on failure/exit |
| guards | 86 | Alternative retries, delayed moves and protected internal views |
| scalar-constants | 54 | Typed scalar queries and range endpoints |
| product-constants | 44 | Unit/tuple/array values and sparse repeats |
| nominal-constants | 63 | Derived equality obligations and structural values |
| floats | 68 | Floating domains, signed zero, NaN and guarded retries |
| slices | 35 | Dynamic length domains, aliasing prefixes/suffixes and rest captures |
| text | 46 | Exact string/byte identity, coverage and module provenance |
| references | 85 | Shared constants, places, alignment and root validity |
| boundaries | 9 | Pending reference proofs remain explicit |

These are authored course fixtures drawn from declared maintainer controls.
Every input and auxiliary module appears in the manifest. Preserve their bytes:
spans, node ordering, diagnostics and serialized output are grading inputs.

There are 348 same-outcome native programs and eight signed-zero pairs with
explicitly different Reference/rustc outcomes. The two members of each pair
assert counts of two and one, respectively. All 16 have a required manifest
reason and qualified pass/panic expectations for both compilers. They are not
runtime-parity claims. The thirteen acceptance differences are likewise explicit
in `rustc.reason`; see the owning handout. No crash or timeout counts as a
language rejection.

Run `make test-pa13`, `make test-native-pa13` and the root independent/architecture
checks. Only explicit `make ref-test-pa13` regenerates sidecars after checking
all declared statuses; never adjust an expected outcome to bless a failure.
