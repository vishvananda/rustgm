# Type names in source constants

Two no_core source-core libraries call `type_name::<T>()` the way pinned core
does: directly in statics and through the inline constant of a generic
`const fn`, as `any::type_name` does, including in a function that runs. A C
observer prints every name. The strict executable graph evaluates the
supported forms through checked constant calls. The recorded text in
`names.stdout` comes from the rustc 1.98.0 object.

Like rustc's type name printer, a nominal type is its declaration's crate and
module path followed by its arguments, with each lifetime argument erased to
`'_` (`minicore::Holder<'_>`); a reference leaves its lifetime out (`&str`).
Const arguments and array lengths print without a type suffix (`Signed<-3>`,
`[u8; 4]`), tuples keep the one-element comma, and primitives, `str`,
slices and raw pointers print as Rust writes them.

`refused.rs` adds a function pointer, which rustc names `fn(u8) -> u8` and
this printer does not spell. Like any type it does not spell (functions,
closures, objects, opaque and foreign types, declarations outside a module,
or imported declarations), it fails at the generic function's inline
constant. `outcomes.json` records the declared and observed outcomes.
