# Source coercion discovery with pending aggregate arguments (PA31)

Original Alignment::try_from returns Result from a blanket conversion while its
target remains inferred. The actual source CoerceUnsized role causes an eager
nominal mismatch probe to visit the associated Error before target inference.
A small authored source protocol reproduces the failure; removing that role or
qualifying the target passes. PA31 owns every source-role fixture here.

The manifest declares all object, complete LowIR and native outcomes before
implementation. Positive controls cover enum/record, source const contracts,
generic receivers, forwarded calls, raw projections and explicit target/from
paths. Negatives keep target/error identity, generic trait/method proofs and
actual CoerceUnsized evidence mandatory. Source declaration headers are authored
protocol fixtures, not copied or altered upstream files. Every input, including
the native C observer, has a manifest hash. The whole PA31 remains planned.

The initial nested opaque-result negative reaches an older normalization fence
after the discovery fix. It remains manifest-registered under required/ with its
missing-equality diagnostic still required; it is not counted as completed. The
active unrelated-result control checks the same missing equality directly at its
opaque root. Original failed source/manifest and independent outcomes are retained.
