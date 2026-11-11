# Ordinary trait function values

Three Rust 1.98 object outcomes belong at the ordinary trait/function-pointer minimum: return a trait function pointer, pass it to a pointer constructor, and reject creation without the required Factory bound. All three complete LowIR outcomes pass. The two positives are unused generic compilation contracts, with explicit code-generation classifications in the main manifest. Three new references were independently reviewed and generated; existing references stay unchanged.

FnOnce constructor inference belongs in the PA24 counterpart; const creation/invocation belongs in PA25. These authored reductions acquire no source-library privileges.
