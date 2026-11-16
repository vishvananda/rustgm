# Type names (source core, bring-up profile)

Two no_core source-core libraries call `type_name::<T>()` the way pinned core
does: directly in statics and through the inline constant of a generic
`const fn`, as `any::type_name` does, including in a function that runs. A C
observer prints every name. Under the bring-up profile the call runs in
constants, waived by name; strict compilation keeps it fenced. They are
checked against rustc 1.98.0, and `names.stdout` is what the rustc-compiled
object prints.

Like rustc's type name printer, a nominal type is its declaration's crate and
module path followed by its arguments, with each lifetime argument erased to
`'_` (`minicore::Holder<'_>`); a reference leaves its lifetime out (`&str`).
Const arguments and array lengths print without a type suffix (`Signed<-3>`,
`[u8; 4]`), tuples keep the one-element comma, and primitives, `str`,
slices and raw pointers print as Rust writes them.

`refused.rs` adds a function pointer, which rustc names `fn(u8) -> u8` and
this printer does not spell. Like any type it does not spell (functions,
closures, objects, opaque and foreign types, declarations outside a module,
another crate's declarations), it fails closed: the generic function owning
the inline constant becomes the one bring-up blocker, as core's
`any::type_name` does. `outcomes.json` records the declared and observed
outcomes.
