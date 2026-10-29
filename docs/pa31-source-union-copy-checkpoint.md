# Source union Copy checkpoint

Source Copy derivation admits an actual union field inventory through the
existing paid collector. Its impl header reuses type/lifetime/const formals,
bounds, default stripping and where constraints. It emits an ordinary empty
marker impl. The source Copy checker still proves every physical field and the
actual Clone supertrait. Copying the whole union selects no active field and
generates no Clone body. Other union derives stay explicitly unsupported;
valid Rust union Clone needs its whole-Self-copy recipe and genuine helpers.

Union field eligibility now asks for the registered, validated source Copy
trait application when source language items are selected. Generic source
premises use that same identity. The bootstrap profile retains its canonical
builtin bound. The existing structural marker solver also recognizes unit as
ordinary source Copy/Clone with no children, alongside real function items and
pointers. It grants neither nonempty-tuple evidence, executable Clone bodies nor
const permission. The pinned [Rust compiler rule](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_trait_selection/src/traits/select/mod.rs#L2078)
provides the independent basis; scalar/reference/array source impls remain
separate. No new ordinary syntax/body fields or second proof engine were added.

`pa31/tests/spec/union-copy/maybe-uninit-original.rs` retains unchanged pinned
core `mem/maybe_uninit.rs` bytes: language declaration, Copy derive, transparent
annotations, real unit/ManuallyDrop fields and following authored Clone `*self`.
The original expansion/member/owner receipts pass. The original core capped
observation also passes that expansion boundary and next reaches Debug derivation
for the seven-field `mem/type_info.rs` FnPtr. This is not original MaybeUninit
runtime, language-role/validity/transport or whole-core artifact qualification.

Pinned Rust1.98 independently accepts the SourceCore runtime fixture, rejects
its missing Clone, wrong actual Copy trait, non-Copy field, private access and
unsafe-read opponents, and produces a staticlib. The candidate uses its real
source-to-LowIR producer and the CPPGM backend. Both native observers print:

```text
11223344:11223344:11223344;u;7
```

The uninitialized payload copies never select or read a field. The separate
ordinary Rust union Clone positive establishes only that valid language boundary.
The incomplete NoCore Clone-helper reducer remains a retained failed observation.

Independent v2 receipt SHA-256:
`02882a939cc23e9c85a866bdfa4ab8695684bff4c09ecb0adab99807ba31a8a6`.
Corrected private/wrong-trait opponents are bound separately by v3 receipt
`f5b26bf33245b8ce38db2d95ff9ee710cd82ce75d8781ad7e47e852a43d4f9c5`.
The candidate explicitly demands the ordinary private-access body; lazy unused
body acceptance cannot satisfy an opponent. The v1 Rust helper failure, worker
5/6 check and Root's initial18/19 missing-unit-proof check remain retained.

Joined qualification uses one unchanged source fingerprint:

- Private28,135.13s: `obj/qualification/fast/1793245621390756234/receipt.json`,
  SHA-256 `861a50f57b00b58d84fd050dc92c2224e0eb52dde8c7663a35656af5592af210`.
- Production/CLI4,179.11s: `obj/qualification/fast/1793245760390592131/receipt.json`,
  SHA-256 `8f5f4ac5fadc41cc26365991c3d0d3f9522690b26c7d866de219e3c0a1eaac62`.
- Joined native/upstream/registration/layout,10.67s:
  `obj/parallel/union-linkage-integration-v1/native-joined-v1/receipt.json`,
  SHA-256 `443e09d474202c4364dd8ec3ea750494dc8426af47ac6bc252b920ae9458c2c8`.

Both development partitions include formatting, architecture checks and strict
Clippy for their actual targets. They retain exact C transport, old bounded and
tuple derives, source marker formation/lifetime debts, field/privacy/unsafe
opponents and both diagnostic cap modes. Union native subreceipt SHA-256 is
`f9b8ba6473cab9b4c6f6fb9b8056a08bf2930cb56a80cd32a61b19229e5b182b`.

At8/32/128 union fields, charged work is5567/5687/6167 and bytes
435291/443195/474715. Warm receipt checks add work without bytes. Syntax Node36
and Body456 remain unchanged. Full original runtime, source union Clone,
nonempty tuple marker children, const/conditional proofs, metadata and every
owning assignment/artifact release requirement remain open.
