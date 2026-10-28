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

Root joined production/private/CLI qualification passes17 private controls,10
lexer controls, all4 CI caller regressions and PA6 in190.20s. Both strict Clippy
modes, formatting and architecture pass. Receipt
`obj/qualification/fast/1793231299061764597/receipt.json`, SHA-256
`776a8f869d19c1d890a74ffb271d87a724172161e83d8c7999d7e427a986fa75`.
The initial four-target build budget failure is retained; the exact unchanged
warm retry passes within the same five-minute deadline.

Fresh Root LowIR is byte-identical to the qualified worker output. Its actual
Span native trace matches, as do the prelude/feature/handler baselines, harness,
upstream pins and reference controls. Joined post receipt
`obj/parallel/block-derive-integration-v1/observed-v2/receipt.json`, SHA-256
`386b53cca22833d9a67893aeba4ead53a73ba5d241cd2064a9bb2b478ea23486`,
passes in40.42s. Span native receipt SHA-256 is
`a04697326639c50913562396a489fe9da067c0ef758027ec35ac71d831e2c6a5`.

Qualified CLI `41d3842a0ae000e27ee7e7acea768de7c8c783eeae753de800e7f9380e37b904`
retains separate strict/capped722-pin original-core observations. Strict remains
the deferred pointer-cast expectation after68 source opens. Allow now reports
builtin derive execution on SimdAlign at `intrinsics/simd/mod.rs407`, after372
opens. The retained expansion trace identifies ConstParamTy as the actual
unsupported member, not Debug/PartialEq/Eq. Receipt hashes are
`9c61a4869e82379b8a602061e55012491d70376a409766ef46440ef12b154356`
and `1b38540d9df1969c4e0ecaf0fedfae6ed226fa00d7965b7f7969243103668696`.
Counts describe loading order, not compiled files or original crate success.

The [actual CI caller/metadata repair](ci-40778928096-caller-repair.md) is included
in this joined source; old provided golden bytes are preserved. Full CI must
qualify the new public cut; focused success is not a full-CI claim. This finite
local declaration does not establish full Ipv6Addr formatting, wider local
generic or closure families, non-record fieldwise Default, nominal metadata
transport, or whole PA26/PA31 completion.
