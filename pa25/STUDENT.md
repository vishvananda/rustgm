# RustGM Programming Assignment 25 — Constant evaluation

Build on PA18's generic bodies, PA20's associated items, and
[PA24](../pa24/README.md) to evaluate the course's const-function subset.
Use Rust 1.98.0, edition 2024, on Linux x86-64 with checked arithmetic and
aborting panics.

## Assignment scope

Implement the typed constant machine exercised by
[the PA25 manifest](tests/manifest.json). It covers const-function eligibility,
evaluating checked constant bodies, and inspecting the resulting values.

- Check every const body, including unused ones. Reject calls that are invalid
  in const contexts, including when a branch would skip them. Support
  direct const calls, selected inherent calls, eligible constructors and
  recursion with determined arguments. Support ordinary runtime calls to
  const functions through the existing lowering pipeline.
- Execute typed statements, loops, scalar operations, assignments, array
  construction, and the supported local borrows. Evaluate only the selected
  branches and short-circuit operands. A reached overflow, divide-by-zero
  operation, invalid access, or forbidden escape is an error. Check the
  declaration and referent permissions before using a borrow.
- Evaluate closed required initializers and repeat counts. Preserve typed
  identity across selected calls and initializer dependencies. Diagnose
  dependency cycles. Defer undemanded generic and associated initializers
  that need additional arguments.
- Preserve ownership and cleanup obligations. A const function can transfer
  an owned result to its caller. A value that needs runtime destruction
  cannot be discarded during constant evaluation. Check the source rule
  even for a zero-length repeat that evaluates an operand.
- Preserve exact scalar bits and the active fields of aggregate values,
  including the supported typed const parameters. Keep repeated elements
  as one value and a count. A pointer to an allocation retains its storage
  identity and access provenance.
- Bound evaluation frames, executed steps, and retained storage. Run
  recursion and loops with iterative frame/step bookkeeping. Reject an
  evaluation that exceeds a limit with a diagnostic.

PA44 carries the broader constant-memory and callable requirements, including
general symbolic allocation graphs, const closures, and wider source-library
integration.

## Command-line interface

```text
rustgm --emit-constants [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
rustgm --emit-lowir [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input and output options. `--emit-constants` checks a crate and
reports its completed value graph. It accepts a source file without `main`.
Use `--emit-lowir` for executable cases. A source error exits 1 with a
diagnostic and empty standard output.

## Constant inspection format

Begin with `constants 1`. Emit declarations, roots, typed value rows, and
allocation rows, then close with the summary.

```text
declaration def=D kind=function type=T const=BOOL status=STATUS
declaration def=D kind=KIND type=T const=true status=STATUS
declaration source=FILE:NODE kind=inline type=T const=true status=STATUS parent=FILE:NODE
root initializer def=D type=T value=vV storage=none|aA
root associated-instance instance=D:ARGS type=T value=vV
root inline source=FILE:NODE type=T value=vV
root promotion owner=OWNER instance=none|T expression=E type=T value=vV
value vV type=T PAYLOAD metadata=METADATA
allocation aA type=T STORAGE
eof declarations=N roots=N values=N allocations=N
```

Order named declarations by definition ID and inline declarations by source
identity. List closed initializers first, then completed associated values,
inline values, and proved promotions. A deferred declaration has no value root
until evaluation completes. Number values and allocations separately from
zero in breadth-first discovery order. Seed the graph with the roots in order,
visit product fields in stored order, and share backing allocations by identity.
Hexadecimal payloads use lowercase digits and the width given by the fixture.

The [entry-free example](tests/spec/inspection/library.rs) computes a scalar
constant and takes two shared references to the same static. Its
[expected output](tests/spec/inspection/library.rs.ref) shows four initializer
roots and a single backing allocation. The complete value and allocation
spellings for other forms appear in the corresponding fixture outputs.

## Run the tests

Run the PA25 test target from the repository root.

```sh
make -C pa25 test
```

The manifest selects the inspection or LowIR mode for each case. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for executable cases.
