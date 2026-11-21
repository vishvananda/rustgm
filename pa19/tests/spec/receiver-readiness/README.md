# Receivers that a pending trait goal determines

Every source and outcome was declared before any Rust 1.98.0 or RustGM run of
these bytes. In `let v: V<_> = gather(b); v.0.f` only the unique
`V<?T>: Collect<B>` implementation names `?T`. rustc selects the pending goals
that can be selected now before it requires a structurally known receiver for
a field or method; so does the checker, when the receiver's head behind its
references is still unknown. Unfinished goals stay with their owner.

The six positives read a field, a nested tuple field and a method through a
by-value or by-reference `gather` result, in one or two statements; the
annotated binding is the control that needed no selection. They execute their
assertions. The rejection keeps two `Collect` implementations for one input, which
leave the result ambiguous (rustc E0283).
