# Original core after local primitive ownership

The [unchanged-source observer](https://github.com/vishvananda/rustgm/actions/runs/40183027468)
uses compiler SHA-256
`50dac790f4a1f83dd015a552d02a745b6174a24a3cff5413993642a272dd6098`,
the same compiler as the qualified primitive ownership/native checkpoint.
It advances past the previous `autodiff_forward` declaration to
`offload_kernel`, `library/core/src/macros/mod.rs:1668`, byte 62644:

```text
ERROR: file 3: 62644: source builtin macro expansion is not implemented
```

The compiler exits 1 with no panic or infrastructure failure. Its four opened
original files are core's root, `prelude/mod.rs`, `prelude/v1.rs` and
`macros/mod.rs`. No core artifact or completed assignment is claimed.

`scripts/observe_original_core.py` fetches the official archive at revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. It verifies the archive's 39,212,056
bytes and SHA-256
`5a9993673ce40804e6b50300df59ecb3d13942c1c89cd11e76acf705b331f982`,
then all 722 selected original files: 309 core, 306 stdarch and 107
portable-simd files. The latter two directories are part of this superproject
revision, rather than unpopulated submodules. These checks use the retained
source-acquisition inventory; no upstream source is patched.

The command retains `--emit-types --edition=2024 --crate-name core
--crate-disambiguator core_1_98_88d9 --source-core core@core_1_98_88d9` and
the existing x86-64 Linux/panic-abort defaults, with no custom cfg overrides.
Only the filesystem root changes for hosted CI. Compiler identity, configuration
source hashes, actual open trace, opened-file hashes and before/after input
hashes are bound in the uploaded receipts. Unknown source dependencies and
missing tracing tools fail as infrastructure errors.

The complete observation took 5.729 seconds including download, extraction,
hashing and tracing. This is a development frontier check, not a compiler
performance measurement. The existing unoptimized CLI build has a 100-second
limit; the guarded observer has a 190-second outer limit. It does not require
a release build or a full test inventory. Future compiler checkpoints must
explicitly update the qualified CLI/configuration binding before reuse.

The next issue is LIB-022: authenticate and classify the actual builtin
declaration family independently of execution availability. Unused declarations
must retain their real kinds and source identity; unsupported invocations must
remain explicit failures. PA31 owns source binding, with PA8 expansion and the
owning feature assignments retaining execution responsibility.
