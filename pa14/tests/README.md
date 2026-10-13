# PA14 fixture ownership

All 124 cases are declared in `manifest.json`, with exact stdout, stderr and
status oracles. The default mode is `--emit-ownership`; twelve cases explicitly
select supplemental `--emit-lowir` and require native execution evidence.

The ownership views contain 58 acceptances and 54 rejections. Clusters cover
moves, initialization, arrays, struct updates, Copy/Clone obligations and pattern
interactions. The twelve executable controls check full values across branches,
loops, storage exit, partial moves/reinitialization and delayed guard capture.
All cases agree with pinned rustc without an acceptance or runtime exception.

These are authored maintainer controls. Preserve fixture bytes and register
new inputs in the earliest owning assignment. Only `make ref-test-pa14` may
regenerate sidecars after checking every declared outcome; do not hand-edit
oracles or bless a failure. See the owning handout and validation record.
