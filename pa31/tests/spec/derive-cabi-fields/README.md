# Typed C ABI fields in source derives

The independently declared source controls copy `Option<extern "C" fn()>` through the existing source derive field producer. Its concrete Clone predicate selects the typed copier without adding unrelated marker or const-trait prerequisites. Original libc's paired Clone/Copy derive remains the actual std-graph consumer.

The copier retains the explicit ABI literal node and imports only its C value from the actual source owner. Raw and escaped spellings preserve their source spans. Omitted `extern`, safety and existing parameter/result structure retain their own syntax; HRTB, other ABI and variadic fields keep separate profile fences. Generic String constants remain outside this selected copy contract.

`outcomes.json` lists and hashes every source and the case table before execution. The source controls inspect expansion/resolution only. Ordinary executable behavior remains frozen; no external library artifact, native/codegen qualification or whole assignment completion is claimed. Private controls place distinct prefixes in source and destination decoded pools and use actual typed splice/append APIs to detect cross-arena indices.

All twelve RustGM outcomes pass in fastcheck `1794903303801942546`;
all twelve pinned rustc outcomes are independently checked. Six selected private
C ABI/storage/image controls pass, as do the existing field and callable-span
controls. The ordinary control preserves acceptance and byte-identical typed IR
from the previous compiler; the originally incorrect rejection prediction and
diagnostic grammar prediction are retained in `outcomes.json`. Actual unchanged
libc/std replay remains required.
