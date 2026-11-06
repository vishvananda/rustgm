# Bootstrap receiver declaration CI repair

Full42199401711 on3116d8b41 passes all2,753 private tests and all2,185 regular
public integration tests. Only two PA19 diagnostic fixtures fail: wrong const
and type arguments in an explicit inherent receiver report a missing source
catalog instead of their actual Self mismatch. No reference is regenerated.

Actual source Target-chain validation now runs only when its provider catalog
exists. Bootstrap declarations retain the existing rigid generic/lifetime receiver
validation directly. Valid source smart receivers still require their genuine
Deref/LegacyReceiver contracts, and their lifetime debts remain unchanged. This
also avoids allocating source-chain scratch for bootstrap receiver mismatches.

The focused gate1793961244062850330 passes nine exact public tests across two
targets and all1,193 PA19 text cases in109.89s, with strict Clippy, formatting and
architecture checks. It includes actual arbitrary-receiver and Coroutine source
controls. Full native/debug/independent CI still needs a successor; the previous
full run stopped at these two diagnostic mismatches.
