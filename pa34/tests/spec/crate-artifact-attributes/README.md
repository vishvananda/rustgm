# Checked source artifact declarations

`outcomes.json` owns nineteen source/driver scenarios. The original trigger is
libc0.2.185's unchanged `#![crate_type = "rlib"]`, byte83 of its crate root.
The original seventeen declarations and refuted predictions remain recorded.

Inspection checks builtin metadata without requesting production. Source
compilation supplies a typed caller selection, preserving omitted selection,
explicit binary and explicit library cases. CLI selections override supported
source requests. Malformed forms and non-string values still fail. An explicit
bring-up guest is admitted as a library unit; its source rlib declaration does
not publish or qualify a rustc rlib. The ordinary PA24 executable profile stays
closed to source artifact selectors.

Pinned rustc accepted fourteen of the original seventeen scenarios and rejected
three malformed attributes. The initial unknown-spelling rejection prediction
was refuted under `cap-lints=allow`; the separate uncapped probe found the
`unknown_crate_types` lint is deny-by-default. Both declarations and raw results
are retained. Unsupported spellings stay outside this finite artifact profile.

Candidate validation also exposed the existing scalar-pool metadata transport
fence in two strict library override cases. Those rejections remain explicit.
Two matching bring-up controls were declared before their candidate execution;
they use empty skip lists, require no body blocker or unproven row, and require
metadata to remain withheld. Their identical source/CLI inputs reuse the two
existing pinned-rustc observations without another oracle run.

The nineteen cases declare eight RustGM acceptances and eleven rejections:
three shared malformed-attribute negatives, six artifact-policy gaps and two
existing metadata-transport gaps. The latter eight differ from rustc acceptance
and establish no independent language rejection. `review.json` records the
prior seventeen-case checkpoint; the current contract records the transport
correction. No strict publication, native, performance or assignment completion
is claimed.
