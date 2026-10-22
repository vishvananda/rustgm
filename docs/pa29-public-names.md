# Public names and defining identities

This is ongoing PA29 work, not a released profile or assignment completion.
The pinned Reference's `items/use-declarations.md` and
`visibility-and-privacy.md` require a public binding to preserve its target's
identity even when the target's canonical module path is private.

The private metadata implementation carries an optional public namespace separately from
owned function, constant, record and storage declarations. A module retains its
canonical artifact-local path and a sorted table of public name-to-ExternalDef
bindings. Bindings use the target's namespace and kind. The root module is
explicit. Only reachable module tables are permitted; aliases to modules reuse
their tables rather than recursively expanding paths. Resolved module aliases
may form a graph without creating an unresolved import cycle.

An owned declaration remains encoded once under its defining crate/path/kind.
Dependencies retain their own declarations and symbol selection. Public aliases
grant name access, never a second nominal identity, copied global, function body,
const permission or source node. An external target must belong to a declared
dependency and be publicly exposed by its defining artifact. Hidden layouts and
backing globals remain private. Reexporting an unsupported semantic contract
must retain that contract's existing diagnostic boundary.

Profile six envelopes the existing profile's typed payload and appends the
public module tables. Profiles zero through five retain their exact bytes.
Bound lengths, name storage, graph work, duplicate namespace names, local target
existence, dependency ownership and reachability before resolver allocation.
Canonicalize once; use indexed module/definition lookups and a bounded worklist.

Required source/native evidence includes renamed local exports from private
modules, original-path privacy, repeated aliases, module and glob reexports,
transitive external targets, same-spelling distinct crate identities, namespace
separation, exact symbol/address identity, private target rejection and ambiguous
imports. Freeze outcomes and qualify unchanged Rust sources independently before
candidate execution. Retain reordered-dependency and deterministic-byte controls,
malformed/truncated wire cases and shared 8/32/128 alias graph scaling. Codec-only
tests cannot establish source support. Source export/import, full checkpoint
qualification and assignment activation remain separate required steps.

## Private checkpoint qualification

`obj/parallel/pa29-public-names-v6` records all 30 metadata codec tests passing,
including wrapping each existing payload profile, exact preservation of its
legacy bytes, malformed/truncated envelopes, target privacy, duplicate
namespace rejection, dependency exposure and shared 8/32/128 alias graphs.
All-target/all-feature release Clippy, the architecture/file audit, harness,
layout, upstream attribution and suite registration checks passed. The final
receipt binds the tested source hashes and each command result.

The independently frozen contract in `pa29/tests/public-name-outcomes.json`
contains 12 unchanged sources. Pinned Rust qualified every declared result and
both executable clients. Its sources, diagnostics, declarations and hashes are
retained in `obj/parallel/pa29-public-name-sources-v1`. These establish the target
behavior; the RustGM source export/import implementation is still pending.
This private checkpoint is not ready for publication or assignment activation.
