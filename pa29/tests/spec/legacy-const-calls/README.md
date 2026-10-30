# Required external legacy const arguments

The genuine Rust provider has an actual const generic and an ordinary addition
body. External direct-path legacy calls and explicit modern/closed aliases are
Rust-positive; explicit wrong argument counts and runtime-valued const operands
reject. The local-header checkpoint intentionally fences provider metadata until
the index contract and actual external argument/CTFE protocol are transported.
No Rust-positive result is relabelled a language rejection. See the owning
`pa29/tests/legacy-const-call-outcomes.json` inventory.
