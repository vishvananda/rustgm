# Actual source Deref method contracts

These twenty authored complete inputs freeze seventeen required positives and
three language rejections before implementing implicit Deref method calls.
All independent Rust object outcomes agree. The native source also links with
the C observer and prints `receiver 23 41\n`, status zero, for real shared and
mutable method calls. Candidate native execution remains required.

The support prefix comes from the existing dereference-source foundation;
original foundation bytes remain untouched. Const variants explicitly declare
const Deref and const implementations. They reproduce the same lookup failure
as ordinary Deref. Direct and explicit-field calls work; even a concrete wrapped
receiver fails, so arrays, const generics and PartialDrop are not prerequisites
of the smallest failure. The array variants retain core's actual receiver shape.

Inherent wrapper priority already passes. Missing Deref, immutable mutation and
moving a non-Copy inner value reject independently; current early lookup errors
alone do not qualify the latter permission checks. Borrowed results, actual
DerefMut, selected source bodies and one-time receiver evaluation remain required.

Every source and auxiliary is in the manifest. The generated independent receipt
binds byte-identical observed inputs, diagnostics, source profiles and current
compiler observations; it generates no course oracle or whole-PA release.
Ordinary library forms remain in [PA26](../../../../pa26/tests/spec/deref-methods/README.md).
See [the reduction review](../../../../docs/pa26-deref-method-reductions.md).
