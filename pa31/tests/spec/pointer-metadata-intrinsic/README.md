# Source pointer metadata consumer

PA31 binds the pinned safe const ptr_metadata declaration, including its actual
Pointee/Metadata equality, PointeeSized input and independent result formal.
The wrapper uses core::ptr::metadata's original body. Language hooks and ordinary
source declarations provide sizing, copying, dropping and coercion prerequisites;
no library algorithm is implemented by a bootstrap factory.

The candidate is UNQUALIFIED until its focused compiler controls pass. Actual
completed Call/body/Instance proof supplies P::Metadata=M. Extraction consumes
that solved result and the actual pointer ABI layout; it reads the evaluated
pointer once even for unit and never reads its pointee. No additional tail walk,
cache, Pointee owner, hot field or IR operation is introduced. Unit requires the
real thin-pointer ABI; usize requires the real wide-pointer ABI. Unknown or
dynamic output cannot silently produce unit. Current dynamic value extraction
has an explicit storage-producer fence despite its genuine source declarations.

Pinned metadata and native observations, exact source hashes and failed initial
protocol hypotheses are recorded by contract.json. Setup corrections add the
real local stability registration, Destruct const policy, pointer CoerceUnsized
and receiver DispatchFromDyn source impls; test expressions remain preserved.
Native values are 7,6,3,3,7 1,7,91 for the recorded cases. Wrong input and missing
intrinsics reject; a const argument fault remains a fault despite unit output.
The UTF8 length is a byte count. The struct-tail case never dereferences its
intentionally mismatched data address. Foreign thin metadata grants no value
layout/MetaSized fact. The untagged same-spelling function retains its own body.
Original failed source bytes/raw streams and warning differences stay in obj;
no old Try, builtin declaration or Pointee observations were repeated.

The first actual candidate run exposed its supported Destruct source profile:
the initial Rust-positive reduced marker lacked the pinned implementation-denial
and dyn-incompatibility attrs. Those original bytes and actual rejection remain
profile history. The successor supplies the real source attrs. Only the changed
shared provider and one new complete CLI companion were independently checked:
metadata accepts both editions and native prints `7 6 3 3 7 91 1` plus LF. The
companion includes a genuine Rust main and conditional C observer linkage, so
candidate execution uses public LowIR without opening the artifact profile.
