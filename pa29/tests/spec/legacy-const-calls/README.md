# Required external legacy const arguments

The genuine Rust provider has an actual const generic and an ordinary addition
body. External direct-path legacy calls and explicit modern/closed aliases are
Rust-positive; explicit wrong argument counts and runtime-valued const operands
reject. The PA29 generic-transport suite publishes the provider and checks
each fresh client. See the owning
`pa29/tests/legacy-const-call-outcomes.json` inventory.
