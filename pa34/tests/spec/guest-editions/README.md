# Guest crate editions

`outcomes.json` declares the source outcomes and every input before either
compiler runs. rustc compiles the host as `core` and each guest as a separate
crate, with the listed edition and the host and earlier guests as dependencies.
The candidate resolves the same crates in its explicit bring-up session.

The mixed positive checks distinct edition preludes and a macro whose `gen`
identifier keeps its 2021 definition edition in a 2024 caller. Root, file-module
and included extern blocks distinguish 2021 from 2024. The cache opponents use
distinct root-file copies and share their module/include source, so a physical
file parsed for 2021 cannot grant a 2024 crate acceptance.

PA34 remains planned; these are development source controls, not release oracles.
