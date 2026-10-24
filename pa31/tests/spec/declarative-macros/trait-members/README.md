# Scoped trait method identity

These authored PA31 inputs cover LIB-002's PA19 trait lookup consumers. The
contract was frozen before execution. Pinned Rust 1.98.0 accepted or rejected all
seven programs as predicted: fourteen metadata/object observations and five
native programs returning `7`. `review.json` binds the source hashes, tool,
profile and all 24 observations. This is independent language evidence;
RustGM scoped invocation and these consumer cases remain required work.

The cases distinguish three rules that must remain separate:

- An opaque trait name still contributes methods when its module is visited.
  A captured method identifier can therefore call the hidden trait's method.
- A literal method name uses the macro definition's imported traits. A captured
  method name uses the caller's imports; the opposing missing import rejects.
- Sibling expansions may introduce identically spelled traits and methods for
  the same type. Each generated call selects its own member; an ordinary caller
  cannot select an introduced method by spelling alone.

The pinned implementation's `rustc_resolve/src/lib.rs::traits_in_scope` visits
scopes using the method identifier's normalized context. `traits_in_module`
collects module traits, and type checking performs hygienic member selection.
These sources use the ordinary upstream sysroot for independent execution;
they do not establish RustGM source-provider admission or artifact transport.
No generated text oracle or assignment activation is implied.
