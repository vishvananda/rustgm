# Projected receivers from closure calls

`call-result.rs` retains the original closure-driven test of an inherent
method on a projected receiver. Both enum variants are produced by calls
through a generic `FnOnce` bound and their returned values are checked.
The [PA20 counterpart](../../../../pa20/tests/spec/projected-inherent-headers/)
uses an ordinary generic function to isolate projected receiver selection.
The original case came from the
[maintainer associated-declarations suite](../../../../dev/tests/associated_declarations.rs).
