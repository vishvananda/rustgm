# Rigid candidate headers in a closure (PA24)

The closure form of PA20's `rigid-candidate-headers`, as at pinned core's
`peekable.rs:318`, where `next_if_eq` compares inside a `next_if` callback.
The closure's input is the projected item. Its trait obligation must not keep a
blanket header that would need `T == <I as It>::Item`. rustc relates that rigid
alias structurally. One executable positive checks the returned value. See
[the checkpoint](../../../../docs/pa20-rigid-candidate-headers.md).
