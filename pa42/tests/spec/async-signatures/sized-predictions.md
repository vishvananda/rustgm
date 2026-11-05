# Pending exact Output sized predictions

Frozen before independent observation: default source Future::Output sized
would reject a required async fn returning str; ordinary required fn returning
str and a genuine relaxed source Output:?Sized would accept. Check the actual
pinned result before applying any restriction. These are declaration-only
controls; no ResumeTy body, future representation or runtime is substituted.

## Independent correction

Pinned Rust1.98.0 accepts all three exact no_core declarations (0,0,0).
The first emits opaque_hidden_inferred_bound as an advisory warning; it does
not hard-reject the impossible associated constraint. The initial prediction
was wrong and remains above and in sized-cases.tsv. The raw observations and
exact stderr are retained in obj/parallel/async-signatures/oracle/sized-output.
reviewed-sized-cases.tsv is the reviewed acceptance contract. Preserve the
actual Sized schema and ordinary unsized required declarations; do not add a
stricter rejection. That advisory warning producer remains required PA40 work.
