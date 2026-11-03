# Source MaybeUninit, union storage and validity

This draft packet belongs to PA31 compiler/library bindings and consumes PA12
layout, PA25 CTFE, PA26 source traits and PA27 union/ownership semantics. PA31
remains planned. Full original core, alloc, std and general PA29 nominal transport
remain required.

Contract-v12 declares 40 source outcomes and three native observers before candidate
qualification. Pinned Rust independently matches all outcomes. Its native execution
returns `8`, runs one real destructor after extracting an owned value, and sends
the actual uninhabited-type message to the source `panic_nounwind` hook before
exiting 23. All source and observer hashes and primary source hashes are frozen.

The genuine MaybeUninit language item requires a union. Its registry entry has
GenericRequirement::None: the role does not impose one type parameter, fixed
fields, representation, Copy, a constructor or executable methods. The method
cutouts retain the actual new/uninit/assume_init, pointer, write and borrowed
access bodies, with the real ManuallyDrop and MaybeDangling source foundation.
`transmute_neo` executes its actual associated-size assertion and unchecked
transmute; no compiler-provided replacement library body is authorized.

Ordinary unions preserve opaque bytes, initialization masks and pointer
provenance. Only a stored/read selected field demands that field's typed value.
Unread uninhabited alternatives are allowed, including in a union without the
MaybeUninit role. Copying opaque storage grants no initialized value. Unit and
empty-array transmute still require equal layouts and destination validity.
False inhabited assertions panic during CTFE and must use the real runtime panic hook;
ordinary const, unsafe, move, borrow and drop rules remain independent.

Transparent unions require the actual crate feature, even inside a macro with
allow_internal_unstable(transparent_unions). Pinned check.rs applies a semantic
crate gate, unlike token-context gates. More than one nontrivial transparent field
still rejects. The corrected macro prediction and failed independent observation
are retained locally; inputs were unchanged. Earlier foundation attempts omitted
builtin derive declarations, conditional wrapper Copy implementations and Sync;
the array fixtures were corrected to isolate initialization masks using literal
elements, since indexing requires its own source declarations. These failures and
original bytes remain under obj/parallel/maybe-uninit-v2 and were not candidate
oracles. Runtime fixtures include an actual panic hook even for inhabited calls,
as required by the pinned compiler's generic source contract.

The [constant checkpoint](../../../../docs/pa31-maybe-uninit-const-checkpoint.md)
qualifies 37 candidate full-pipeline constant cases and 35 capped/uncapped CLI
cases. Two ownership opponents require the full pipeline. All three candidate
native observers, runtime panic execution and imported library closure remain
required. A semantic-negative case must reject for its own rule; budget
exhaustion, an unsupported prerequisite, panic or timeout cannot qualify it.

The source cutouts are adapted from the Rust project at the pinned revision.
Retain the Rust attribution and licenses recorded in the repository NOTICE and
third_party/sources.json; the contract records primary-file hashes.
