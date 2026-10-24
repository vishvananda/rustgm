# Physical type privacy through scoped macros

Definition-site lookup can select a name while its semantic type remains private
from the physical module using it. These contracts cover private function items,
return types, nested signatures, inherent methods, associated constants and
opaque bounds. Public function pointers and public opaque interfaces must still
hide their private implementations successfully.

The immutable `contract.json` predictions matched pinned Rust 1.98.0 metadata
and object observations. `review.json` binds all 36 observations, including five
native answers, from the ten scoped cases and three earlier ordinary companions.
The ordinary companions are qualified as standalone PA12 programs in
`pa12/tests/spec/privacy/`, with independently declared outcomes, generated text
references and full PA12 source/native checks. These PA31 cases use the internal `decl_macro`
feature; PA31 remains planned and none enters an active assignment pass total.

The compiler must inspect completed type/adjustment facts, independently of macro
name selection, without reparsing syntax or flattening shared type graphs.
Opaque interfaces expose declared bounds and captured arguments; their hidden
representation is not the caller's privacy obligation. Type work is shared per
physical body, with bounded work and lazily built module ancestry.

RustGM component `80f9e069` passes the ten scoped cases with all frontend
controls and five affected native test functions. Integrated release and
performance qualification remain required; see `docs/pa31-physical-privacy.md`.
