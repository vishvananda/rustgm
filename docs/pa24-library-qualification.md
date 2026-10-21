# Entry-free assignment fixture qualification

PA24 remains planned. The course, independent source and native runners now
accept explicitly classified complete library sources without rewriting their
bytes. The exact contract is in `TESTING_AND_REFERENCES.md`. Native library
qualification requires the existing separately pinned CPPGM object capability;
it validates relocatable objects and makes no execution-equivalence claim.
Earlier program contracts and reference bytes are unchanged.

The syntax runner also honors the resource controller's `RUSTGM_TEST_JOBS`
setting instead of always starting four compiler processes. Explicit job counts
must be between one and eight; the configured local controller still supplies two.

The checkpoint passed all 77 harness tests, layout validation, all PA1–PA23 text
suites, the ordinary executable backend smoke test and the independent C/object
interoperability smoke test. A temporary mixed assignment used the original
14 PA24 output-interface library fixtures plus one existing PA11 program.
All 15 source and native checks passed: ten accepted libraries produced valid
objects with both compilers, four libraries were rejected as declared, and the
program executed successfully with both compilers. Original source bytes were
checked unchanged; the produced objects and executables were removed.

Evidence is retained in `obj/parallel/library-harness-v1/`. Two temporary
post-check assertions were corrected without rerunning completed qualification:
the first cleanup assertion also matched retained `output.lowir` text; the
second source-preservation assertion included non-library fixtures that were
not part of this temporary assignment. Their failed receipts remain available.
Neither failure changed a language expectation, fixture, compiler or oracle.
No planned assignment was activated and no production test manifest was added.
