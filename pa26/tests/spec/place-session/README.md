# Per-body indexing place sessions

This typed state reduction reuses the active, independently checked PA26 shared
index fixture. Seed a completed body's selected index/call offset and mutable
mark, then begin a new function or constant using the same NodeId. The new body
must create its own row and clear old mutable permission. Ordinary same-body
Index-to-IndexMut refinement remains covered by the existing PA26 fixture group.
This is the reduction for the actual original-core dot_calls index panic exposed
by clearing the source method-probe cumulative guard; a panic is never an
accepted source-language outcome.
