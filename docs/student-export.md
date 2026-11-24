# RustGM student export

The initial release for `rustgm-assignments` includes PA1–PA37, ending
with self-hosting. The exporter assigns student PA37 to maintainer PA45.
Maintainer PA37–PA44 appear as student PA38–PA45 in the follow-on track.
Existing fixture ownership and qualification records use the maintainer
numbers. The [course order](../student/README.md#course-order) explains
which parts of crate building and target support precede `core`.

PA32–PA36 require separate sysroot artifacts and fresh consumers. Qualify
the required source diagnostics and metadata before releasing those
assignments. Student PA37 then rebuilds the compiler and its dependencies
in a single whole-graph compilation session per stage.

Build a preview from a committed revision to review all of the handouts
on the self-hosting track.

```sh
python3 scripts/export_assignments.py --self-hosting --include-pending-handouts obj/self-hosting-preview
```

Use `--self-hosting` alone for a release candidate. The exporter requires
a grading manifest for each assignment in that track. The explicit preview
option includes the README for a planned assignment and records it in
`export.json` under `handouts_only`. Choose `--through` with a student
assignment number to produce a prefix. `--through pa37` selects the same
track as `--self-hosting`.

```sh
python3 scripts/export_assignments.py --revision HEAD --through pa11 obj/student-preview
make -C obj/student-preview
make -C obj/student-preview setup-backend backend-example
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
