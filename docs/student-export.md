# RustGM student export

The initial release for `rustgm-assignments` includes PA1–PA37, ending
with self-hosting at PA37. PA38–PA45 are the follow-on track (procedural
macros, unwinding, remaining edition/target/async closure, and final
frontend acceptance). Maintainer and student PA numbers match directly,
with no renumbering during export. The [course order](../student/README.md#course-order)
explains which parts of crate building and target support precede `core`.

All 37 assignments on the self-hosting track (PA1–PA37) now have active
grading suites: PA1–PA31 and PA34 use text/native oracle manifests (with PA34
also running its host `build.rs` driver suite), PA32/PA33/PA35/PA36 run real
upstream library and crate consumer suites via `scripts/grade_consumers.py`,
and PA37 runs the reproducible self-hosting gate (`stage0` → `stage1` →
PA1–PA24 ladder → `stage2` → byte-for-byte compare + negative controls) via
`scripts/pa37_self_host.py`.

Export the complete PA1–PA37 student repository from a committed revision:

```sh
python3 scripts/export_assignments.py --self-hosting obj/rustgm-assignments
```

`--self-hosting` (equivalent to `--through pa37`) requires every assignment on
the track to have a grading suite and produces `"handouts_only": []` in
`export.json`. `--include-pending-handouts` remains available only for
previewing planned stages beyond PA37 (PA38–PA45). Choose `--through paN` to
export a prefix.

```sh
python3 scripts/export_assignments.py --revision HEAD --self-hosting obj/student-preview
make -C obj/student-preview
make -C obj/student-preview setup-backend backend-example
make -C obj/student-preview setup-rust-src
make -C obj/student-preview/pa1 test RUSTGM_TEST_APP=/absolute/path/to/rustgm
```

The exporter reads the selected commit through Git and creates a new directory.
It includes the handouts and manifest-declared fixtures, supporting files and
text oracles, with original bytes, source symlinks and upstream attribution.
It includes a Rust starter, the course test runner and the optional C LowIR
example. The backend lock and distribution manifest come from the same commit
as the fixtures. The shared installer verifies those files' pinned package and
members. The compiler implementation, test ownership notes, unpublished
fixtures, build outputs, and local backend configuration stay in the
maintainer repo.
`export.json` records the source commit, backend lock digest and case counts.

Follow CPPGM's two-part release process for `rustgm-assignments`. Qualify the
chosen maintainer revision with the assignment release checks and run the
exported suites against the same immutable reference executable. Use the
export's backend installer and run its library example as part of that check.
Review links and commands from a fresh export. Every assignment's
`README.md` is its student handout. `MAINTAINER.md` and fixture ownership notes
stay in this repository. See the [handout review tracker](student-handout-review.md).

Publish the qualified Linux x86-64 reference executable as a separate release
asset. Pin its URL, archive and executable digests, source commit,
target/profile, CPPGM commit and backend lock digest in a `reference.lock.json`
shipped with the student tree. A student download command can check the source
commit and backend lock digest against `export.json`, verify the bundle and
binary before installation, and pass the executable to the test runner through
`RUSTGM_TEST_APP`. Publish the student tree only after the asset is available
and its downloaded executable passes the exported suites. Restrict publishing
and repository credentials to the trusted release job for a qualified main
revision. The local exporter creates the preview and receipt. The download
command and publishing workflow need to be added when the student repository
and reference release destination are set up.
