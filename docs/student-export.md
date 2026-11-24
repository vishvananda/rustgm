# RustGM student export

Preview the assignments from a committed RustGM revision. Choose the last
assignment in the prefix being prepared for release.

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
members. The compiler implementation, unpublished fixture files, maintainer
build outputs and local backend configuration stay in the maintainer repo.
`export.json` records the source commit, backend lock digest and case counts.

Follow CPPGM's two-part release process for `rustgm-assignments`. Qualify the
chosen maintainer revision with the assignment release checks and run the
exported suites against the same immutable reference executable. Use the
export's backend installer and run its library example as part of that check.
Review links and commands from a fresh export. When a PA has a `STUDENT.md`,
the exporter uses it as that PA's `README.md`. See the
[handout review tracker](student-handout-review.md).

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
