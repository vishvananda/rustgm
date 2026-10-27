# Scoped and legacy builtin declaration kinds

PA31 retains the compiled source declaration, matcher, visibility and token
provenance while taking the actual macro kind from its authenticated builtin
role. This follows pinned rustc_resolve::macros::compile_macro: it compiles the
source declaration before replacing only the extension kind from the registry.
Legacy macro_rules declarations retain textual scope; scoped macros retain
path scope. Lookup filters each actual textual candidate by requested macro
kind and resumes its real preceding scope without caching a filtered miss.

The original Iter file remains byte-for-byte after its recorded provider prefix.
Its declaration is a positive; invocation is explicitly unsupported. Parsing a
known role grants no expansion, compiler feature or foreign transport permission.
The original scoped formatting, byte concatenation and legacy autodiff inputs
retain their independently accepted declarations. Their prior failures remain in
historical evidence.

Seven new exact sources were frozen before fourteen metadata observations in
2021/2024. All statuses matched; eight warning/category hypotheses differed:
legacy cases emit unused_features (one also unused_variables), and the malformed
matcher diagnostic says missing fragment specifier. Raw predictions and streams
remain under the recorded observed binding. These are metadata observations,
not native claims or a blanket lint exemption. Existing source lint requirements
remain independent. Ordinary compiler-profile, missing feature, malformed
matcher, unknown role and unsupported execution controls stay closed.
