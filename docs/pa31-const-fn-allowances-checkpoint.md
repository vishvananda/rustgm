# Const-function feature allowance checkpoint

The maintainer implements `rustc_allow_const_fn_unstable` as an exact lexical
function contract. It validates the configured attribute envelope, rustc_attrs
request or actual macro allowance, identifier lists, and real const function or
provided const method body. Multiple lists combine; unknown and empty lists do
not enable features. Nested functions, siblings and inline constants keep their
own contexts. Const eligibility, unsafe permission and imported API feature
requests remain separate checks. An intrinsic still needs its enabled feature.

Unmarked const functions in a staged crate also undergo recursive const API
checking. Their sparse body identities are distinct from explicit API policies.
This prevents an enclosing grant from silently authorizing a nested body.
Parsed attributes and typed declaration facts are reused; no source is reparsed.
Sorted function/feature rows support bounded binary queries. Ordinary sources
still allocate no library policy. Inventory112 and Body456 remain unchanged;
the optional ConstPolicies grows24 to56 bytes. The cold AvailabilityRows gains
one optional pointer; ordinary syntax, functions and calls gain no fields.

## Actual metadata and execution

The existing availability encoding has an optional tag0xEC/version1 trailer
for implicit body identities and exact canonical function/feature grants. It
defines no API feature and confers no declaration constness. Validation checks
real function headers, defining bodies, canonical identifiers, budgets and
unique ordering. Absent extensions preserve older bytes. Unknown versions and
truncation reject; the previous CLI rejects the new extension. Removing the
extension leaves valid older metadata but fails the actual defining-body policy
when the fresh const consumer demands it.

The fixture publishes a stable const caller that reaches a private unmarked
const function, whose own grant permits calling a const-unstable local target.
A fresh consumer evaluates the imported caller as a constant. Both the implicit
body proof and lexical grant must survive encode/decode and remapping. Direct
foreign target use, with or without a client body grant, still requires its API
feature. Both required denials pass.

This consumer exposed premature full-header queries in comptime classification
and const execution admission. Both now read authenticated declaration headers.
The existing body producer still constructs and proves defining environments
before publishing executable Code; its guards remain intact. A debugger trace
records the actual comptime query stack. Provided and ordinary import protocols
remain distinct.

The CLI publishes the provider and compiles the fresh executable consumer.
Its provider metadata is byte-identical to the independently reloaded maintainer
artifact (3823 bytes, SHA25615a3c1c1e0ca4dac45ead77274ef033275d5345874bf1037df7f7dea82db4c68).
Native C observation links the entry-free consumer with the CLI provider: the
executable CLI output has its own main. Both RustGM/locked-CPPGM and independently
compiled pinned Rust print `7 7`. No original upstream library artifact is claimed.

## Qualification and remaining work

- Thirty-five source outcomes match pinned Rust. Seventy capped/uncapped CLI
  observations pass, plus provider publication, fresh consumer and API opponents.
- Twelve private controls pass120.58s;48 production regressions and PA8 126/126
  pass157.18s. Both modes use strict all/pedantic Clippy, formatting and the
  architecture audit. Harness and upstream checks pass.
- Widths8/32/128 charge2915/10835/42515 work and retain2552/10136/40472 grant
  bytes. Each4x increase stays below6x. These are changed-path accounting bounds,
  not an ABBA compile-time claim. No release build or full local sweep was run.
- The packet retains independent source, intrinsic and crate/native contracts,
  source hashes and receipt hashes. Failed predictions, earlier fixture bytes,
  helper errors and the duplicate-main link diagnostic remain in local evidence.
  Whole assignments, broader bodies and all original core/alloc/std artifacts
  remain required. Full CI follows integration.

Local evidence is under obj/parallel/const-fn-allowances-v1. Final private and
production receipts are1793664640355061588 and1793664812117588439. Joined-v2
passes34.81s, SHA2564973e505944cc70483c26001f48984443b3ec728fb284a1b4c344f20e95c3539.

Immutable CLI0cee4b63c2bc8f0809cdbbca67f891f413091819d90a7ce5bf1d7c9daa158be9
clears LIB-121. The original core observation takes16.586262s, opens375 files,
verifies all722 pinned input hashes and preserves target, arguments, capAllow,
work14,500,624/attempt1GiB/retained64MiB and all six configuration source bindings.
It now reports file56:5701, unsupported source language item, at
library/core/src/mem/manually_drop.rs154: `#[lang="manually_drop"]`.
That file's SHA256 is c21100b507893a11ee27b0711be3ac3fe6cc970b96bba55a7d3bae5978209818.
LIB-122 owns actual source ManuallyDrop/MaybeDangling identity, layout, validity,
drop/liveness and imported authority. Registering a spelling or substituting the
bootstrap wrapper cannot satisfy it. The original panic/const_eval_select graph,
other stability forms and whole libraries remain required beyond this checkpoint.
