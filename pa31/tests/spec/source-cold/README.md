# Cold advisory source targets

The packet freezes 27 independent Rust 1.98 inputs before candidate observations:
16 accepted inputs and 11 rejected inputs. Five accepted inputs retain separately
named implementation requirements: const-closure, async-block, async-closure,
prototype-required and foreign-required. Header acceptance is not complete body
or native qualification for these inputs.

The reference receipts bind the resolved rustc executable, driver library,
version, exact source hashes, argv, status and diagnostics. The actual no_core
source-native observer independently prints `cold 23 71`. Statement and block-tail
closures without stmt_expr_attributes are valid. A closure in a let initializer
requires that feature. Actual generated-token macro allowance permits it; an
expression forwarded from the caller cannot borrow that allowance. The 64-owner
pair retains one illegal expression among legal statement attributes.

The original core fragment and the pinned ColdParser interval are preserved with
whole-file and interval hashes in provenance.json. The fragment's real unsafe,
bounds and loop body remains required; this packet is an attributed ordinary
closure consumer rather than a replacement for that full source module.

The implementation follows the compiler's existing advisory function contract.
It creates no LowIR cold flag, call/effect/const/unsafe permission, optimizer
promise or exported advisory metadata profile. Actual target, word syntax,
duplicate/wrong-target lint policy, feature and source-origin allowance checks
remain independent of normal body, capture, ownership and lifetime verification.
The maintainer root dev/tests/cold_sources.rs owns these PA31 source-hook fixtures;
PA26 owns the earlier function-advisory fixtures and PA24 owns closure semantics.
