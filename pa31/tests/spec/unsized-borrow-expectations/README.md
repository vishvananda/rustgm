# Unsized borrow expectations

The complete sources in `outcomes.json` preserve the independently checked
minicore inputs. The prefix comes from the operator-RHS source packet and
retains the repository NOTICE attribution. The sized wrapper in the unsized
controls has a real `Deref<Target = str>` with static backing bytes. These
fixtures isolate type inference from the allocator.

Pinned rustc accepted the foundation and all seven unsized controls. The
immutable RustGM compiler at ce314b4aa accepted the direct, bound-local and
explicit-generic controls. It rejected all four inferred generic calls at the
identity argument. The sized owner opponent has its own retained independent
rejection, since that call uses a hard expected `View` result. The manifest
declares the candidate outcomes and binds each fixture to its original hash.

The inline-const, macro-place and thin nominal cases are separately declared
source controls accepted by the pinned independent compiler. Weak hints are carried
through groups, ordinary block and macro tails, and if and match branches.
Loop and labeled-break result guidance, and additional slice-element and
dynamic-object controls, remain required.

RustGM passes all twelve declared source outcomes and four existing unsizing,
array and borrow controls, with strict selected Clippy, format, architecture,
harness and upstream verification. Three private controls and full successor
CI remain pending. No native, actual format or full String acceptance is claimed.
