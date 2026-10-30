# Reservation implementation checkpoint

Local compiler source3d2871da7 implements actual reservation polarity and
coherence without using a reservation as positive trait evidence. It checks
ordinary method conformance and physical Copy requirements, preserves real
associated-parent proof, and checks unused const definitions through the same
retained defining CFG owner used by demanded constants. Concrete nongeneric
impl exports use full callable ABI conformance. Inherited lifetime parameters
remain inside the existing generic-export fence.

The expanded exact32 check passes in112.14s, including all original25 and seven
ABI/ordinary-const/Always controls, strict Clippy, formatting and architecture.
Receipt: obj/qualification/fast/1793339544859577425/receipt.json.
The affected constant-pattern/linkage suites pass9 exact tests in229.83s;
receipt: obj/qualification/fast/1793339807540058061/receipt.json.

Three real native producers match pinned Rust with7/7/7. The37 actual backend,
CLI, old-policy and source-control operations pass in2.29s; receipt SHA256
63913a5e0ac1d5a61ead0816de6b0a3f422990693010e2e66360e121947e2ff2.
These are independently generated objects, not a frontend fallback.

Fresh pinned Rust metadata observes the declared PA20 closed-local/coherence
cases with statuses0/1/1 before the owning runner generates exactly nine new
sidecars and one partial provenance batch. Previous references and compiler
sources remain unchanged. Generation receipt SHA256
39e969ef9b8bfcad2e72232defb2c4f800d911cf022fbab7dafbeaeaae6bcb0f.
Manual review confirms that both rejections are actual overlapping trait
implementation headers. Final full harness275, registration385 roots/13 bins,
assignment layout and upstream24/Reference126 verification pass.

This is a local source/native checkpoint. Library metadata still explicitly
rejects reservation implementation transport. Original core, alloc, std and
whole PA20/25/29/31 release requirements remain open. A fresh unchanged core
probe and broader CI qualification remain separate acceptance evidence.
