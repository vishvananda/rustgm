# Source C variadic cursor checkpoint

This finite PA31/30/28 checkpoint extends stage 1 (1ef2057444, integrated as
4cd080c84) with actual C variadic bodies and the ordinary c_void enum producer.
It preserves the original VaList, VaListInner, VaArgSafe, variance marker,
PhantomData, Clone, Copy, Drop, next_arg and intrinsic declarations and reached
bodies in an explicitly attributed active source cutout. Complete original module
and variance-macro bytes remain in the fixture's original inventory. This cutout
is not a complete unchanged standard module.

The real ellipsis parameter receives an internal inference region and one owned
VaList local outside the fixed public signature. One borrowed Start plan checks
current source/header/body/parameter/type/binding/region identity and the unique
entry StorageLive before initialized storage and typed LowIR va_start emission.
The original source parameter list includes the ellipsis; the callable type has
only fixed ABI inputs and no extra public binder. Normal ownership, loans,
source destruction and storage scheduling remain in force.

VaList target authentication uses its completed actual declaration template and
full marker → covariance carrier → PhantomData relation, existing variance facts,
and the independent actual lifetime application. Physical region-only field
shapes are not mistaken for semantically substituted field views. Real x86-64
cursor fields and layouts must give size 24/alignment 8 and offsets 0/4/8/16.
No builtin nominal or marker alias supplies that storage.

The va_arg descriptor authenticates the actual intrinsic declaration, complete
instance, opened mutable VaList argument, result and original call/report/effect
proof. Its two public late slots are distinct; valid concrete lifetimes may be
equal. Only authenticated direct calls omit callable-body demand. va_copy and
va_end execute original source fallback bodies through ordinary selection,
original/instance and reached-body proof. No memcpy or no-op replacement is
introduced. The shared discriminant opening law retains its earlier diagnostics
and actual source consumers. Genuine const Drop owners pass ordinary impl
conformance and exact owner constness checks; this grants no CTFE execution.

The qualified gate is obj/parallel/source-transport/va-run/gate9/receipt.json;
strict formatting, architecture and Clippy are in static11/receipt.json.
Seven regular integration tests include eight positive LowIR consumers, thirteen
intended-cause cursor/Drop outcomes, sixteen original positive feature requests,
actual source discriminant regressions, fixed-signature checks and bounded work.
The separately named required-gap test is not Rust semantic parity. The initial six native
programs match independently frozen Rust observers:

| Input | Native stdout | Observation |
|---|---|---|
| run | 17 11 17.5 23 23 29 31 | integer/float/pointer, register overflow, clone independence, early return, owned forced-indirect VaList |
| rzro | 37 | zero fixed arguments |
| cvloc2 | 1 | real c_void constructor, move, Rust call and discriminant cast |
| rorder | 41 41 77 47 | operand-dependent ordinary const Drop order and early-return cleanup |
| rfixed | 31.5 | fixed floating argument before floating variadic reads |
| rfrom | 17 | actual Rust source caller of the C variadic definition |

The native handler successors contain a genuine application panic_handler that
calls the existing actual abort intrinsic. The initial link failure without a
handler remains recorded. Reference objects explicitly use overflow-checks=no;
rorder additionally uses debug-assertions=no after checked setup required
mem_align_const. Inputs are valid and aligned. Checked arithmetic, alignment,
null instrumentation and panic parity are not established by these observers.
No panic helper was fabricated and no backend or pin changed.

Canonical work is 10684 at width 0 and grows 114/378/1434 for 8/32/128 distinct
cursor bodies, with exactly one emitted va_start per demanded function. This is
a bounded-work observation, not an ABBA speedup or general regression claim.
Old Va 33-outcome, Index and ten-assignment controls, upstream verification and
maintainer registration also pass. Full private/release qualification remains CI.
The evidence JSON binds compiler/fixture/native hashes, reference and local
failure receipts, and immutable backend tools.

The initial original cvoid.rs by-value C ABI gap is now closed by the
[fieldless enum checkpoint](source-c-enum.md), with original source and observer
unchanged. The initial gate9 evidence above remains a historical qualification.

Required work remains:

- cldef.rs and clcall.rs are Rust-positive. The shared nominal const destruction
  eligibility fence rejects the original concrete const definition. Proper
  nominal/field/actual const Drop proof must precede removing it; actual cursor
  CTFE state/init/read/copy/end/cleanup remains required separately.
- Full unchanged standard module/macro closure, imported/source-serialized cursor
  locals and operations, generic variadic definition specialization, other target
  cursor ABIs, Miri semantics and unnamed-ellipsis lint policy remain required.
  Existing wire profiles explicitly reject the new local kind; old bytes and
  interpretation are unchanged.

Initial setup/prediction failures remain, including unknown copied API features,
missing setup imports/stability, the original checked object failure, a rustc
ICE101 after a non-const Drop mutation, checked alignment setup, and a separate
observer's duplicate user name. The corrected ordinary non-const-trait opponent
is independently rejected without an ICE. Infrastructure failures are not
semantic rejection evidence.
