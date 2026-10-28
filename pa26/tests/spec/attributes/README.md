These declaration fixtures cover the finite `rustc_as_ptr` and `cold` header
consumers needed by retained source-library functions. `rustc_as_ptr` requires
its real internal-attribute request and actual function ownership; it supplies
no pointer, borrow, const-evaluation or optimization permission. The foreign
prototype, missing request, malformed word, unsafe wrapper and wrong owner are
opposing controls.

`cold` is an independent stable advisory word and preserves `inline(never)`.
It cannot make a non-const function callable in a constant. The current finite
consumer supports function bodies. `required-prototype.rs` and
`required-foreign.rs` are valid Rust inputs that remain explicitly unsupported;
they are not Rust rejection predictions. Wider valid targets and downstream
lint behavior remain required.

Both words retain authenticated physical headers. Strict source publication
rejects their missing metadata transport rather than silently discarding them.
PA26 remains planned; these inputs do not activate a grading lane.
