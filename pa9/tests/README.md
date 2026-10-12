# PA9 fixture ownership

`manifest.json` declares full-program outcomes for the type-checking source
profile. Numbered cases in `inference/`, `control/`, `well-formed/` and `elision/`
correspond to independently maintained rustc acceptance matrices. Descriptive
alias, occurs-check and package fixtures extend their integration coverage.

`boundary/` cases deliberately defer initialization/write permissions, region
proofs and constant-call validity. Each declares its difference from rustc and
names the later owning phase; these are not whole-program acceptance claims.

Reference stdout/stderr/status sidecars are generated only by `make ref-test-pa9`
after declared outcomes and independent checks agree. All fixtures are authored
RustGM tests; imported upstream fixtures remain with their earlier owners.
