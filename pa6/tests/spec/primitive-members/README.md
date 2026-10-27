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
required. Candidate private6 and separate production/CLI lexer10 now pass on source
376c9ed90. Actual public CLI name checks pass for all10 inputs; the two newly
accepted trait-member cases also pass public LowIR construction. No native
result is inferred. Reviewed text references remain root-owned and pending.

At widths8/32/128, definitions/scopes/primitive identities remain39/9/19,
references are12/36/132 and inference steps394/1546/6154. These are exact
existing counters, not whole-front-end work or timing claims. Resolution2056,
Package256 and Body456 remain unchanged; no cache or persistent field is added.

The first candidate gate stopped at filename ownership audit before compilation;
renaming the cfg(test) file left its module alias and contents unchanged. Both
subsequent gates and all actual CLI observations pass. The original eight-source
and separate two-error-boundary oracle packets remain preserved without repeats.
