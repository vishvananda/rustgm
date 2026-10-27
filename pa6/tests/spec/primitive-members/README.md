# Primitive member fallback

The ordinary source packet was frozen before implementation. Pinned Rust
metadata accepts both direct and imported empty-module fallback cases; immutable
RustGM2eda rejects their actual u8::build uses during name lookup. Real module
members and nominal/type-alias shadows already accept in both. Qualified and
nonprimitive missing paths, imports, private members and ambiguous members
remain rejected. All20 predeclared tool outcomes match across the ten sources.

The fix must reuse the canonical primitive associated-prefix path after normal
reference lookup, keeping actual method/impl/body proof in PA19. It must not
change imports or suppress privacy/ambiguity errors. No new cache or record is
required. Candidate qualification and reviewed text references are pending.
