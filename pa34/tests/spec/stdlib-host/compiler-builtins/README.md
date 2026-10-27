# Original compiler-builtins host pair

The contract and text template were frozen before running the original pinned
build script. `{PACKAGE_ROOT}` substitutes only the explicitly authenticated
package's absolute path in its `compiler-rt` metadata line. Preserve all141
lines and their order, including the upstream duplicate check-cfg entry.

Use the original std graph's paired host units3 and4, actual selected features,
unwind host profile and separately frozen abort target cfg. The host compiler is
the pinned rustc; this host-only boundary does not implement target Rust code.
The shared `library/compiler-builtins/libm/configure.rs` is authenticated against
the original source inventory and retained in the existing Context input pins,
which are checked before and after execution. No source copy or rewrite is used.

Expected enabled cfgs are `intrinsics_enabled` and `mem_unaligned`. No source is
generated and no compiler query is made. The environment omits the two declared
optional overrides. This observation is specific to the frozen configuration;
complete rerun/environment/external-tool closure and target output consumption
remain required. See [crate planning](../../../../../docs/crate-planning.md).
