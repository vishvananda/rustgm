# Exact C source linkage

`contract.json` freezes every input before observations. The provider preserves
the literal bodies of the independently qualified `scalar-body-values` source;
its declarations use distinct exact C symbols. The client retains the existing
seven C wrappers, and `observer.c` retains the existing observer bytes.

Pinned Rust and the actual version5 provider/codec/fresh-client/LowIR path both
produce the declared values through separate provider and client objects. The
provider alone defines the exact imported symbols. A second distinct provider
using those symbols is a real link collision opponent.

See [the checkpoint](../../../../docs/pa31-source-exact-linkage-v5.md) for source
ownership, paid atomic publication, measured records, passed evidence and the
pending joined production/CLI check. Version5 remains unpublished. These inputs
are linkage evidence; the unchanged original `Value` nominal7 fixtures and their
real source-provider/nominal transport boundaries remain Required.
