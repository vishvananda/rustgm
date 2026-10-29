# Bounded derives and public transparency

Source derive impls now preserve parsed type/lifetime bounds, ordinary const
formal types and where predicates. Target applications contain parameter names;
source defaults remain on the original declaration and are omitted from impl
formals. Each derive still adds its actual trait requirement. Existing paid
typed splices preserve original constraint syntax and hygiene, choosing path or
type fragments from their parsed roots. Higher-ranked and unsupported transfer
forms remain explicit requirements. No source text is reparsed by semantics.

The unchanged four-member `ManuallyDrop<T: ?Sized>` declaration and its original
`MaybeDangling<T>` field are retained under `pa31/tests/spec/bounded-derive/`.
Its Copy/Clone/Debug/Default list and generated bounds have real expansion receipts.
The native bounded/where/type-default/const-default source calls actual non-Copy
field defaults, matching pinned Rust's trace:
`d7;d9;p7:9;d7;r7:5;7\n`. This does not establish candidate DST Debug/Formatter or
the complete original MaybeDangling/ManuallyDrop runtime; those remain required.

`rustc_pub_transparent` is recognized and validated on each active configured
owner using resolution's retained-attribute interface and the already parsed
representation. It requires word form, its actual `rustc_attrs` request or
authenticated macro allowance, and a nominal `repr(transparent)`. Duplicate
markers are hard errors. Field privacy and ordinary layout checks still apply.
There is no new dense field, permissions table or scalar validity shortcut.
The actual configured marker remains on its owner. Existing metadata rejects
transparent records; external private-field eligibility cannot be silently lost.

The pinned parser permits only struct/enum/union targets and rejects duplicates;
[the attribute check requires transparent representation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_passes/src/check_attr.rs).
The marker also affects [eligibility of external private/non-exhaustive zero-sized
types](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_hir_analysis/src/check/check.rs),
so transparent metadata remains a semantic requirement. Rustdoc rendering and
broader advisory parity are deferred; no advisory producer was added here.

## Evidence

- Fifteen exact tests across private, frontend and trait targets pass in156.66s,
  including production/CLI and private strict Clippy, global format and architecture.
  Receipt `obj/qualification/fast/1793242897130433575/receipt.json`, SHA-256
  `d8bdfa7a03696184a6e499a294b93eb808a75538e617cf07869f14af728b54ae`.
- The joined native/upstream/registration/layout check passes in6.73s. Both the
  ordered bounded-default trace and the public-transparent record's native7
  match independent Rust. Receipt
  `obj/parallel/bounded-derive-integration-v1/native-v1/receipt.json`, SHA-256
  `c8c051aec4cf9fff0c0e5dd4ce6bf1911df8bf6f159e33249bc3c31b10c9e054`.
- Worker bounded-derive Rust oracle v2 SHA-256
  `2192640aecb3937017c7a387db3aa70ce5d8958c17c459a1cd94d231cd4c4f5b`
  proves the original expansion, authored bound/where/default opponents and native
  trace. Public-transparency oracle v2 independently passes18 language cases and
  native7 in21 operations; SHA-256
  `c05934b5d88c8a7fed14b796a896683591844a50e1c26eb6950a576aa048bf7d`.
- Actual8/32/128 predicate work is9798/11334/17490 steps and
  971099/1164315/1954587 charged bytes. Warm receipt checks allocate no new bytes.
  Body remains456 bytes and syntax Node36.
- Earlier failures remain recorded: worker static truncation and4/6 intermediate
  result, Root architecture correction, overly broad fixture-foundation AST
  assertion, and the unchanged-source build-budget failure. The final AST checks
  use the four real derive receipts, excluding foundation `Location` expansions.
  The actual build timeout was276.45s; the unchanged warm retry passed without a
  deadline extension or a reduced selection.

Full CI40805449339 on the preceding public `fdbaf791f` passed release/course,
native, debug and independent checks. It is not full qualification of this newer
source; broad checks for the new checkpoint run in CI.

## Original core frontier

Qualified immutable CLI
`048223c6280d9f153370bbe9ea1bff4c907562d4fdab164cad8acf5ceabc9f65`
observes all722 unchanged source hashes and the same effective target arguments.
Explicit Allow passes the old bounded ManuallyDrop boundary and stops at
file57:14105 in `mem/maybe_uninit.rs353`: source Copy derivation on a union needs
implementation (LIB-072). It opens373 sources. Strict separately opens68 at the
deferred genuine `implicit_provenance_casts` expectation, with no fabricated
fulfillment. Receipt SHA-256s are respectively
`86d8fab040e56c4dc39214e36da966ea0d908277978c3036333066c9d637a0f7` and
`d0cee9a298af2c71c197bf059b0b75077bc00c8eca6fa871798b6a7327f440bc`.
Loading counts are not compiled-file counts or an artifact. No whole assignment
or original core/alloc/std artifact is complete.
