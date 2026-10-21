# Captured output and execution signature checkpoint

A closure whose output lifetime comes from a generic callable expectation now
retains that lifetime in its checked execution Self. Its public callable binder
keeps declared quantified inputs local; captured body universals sit beneath
that binder. Reconstruction uses the completed body signature and the retained
opening map, preserving unused slots and nested callable binders.

An imported caller lifetime that occurs in the output now uses the existing
conditional output interface. It retains its original parent identity instead
of allocating an unrelated region. The parent must discharge the child's
conditional relationships. Public quantified input identities never become
parent proof bridges, even when they occur in the public result. This also
rejects the parent-binder collision control through the lifetime proof.

`pa24/tests/execution-self-outcomes.json` declares eleven whole-program cases:
six accepted native controls and five lifetime/loan rejections. Pinned rustc
full-object outcomes agree. The type-only view retains pending proof duties and
accepts the five negatives; LowIR and constants must reject them. The initial
uniform phase expectation was wrong, and its failed receipt remains alongside
the correction. No executable expectation or source was weakened to accept a
failure.

The focused combined guard passed strict release Clippy, the file audit (1,380
files, zero findings), layout/upstream checks, 534 library invariants, 113 affected
maintainer groups including native execution, and all 7,787 active text cases.
The typed mixed-output control separates a quantified input from a captured
output. Widths 8/32/128 check additive canonical work and bounded interface ports
and bridges. No matcher equality or nested-binder rule was relaxed.

Evidence: `obj/parallel/pa24/execution-self/combined/complete.json`; debug
executable SHA-256
`1e27fbab536b63e0ccd0c845f545485e1f254808166873cac70d63d9fe87e123`.
Earlier diagnostics and phase review are retained in the same evidence root.
Public combined release and timing qualification remain pending. PA24 is planned;
generic-owner captured execution and dynamic callable adapters remain separate
work.
