# Whole-array slice result validity

Original core stopped in fmt/num.rs at the named digit-table slice. A bounded
analysis-only trace found an allocated [u8;2] backing, actual length metadata2
and no explicit range projection. The actual MaybeDangling role enables strict
result-reference revalidation even for ordinary values; role-free slice probes
therefore did not reproduce the failure. The trace is fully withdrawn.

The PA31 whole-slice-validity packet freezes five authored sources and their
independent Rust1.98 outcomes before implementation. Named byte/array/empty
slices reproduce the exact baselinea73fa135 failure; string and ordinary-wrapper
controls accept. A function-body array annotation reaches the same late producer.
A top-level scalar annotation instead stops at a separate early implementation
limitation; that failed reduction remains evidence, not this regression.

Array unsizing may preserve its original zero-offset allocation or array field.
The existing validity consumer now uses its retained length metadata as a prefix
range when there is no explicit range. It still checks actual array backing,
physical element compatibility, bounds, alignment, initialized bytes, static-read
policy, liveness and per-pointer write permission. Byte-address terminals still
fail before this fallback. No metadata is invented from a type, no pointer is
rewritten, and a relaxed wrapper cannot grant ordinary referent validity.

Private fastcheck1794285598981203174 passes16 exact cases with strict private
Clippy, format and architecture audit. New controls cover source acceptance,
original/byte-backed identity, empty/nonempty bounds, denied writes, expiry,
changed initialization masks and wrong elements. Existing string/scaling,
suspended byte-duty and wrapper-policy opponents pass. The first attempt caught
a test I/O error conversion; its failed receipt is retained.

Production1794285826400693246 passes the33 wrapper and37 MaybeUninit source
outcomes. Native1794286021356961497 passes three actual programs: values,
destruction and the expected panic hook. Harness313/465 roots/44 stages and
upstream128 fixtures/126 Reference files pass. No backend contract changes.

Source-bound dev-fast CLI
`61e80685835aaf615a40d2eb527c6d1ef15fad700778f7c6e1bc13f3659c6782`
accepts all five frozen cases. Immutable A/A and two paired ABBA cycles preserve
20 complete-output-equal observations for strict string validation and the74
ordinary controls. Ratios are0.966/1.003, A/A0.973/0.990; no speed or optimized
performance conclusion is claimed. The three declared baseline rejections change
to acceptance and are not relabelled as output-equal measurements.

The unchanged722-input census opens375 sources, checks42310 declarations with
none failed/blocked and bodies0, then advances to num/uint_macros.rs byte75957
(line1824), checked_ilog: source intrinsic constant execution is not implemented.
The uint_impl expansion is at num/mod.rs byte49157. TSV SHA256
`aef48e5a37e6c4e936af398fac7f39769f09213ff230e0a79d1358cc9f1d276a`.
Inputs, profile and limits are unchanged. Inspect the actual registered intrinsic;
is_val_statically_known is the first source lead. This is discovery, not original
core artifact completion.

Raw attempts, traces, source bindings, oracle outputs, checks and observations
remain in pa26-work/obj/parallel/radix-slice. Full predecessor42754442914 has
passed release/course and native maintainer qualification; its broad native,
debug and independent stages remain pending at publication. A successor dispatch
must wait for it to terminate. Whole assignments, HR transport, source-client
exports, optimized nested-proof measurements and original artifacts remain open.
