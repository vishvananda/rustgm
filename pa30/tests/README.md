# PA30 fixture ownership

The released `c-abi` component publishes `units/c-abi.json` and the reference
sidecars of its 143 cases; see the
[release record](../../docs/pa30-c-abi-release.md) for its deferred list. Every
other family here is unpublished.

A program that imports C symbols names its host providers in
`native.providers`. The native check compiles each one with
`cc -std=c11 -O0 -c` and links the same objects into the rustc executable and
into the candidate's, whose LowIR is emitted as an object for that link. The
text check does not use them.

Whole PA30 is planned: no graded whole-assignment suite yet.

Required families: repr(C); aggregate ABI; callbacks; exported globals; C interoperability.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in `manifest.json`; no undiscovered fixtures.
