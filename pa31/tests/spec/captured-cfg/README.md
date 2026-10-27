# Original atomic primitive reducer

`atomic-primitive.rs` retains the exact `impl_atomic_primitive` macro definition
and its first eleven invocations from pinned `library/core/src/sync/atomic.rs`.
The surrounding ordinary trait and storage declarations make the names real;
the reducer tests expansion/configuration, not atomic runtime behavior or a core
artifact proof. Nine supported impls must remain active and the two 128-bit impls
must be removed. Every demanded predicate still uses actual feature authority.
