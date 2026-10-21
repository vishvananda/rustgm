# PA26 equality performance preparation

This record prepares measurement; it reports no timing or instruction conclusion.
PA26 remains planned. The integrator schedules the exclusive measurement window.

The immutable baseline is `a72d8990e422dbd9c6e34542e234279c24c4ba6d` and candidate
is `45870765b493ef74d70e6a5797faec07eed82d58`. Both were built offline from archived
tracked `dev/` sources with the same Rust 1.98 toolchain, default release profile
(opt-level 3, debug 1), no features or RUSTFLAGS override, two jobs and incremental
compilation disabled. Pins, source archives, commands and environment receipts
are under `obj/parallel/pa26/comparison-performance/`. Disposable Cargo targets
were removed after pinning.

| Pin | SHA-256 |
| --- | --- |
| Baseline | `d17b1b17b341828c5cf6100767db30bf4ccf18fccc2da8889af501085ee1a7f7` |
| Candidate | `4a12a4fa2a797a8338eb01c9fb642c115e33a3c539c48ba774b06db895b12bc8` |

Seven frozen inputs independently compile and execute with Rust 1.98 and the
candidate's pinned CPPGM object backend. Five shared controls cover scalar code,
generic Add, assignment dot lookup, mutable indexing and a nominal constant;
all have exactly equal LowIR, exit status and stderr across pins. Candidate-only
widths 32 and 128 exercise distinct nominal implementations, generic equality
selection and provided ne specialization. Each native control checks its values
and exits with empty stdout/stderr.

The original assignment control used `n.add_assign(4)` with `n: u32`. Rust accepts
this, but both compiler pins reject its unresolved integer RHS. Preparation v1
stopped and retained its bytes, declared outcome, raw diagnostics and commands.
The separately frozen v2 control uses `4u32`; its independent Rust and native
qualification passes. This is a real PA26 inference gap, recorded in the owning
remaining-work list, rather than a language rejection. `plan.json` and
`preparation-v1/` retain the original; `plan-v2.json`, `preparation-v2/` and
`complete.json` identify the qualified inputs. No compiler change was made for
this adjustment.

`measure.py` requires an exclusive controller lease and verifies every binary
and input hash. The fixed protocol retains four A/A pairs and five ABBA blocks,
three executions per timing observation, eight RSS observations per label and
one Callgrind observation per input/label. New-feature inputs have candidate-only
observations. Every execution must match its frozen output and successful status.
There are no automatic repetitions. Unrelated host activity remains a possible
source of wall-time noise even within the RustGM-exclusive window.
