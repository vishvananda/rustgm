# Rigid candidate headers (PA20)

Reduces pinned core's `Peekable::next_if_eq` (`peekable.rs:318`), whose
`next == expected` compares `&I::Item` with `&T` under `I::Item: PartialEq<T>`.
Besides core's reference impl, the blanket `impl<F: FnPtr> PartialEq for F`
(`ptr/mod.rs:2592`) is a candidate whose header needs `T == <I as
Iterator>::Item`. rustc relates the normalized, rigid alias structurally, so the
header does not apply. Trait selection kept it as an ambiguous alternative.

Each case keeps that shape in an ordinary program: a generic
`impl<A: Pick<B>, B> Pick<W<B>> for W<A>` beside a header that would need the
rigid alias to equal another head. The local `Marker` blanket stands in for
`FnPtr`; the closed `W<u16>` header tests a concrete head. Three executable
positives check returned values: a parameter, a concrete head and an
impl-parameter projection with a method-level premise. Without the premise, or
with only the closed header applicable by shape, the program rejects (Rust
E0277). The closure form is PA24's `rigid-candidate-headers`; the source-core
forms are in `pa31/tests/spec/rigid-candidate-headers`. See
[the checkpoint](../../../../docs/pa20-rigid-candidate-headers.md).
