# Inherent member identity checkpoint

Inherent method lookup now partitions declaration names by normalized item
context before shape filtering or applicability. Receiver and associated-path
uses retain the actual method identifier's context and caller scope. Lookup
visits that context's ancestors, adjusts against each actual impl container,
and checks visibility for the actual caller. Coherence compares declaration
contexts directly; it does not apply a caller's access rules to declarations.

This is one LIB-002 component. General scoped macro invocation remains disabled.
Trait discovery, explicit trait members, implementation conformance and associated
members still need their context-aware consumers. The seven independent
[trait-member contracts](../pa31/tests/spec/declarative-macros/trait-members/README.md)
show that an opaque trait binding can contribute a captured method. Filtering
traits by the trait binding's context would therefore be incorrect: the method
identifier controls scope traversal and member matching.

## Representation and work

Ordinary methods retain their existing flat nominal/name buckets and shape
indexes. Non-root contexts allocate a separate partition beneath the existing
lazy method lookup owner. The owning `Methods` remains 312 bytes and
`Declarations` remains 7,064 bytes; the checked use carrier is 8 bytes. Root
methods alone allocate no context index. Failed ancestry attempts count toward
the existing explicit work bound before returning an error.

Private width controls at 8, 32 and 128 sibling contexts visit 8, 32 and 128
ancestors respectively. They verify the index algorithm, not source-level
scoped invocation or full compile-time performance. The ordinary path and
complete source scaling still need paired measurements at combined integration.

## Component qualification

Qualified source is `8e798fcdaa050b4fe7db1634ba3be94ac91a691f` in the
`rustgm-pa29-work` checkout. The three implementation commits are `a97bde6e`,
`1b03cf2d` and `8e798fcd`; their exact combined compiler diff is integrated as
`ebceca4e`, `84c3fdbf` and `abae2094` after the independent cfg_select component.

Fresh format, architecture/file audit and all-target/all-feature strict Clippy
passed in `obj/parallel/member-identities-static-v5/`. The final runtime batch,
`obj/parallel/inherent-identities-runtime-v3/`, passed all **1,443 nonignored
private cases**, **28 affected integration cases**, **seven native test
functions**, and **8,382 text cases across PA1–PA24**. All six new identity,
container, layout and attempted-work controls are required by name in the
private inventory. The final CLI SHA-256 is
`3b48421d23c45a560761b991748367eda78b4b638918d4e7b4ea380de160c2fb`.

The runtime completion SHA-256 is
`e1293b53af80c96545643ec59f2cd2529895a111b170d89ca231682facf6acfc`;
its checks SHA-256 is
`501126ca857fa8213ec95e1d409ab734fb593c984905af795c426180d55f665d`.
Guard `5ebb31db76f84202ad6af05ef6e012d2` finished with status 0/error null under
the 12-GiB absolute cap and unchanged 150-GiB reserve. Cleanup receipt
`1792863197975134995` removed 334 generated files, 0.70 GiB of unshared blocks.
The 35 recorded stages took 314.81 seconds in total, including a 174.15-second
debug build and 67.93 seconds for all text cases. These are shared-machine
qualification durations, not controlled performance comparisons.

## Retained failures and remaining gates

The first runtime batch exposed five declaration-layout assertions after a
new field grew the owner from 7,064 to 7,072 bytes. Moving the optional context
data under the existing lazy owner restored the layout without changing those
assertions. The second batch passed private, affected and native checks, but
its text launcher attempted an unnecessary nested storage/build wrapper and
was rejected before text execution. The direct text recovery then found one
changed PA19 private-receiver diagnostic. Early filtering now retains the
distinction between inaccessible candidates and no declarations, preserving the
committed diagnostic. No text reference was edited.

The final batch uses the already built CLI directly for every course suite.
The two refused admissions before that batch launched executed no tests and are
retained separately. Combined release qualification and shared-path performance
remain pending; this component does not release an assignment or build core.
