# Visibility of re-exported namespaces

A `use` can select both a type and a value of the same spelling. A public
re-export is valid when an available namespace permits that visibility. Other
available namespaces remain imported at their own narrower visibility.

The public type of `pub struct Token(pub(crate) u8)` therefore keeps a callable
crate-visible tuple constructor through a public re-export or alias. A
`pub(super)` field permits construction within that parent and its descendants;
it does not permit construction outside that module. Inaccessible private
constructors and functions remain unavailable. An import whose selected names
all reject its requested visibility remains invalid.

These fourteen cases include the original core `num::TryFromIntError` shape,
aliases, public fields, parent restrictions, a public type paired with a
crate-visible function, and a crate-visible type paired with a public function.
The manifest declares nine successes and five rejections. Independent pinned
Rust metadata checks establish those outcomes before text generation. The first
independent investigation disproved a private-function/public-type rejection
hypothesis; the original input, prediction and raw result are retained separately.
No fixture was changed to make an implementation failure pass.

The text view must preserve each imported namespace, actual definition target,
import origin and effective visibility. The compiler's private controls compare
those facts to the retained real source selections. Privacy opponents run
completed ordinary name resolution, including body paths. Owning reference
batches bind the source bytes, declared outcomes and generated text.
