# Callable premise results in implementation selection

Every source and outcome was declared before any Rust 1.98.0 or RustGM run of
these bytes. Each program selects an implementation whose callable premise,
`F: FnMut(I::Item) -> Option<B>`, is the only clause that determines a header
variable. The closure's output fixes `B`, and `B` is the `Item` that a later
`Gather` goal needs, as rustc's projection of the selected premise gives it.
The premise input is a projection, `I::Item`, that is not normalized while the
header is matched; the closure's output does not depend on it.

The five positives cover a struct-literal adapter, a provided-method adapter
(the `env::args().filter_map(|a| a.parse().ok())` shape), turbofish selection,
and each with the premise input written as `u8`. They execute their assertions.
The three rejections keep the premise requirements: a closure output that is not
an `Option`, an `Item` no `Gather` implementation accepts, and two `Gather`
implementations that leave the result ambiguous.
