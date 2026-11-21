# Method paths as callables with a late-bound self lifetime

Every source and outcome was declared before any Rust 1.98.0 or RustGM run of
these bytes. Each program passes a method path, `View::view` or `Word::text`,
to an adapter whose premise is `F: FnMut(S::Item) -> B`. The method's
signature, `for<'r> fn(&'r Self) -> &'r T`, names its own late-bound lifetime
in its output, so the output is known only once the input `&'a Word` is
equated with it: `B` is then `&'a Text`. A trait path also infers `Self` and
`T` from that input and the selected implementation, as rustc does.

The five positives map through a trait path, return the adapter as an
`impl Seq` with a named or an elided lifetime, select between two `View`
implementations by the expected `Item`, and keep the inherent path as the
control; they execute their assertions. The rejection maps a type that
implements no `View` (rustc E0277; RustGM reports the path, rustc the
argument that holds it).
