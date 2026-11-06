The CStr typed-literal kernel rejected existing byte-string patterns whose actual
type is `&[u8]`. Full CI 42265080175 exposed five pattern regressions; its sixth
failure expected the obsolete Bootstrap C-string diagnostic. PA13 already requires
both byte-array and byte-slice pattern views.

The repair recognizes an immutable reference to an actual `U8` slice in the
existing `Bytes` path. Other nominal values still require the actual CStr
role/storage profile. No records, caches, allocations or permissions were added.
The paired Rust/candidate controls now include array, slice and string constants
sharing the same decoded bytes. The Bootstrap C-string control requires its
actual source declaration; real source CStr storage remains separately tested.

The guarded main-only batch passed formatting, architecture audit and selected
strict Clippy; all 10 text-pattern tests, one literal bounds/shared-write test,
six CStr source tests and the existing four native observers passed. Exact argv,
timings, source hashes and the retained prior failure are bound by
`text-slice-repair.json`. Full private/release validation remains CI work.

Reborrow consumer commit `b6adbf1e8` remains on its preserved branch and is absent
from this repair, which starts directly at published CStr main `afcc7d4a8`.
