# Transmutability declarations and structured assumptions

LIB-124 requires the actual `TransmuteFrom<Src, const ASSUME: Assume>` protocol
from pinned Rust 1.98.0 `library/core/src/mem/transmutability.rs`. Local source
declarations and assumption decoding are qualified. The all-values relation,
implicit RustGM selection, coinductive witnesses, imported transport and original
core/alloc/std artifacts remain required. PA31 and its parent units stay incomplete.

The [layout-language successor](pa31-transmutability-layout-relation.md) qualifies
the private ABI data kernel; source selection and execution remain open. A fresh
unchanged-core observation on this published header compiler clears the original
declaration frontier and reaches `type_info` in mem/type_info.rs13. The older F5
observation below is historical, not the current frontier.

## Declaration and value boundary

The source registry appends `transmute_trait` and `transmute_opts` at ordinals
83 and 84, retaining every earlier ordinal. The trait must preserve its unsafe,
ordinary, non-auto header, its own unsized-capable `Src` formal and a const formal
whose type is the actual options declaration. Its explicit-implementation, dyn
and coinductive restrictions are authenticated independently. An unrelated trait
or identically shaped options declaration cannot supply this role.

After nominal fields complete, the options role requires a nongeneric module
struct with four source-owned named boolean fields: alignment, lifetimes, safety
and validity. Only then is its nominal role completed in the existing cold ledger.
Decoding consumes the actual canonical struct constant, checks its owner, shape,
boolean leaf types and values, and maps the real field ordinals by those names.
Source order confers no assumption meaning. All sixteen combinations are checked
under both the original and reversed field order, with scalar/tuple opponents.

These checks reuse typed generic declarations, source claims, completed member
facts and canonical child IDs. Fixed-size query scratch allocates no new payload;
the existing source ledger pays validation. Catalog grows 1608 to 1640 bytes and
Reservations 2560 to 2608 because their existing cold role tables gain two entries.
Body remains 456 bytes, syntax Node 36 and NodeFacts 80; the completion bitmap
still occupies two words. Exact record/count guards retain those measured values.

Ordinary source checking retains the default method's union/ManuallyDrop
algorithm. Its authored fixture supplies the genuine NoDrop and Destruct
declarations and const-trait context needed by those helpers. Missing support
in earlier minimal fixtures remains recorded as failed qualification. This
checkpoint supplies no shortcut from one successful value reinterpretation to
an all-values type relation. A closed implicit request still reports the required
layout/validity/lifetime relation; normal caller premises retain their usual proof.

## Evidence

Final fastcheck `1793739270568877422` passes 23 exact tests across three targets,
strict production/private Clippy, formatting, architecture checks and PA21
758/758 in 147.49 seconds including admission. It includes source headers, all
assumption combinations/orders, source checking of the default body and ordinary
program lowering,
wrong-role/header/field opponents, nine existing cold-record/protocol guards,
canonical parameters, wrappers and symbolic operands. Earlier failed runs retain
their actual missing-support, stale-count, lint and output-cap outcomes.

The same final binaries pass the eight existing aggregate native observers and
all 109 constant-machine text cases, with no reference changes. Pinned Rust
independently accepts the authored source declaration, reversed-field declaration
and implicit-use programs, rejects an explicit implementation, and executes the
C-linked default-method observer with status 0, stdout `1\n` and empty stderr.
This is independent Rust behavior; RustGM implicit selection remains unimplemented.

Current immutable CLI SHA-256 is
`5d9a9ce5549a29d4921cf99083edf1b895511cf206ba4a4715dd92578fdb61d7`.
The joined receipt is `obj/parallel/transmutability-v1/joined/receipt.json`, SHA-256
`643bdf7f46a7a1022621f536eaf0da620affc332fca84908c4ed34c45de43872`.
It binds the final source, gate, native/text, independent source/native, record
measurements and all 168 immutable paired timing observations. Frozen controls
cover an ordinary program, a source provider and 64 scalar const arguments;
outputs agree with the previous `466cf52e` CLI. Timing is noisy: one ordinary
paired block reaches 4.33 and A/A spans 0.856–1.282. No precise speedup or
regression conclusion is claimed, and no observations are discarded.

The prior `79a553670` full CI41712407595 passes every gate, including the repaired
fixture-owned bootstrap context. The header publication originally retained the older `f5ae84eb` source frontier.
The layout successor above records the fresh b9 original-core probe; neither
local source qualification nor that later blocker observation is an artifact build.

## Required next work

Implement the all-values layout language and memoized source/destination-state
relation, consuming actual ABI layouts, initializedness, enum tags, union
alternatives and reference descriptors. Preserve compressed repeats and shared
structure; do not enumerate bit values or duplicate expanded layout graphs.
Keep structural cache facts separate from caller-scoped permission and proof.

Pinned `compiler/rustc_transmute` is the primary guide. Ordinary validity checks
all source transitions; assumed validity permits a viable transition. Truncation
and uninitialized destination extension differ from initialized extension.
Reference matching retains size/alignment/uniqueness, recursive type relations,
outlives and shared-destination Freeze obligations. Mutable references require
the reverse relation too. All nominal definitions currently carry potential
safety invariants in the pinned implementation, including public declarations.
An `Assume` value does not waive every obligation. Thirty-two independently
declared pinned-Rust controls already establish these distinctions under
`obj/parallel/transmutability-v1/oracle-v1/`.

Connect that relation to actual coinductive solver evidence and the real source
default method, then qualify genuine implicit RustGM/native consumers and PA29
nominal/structured-value transport. Only those wider checks can close LIB-124.
