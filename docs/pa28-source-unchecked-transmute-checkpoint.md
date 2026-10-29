# Genuine unchecked reinterpretation and NonZero methods

Source TransmuteUnchecked now has its own attributed declaration identity.
Ordinary Transmute retains its pre-monomorphization size relation; unchecked
reinterpretation permits generic and unequal-size body typing. Reached constant
size mismatch faults, and unsupported runtime layouts remain an explicit
execution-profile boundary. This does not turn a Rust-positive declaration into
a language error or permit invalid bool/char/reference/nonzero values.

The actual unsafe const, bodyless Rust signature has two own ordered Sized
parameters, one Src input and Dst output. Complete instances, opened signatures,
current/foreign report identity, caller unsafe permission and source API/const
policy remain independently checked. The genuine indirect-const marker differs
from ordinary transmute stability. Runtime/CTFE share existing owned storage,
byte codecs, destination validity and provenance duties; source moves still
avoid an extra destructor. Old authored export profiles explicitly reject the
new declaration semantics. No metadata tag, Body/Code/Rvalue field or second
layout/provenance cache is added. Selection16/Node36/Body456 remain unchanged.

The PA31 source consumer retains unchanged pinned outer NonZero new/get bodies,
the real ZeroablePrimitive contract and selected u8 associated-type instance.
Its inner carrier is explicitly closed `u8 is 1..=255`; this does not qualify
the original open helper or full NonZero/inner trait graph. Actual CTFE and native
calls distinguish None, Some1, Some255 and get. The C observer matches the valid
independent NoCore Rust program: `0 1 255 0 1 255 255\n`. Forged Some0 faults on
the actual nested range. The diagnostic name/documentation attribute grants no
physical encoding by itself.

The component19-case gate passes159.37s, including checked-transmute size,
borrowed-binder, move/drop, validity and indirect-const controls. Receipt
1793286733015988270 SHA-256:
`502a151e57947c77d4cc4bb7d2e3e29ad332a571a494e9ea9d2764efd7cfe378`.
Actual pinned-backend native receipt SHA-256 is
`6b65c9634531f258152ba8598b47ab5a82dc1058493267da71b47c69b2775ce4`.
The authenticated handoff is `obj/parallel/source-transmute-unchecked-v1/handoff.json`,
SHA-256 `d6a466489ddbd61227b39eee4f06857f036d5e53789f94289dd8a8245324133e`.

Those controls also pass in the joined87 private gate; its fresh NonZero LowIR
hash equals the native-qualified component hash. Root production/CLI3 passes
with strict production Clippy, full architecture and format checks. See the
[joint checkpoint](pa8-derived-constants-checkpoint.md) for exact joined proofs
and the unchanged-original-core observation.

Cold timeouts and initial fixture/assertion failures remain failed receipts.
A runtime-only nonconst attributed declaration is a Rust positive outside the
finite original-const-header profile, not a Rust negative. An actual const call
of that nonconst declaration independently rejects. Private source-typing views
and native consumers do not establish a released course manifest; the proposed
manifest rows remain proposals. General runtime mismatch handling, original
open/signed/char and wider carrier graphs, other NonZero methods, full body/value
transport, original core/alloc/std artifacts and every whole-PA requirement remain
required. No assignment is activated by this checkpoint.
