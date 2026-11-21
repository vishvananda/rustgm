# Inference holes a nested closure resolves in a captured local

Every source and outcome was declared before any Rust 1.98.0 or RustGM run of
these bytes. A parent local's type can hold an inference hole that only a
nested closure resolves: the input of a sibling closure that the nested body
calls with a borrow, or an `Option` slot it assigns a borrow to. rustc infers
the bodies together. The region the hole gains inside the nested body is a
port to the parent, as a region an unannotated parameter gains there is; the
body's uses of the captured local name the port's universal slot, as its
captured place does.

The two positives pass a sibling closure to `ok_or_else` and `map_err`, and
assign a borrow to a mutably captured `Option`; they execute their assertions.
The rejections keep a hole used at both `&u8` and `u8` (rustc E0308, at the
same call) and an annotated input lifetime that escapes into a captured local
(rustc E0521; RustGM reports the closure whose parameter rustc labels).
