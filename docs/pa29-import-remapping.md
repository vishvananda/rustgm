# Validated dependency remapping

`support::crates::Dependencies` canonicalizes artifact identities and direct
aliases once. Before resolver allocation it rejects missing dependencies,
duplicate identities/aliases, foreign alias targets and dependency cycles.
The iterative graph pass visits each vertex and edge once per construction
phase, after sorting. Graph limits are 1024 artifacts/aliases, 100000 function
rows, 1000000 dependency edges/path/type atoms and 8000000 name bytes across
the retained artifacts. Interface atom/name totals are computed at publication
and reused without rescanning each function for every graph consumer.

`resolve::expansion::analyze_with_crates` installs the validated immutable graph
before initializing expansion's import work. Existing entry points delegate
with no graph and retain the ordinary path. `Resolution.crates` shares the
immutable artifacts and owns an explicit `ExternalDef -> DefId` remap, with
local function-table coordinates. Root/module/function identities are canonical
under reordered artifact/alias arguments. Only supplied direct aliases enter
the extern prelude; transitive artifacts retain hidden canonical definitions.
Imported definitions have no AST node or invented body. Builtin metadata reuses
the existing extern scope so preparing Option/core does not erase imports.

All declaration entry points seed imported closed scalar signatures and mark
their definition states Complete. Nongeneric signatures do not create generic
templates. `Registry` receives canonical source-free Import records with
effective unsafe permission, Rust ABI and Rust/Exact naming. Exact bytes are
reinterned in the destination symbol pool. `Declarations.crates` retains stable
Rust spellings; imported functions never become local function/body entries or
const-eligible callees. A const bit cannot grant execution without a checked
body, and this initial metadata profile rejects that bit at decode.

Validation passed check, 13 focused import/codec/root tests, strict all-target/
all-feature Clippy and the architecture/file audit. Pinned rustc independently
built both libraries and qualified six client outcomes: three accept (nested
imports, explicit extern rename, nested local module calls), and three reject
(unsafe call outside unsafe, missing private export, ordinary imported call in
a constant). Graph controls cover missing/duplicate/cyclic inputs and chains
of 8/32/128 artifacts. Canonical local remaps agree under reversed inputs, and
only the client's actual main appears in local function facts. An initial
test-only private-field access and unnecessary empty generic-template
registration were corrected; their failed logs remain in the evidence folder.

Evidence is `obj/parallel/pa29-imports/`, including qualification sources,
diagnostics and declared outcomes. This remains an import interface checkpoint:
crate-aware lower naming, source export collection, CLI artifact transport and
multi-crate native values follow immediately. Existing PA25 boundaries remain
listed in `pa25-initializer-validation.md`; neither assignment is complete or
activated.
