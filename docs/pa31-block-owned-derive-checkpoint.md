# Block-owned source derive checkpoint

The unchanged `Span` declaration from pinned core `net/ip_addr.rs:2146` now
retains its block scope through genuine source `Copy`, `Clone` and `Default`
expansion. Each cold derive transaction records the actual module, lexical scope,
parent and scope/caller introductions. Consumption checks the existing scope and
configured-source receipts. Generated impls enter the existing block statement
list through the existing batched attachment pass; block flavor, tail and order
are preserved. There is no per-derive AST census.

Derived record `Default` calls the actual source `Default::default` for every
field. The producer retains the original Default trait, builtin declaration,
macro and usize implementation. Removing that implementation fails the real
trait obligation. It does not replace the generated body with scalar zeros.

The frozen producer, observer and four opposing sources are under
`pa31/tests/spec/block-derives/`. The exact original Span bytes and Default
fragments have separate hashes and source provenance in `contract.json`.
Pinned Rust 1.98.0, edition2024, Linux x86-64 independently accepts the producer,
rejects the leaked name, unrelated same-spelling local owner, illegal nested
generic capture and missing field Default, then links and runs the C observer.
The native trace is `0:0:0;1:7:3;2:7:3;7\n`.

The focused candidate passed strict private Clippy, formatting, architecture and
seven exact controls in268.81 seconds. These include the original Location and
generic fieldwise Clone controls, receipt relocation and attempted/warm payment.
At8/32/128 real local blocks, charged work is14433/45575/172549 and charged
bytes2136547/7626403/29585827. Cold Placement is28 bytes and Transaction200;
ordinary syntax Node36 and expansion Body96 are unchanged.

| Retained receipt under `obj/` | SHA-256 |
| --- | --- |
| `parallel/block-derive-v1/oracle-v1/receipt.json` | `cdb783355415373b99e8f581db544e7367e36a0c2728b2d1ccf30d511ab37d8a` |
| `qualification/fast/1793229483639334333/receipt.json` | `3082721de91782f6721f6288e9413ff8338734223b489baee58f72a11a110de6` |
| `parallel/block-derive-v1/native-observed-v1/receipt.json` | `0fa15706ae19d46531038814c36054fe3e436a876328a1966708d64143d3e9c2` |

The candidate's source-bound LowIR independently validates, produces a CPPGM
object, links to the same observer and produces the exact Rust trace. Its SHA-256
is `154e3381496970c3fc96cf51a74b3b44276f0c58899b72302eb37a583c85f82f`.
Generated objects/executables were retired with scoped guarded cleanup; raw
sources, diagnostics, hashes and receipts remain.

Root production/CLI and original-core integration remain pending. This finite
local declaration does not establish full Ipv6Addr formatting, wider local
generic or closure families, non-record fieldwise Default, nominal metadata
transport, or whole PA26/PA31 completion.
