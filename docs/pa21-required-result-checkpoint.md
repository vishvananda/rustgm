# Required opaque result equality repair

Required ordinary trait methods now expose their published associated equalities
through the existing proof/normalization path. `Search::start` short-circuits on
its existing normalized-input evidence before asking for a structural head.
A fresh inference context need not expose the enclosing normalizer's opaque
head. No result flag, hidden representation or caller permission changes.

Six authored PA21 controls independently match pinned Rust: borrowed/value
consumers pass; wrong Output, distinct method identity, distinct Self identity
and a borrowed-result static escape reject. The public test follows complete
LowIR construction, including defining/region/loan proof. Semantic analysis and
`pipeline::analyze_source` alone retain lifetime obligations and are not claimed
as escape qualification; those failed test attempts remain recorded.

The focused gate passes8 exact tests across two targets plus1193 PA19 cases in
48.81s, with formatting, architecture and strict selected Clippy. It includes
existing opaque identity, Deref, receiver and Pin controls. Authorized selected
reference generation adds only the six PA21 text oracles and a scoped provenance
batch. Both positive programs execute cleanly with pinned Rust and the locked
CPPGM backend; the four negatives independently reject. Upstream/harness pass.
The two previously qualified Deref/receiver native LowIR programs are unchanged.

Candidate CLI SHA256: `992476748c08eb2b153b64e2489e5a5d7013ad302f1e08fe76b9e208f17a531a`. Gate receipt
`obj/qualification/fast/1793839385358734829/receipt.json`:
`794e1d879b2973dc237ef91b923c2705b02f1ac6c6f856c7297acd1548f75f99`. Qualification/native/independent receipt
`obj/parallel/required-rpit-v1/qualification-v1/receipt.json`:
`a14b8ae6ef3e32399346bc1bf326528ebaf459e18a560c5b1ca7726766aa06d1`. Earlier frozen predictions and failed
attempts remain under `obj/parallel/required-rpit-v1`. The async worker separately
confirms actual source Future and async trait consumers with this shared repair;
that unmerged signature checkpoint supplies no state-machine execution claim.

Full Pin successor41939550904 is now terminal success in every gate, including
private/release/course, native, debug, independent Rust and const qualification.
It precedes these changes. No full successor, original library artifact or
additional whole assignment is inferred from local selected checks.
