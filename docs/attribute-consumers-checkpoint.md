# Attribute consumers and configuration demand

The maintainer frontend now distinguishes bang and attribute macro declarations
and lookups, consumes `must_use` and inline attributes, handles trait diagnostic
messages and removes disabled test functions without losing persistent warnings.
This checkpoint does not enable procedural macro execution or release PA31.

Configuration below a removed test function has an authenticated, borrowed
no-demand receipt. Only strict descendants of the actual function body qualify.
The body's own attributes, declaration header, ancestors and live siblings retain
their ordinary feature checks. Receipts authenticate saved uses, captured trees,
the source image and real parent/child edges. Shared parent paths are checked
once; warm uses do not rescan a whole function or sibling list.

The original syntax journal preserves both active and owner-enabled flags with
first-save-wins behavior. Fresh module instances restore the original owner
prefix, including generated functions that replaced old macro-call nodes.
Retirement preflights work, storage, indices and publication before mutation.

Focused evidence, with failed attempts retained:

- [Ten private receipt and provenance controls](https://github.com/vishvananda/rustgm/actions/runs/40172909557)
  pass with formatting, architecture audit and strict Clippy in 115.70 seconds.
- [Nine frontend controls](https://github.com/vishvananda/rustgm/actions/runs/40174042960)
  pass in 62.40 seconds, including driver warning/error transport and strict
  production/CLI Clippy. Reference metadata defects found afterward were fixed
  separately; those jobs' final status is not a passing whole-job claim.
- [Selected generation](https://github.com/vishvananda/rustgm/actions/runs/40176128203)
  checks all 91 remaining source contracts before writing any oracle. Together
  with the two PA7 cases, 93 new active cases are backed by 372 retained rustc
  metadata/object observations. Initial disproved warning predictions remain.
- [Native and reference closure](https://github.com/vishvananda/rustgm/actions/runs/40177790343)
  passes fifteen actual rustc/CPPGM comparisons and regenerates only thirteen
  changed metadata contracts. Their 39 output files remain byte-identical.
  These programs have empty entry points; this establishes build/link acceptance,
  while the frontend controls establish the feature and warning behavior.
- [Combined integration](https://github.com/vishvananda/rustgm/actions/runs/40179084740)
  passes nineteen focused private controls, all 212 harness tests, upstream and
  audit-cache checks. Its additional primitive work remains a separate checkpoint.

Existing fixture bytes, unselected oracles and full reference receipts remain
unchanged. Scoped receipts preserve generation history. PA18 now explicitly
declares its supplemental pattern inspection mode; all native cases have declared
outcomes. There are 8,478 active text cases across the existing 24 draft stages.

The fast gate now checks shared assignment metadata before compiler work. It
validates all declared modes, native/layout outcomes and feature placement without
requiring newly generated references. Ignored per-case `cfg` fields fail with
guidance to use `args: ["--cfg", ...]`. Full reference provenance validation remains
in the harness. The metadata change passed twelve focused Python controls and
all active case metadata in a 13.55-second guarded scope.

Measured cold receipt records are Removals 80, Index 104, Path 88, Work 16 and
Session 32 bytes. The overwrite journal measures Saved 44, Original 152 and Unit
672 bytes. Ordinary Body 96 and Definition 136 layouts remain unchanged. These
are focused measurements, not a new whole-program performance claim.

Original core/alloc/std artifacts, imported macro and attribute contracts,
provided-body transport, nonempty source formatting and further source-language
requirements remain on the library blocker tracker. No original crate artifact
or additional assignment completion is claimed.
