# Initializer lookup/completion follow-up

This bounded study compares immutable release binaries: A is the original core
checkpoint `4ebba641` (SHA-256
`89712e19635f3e55302e9e2467b1d7976637725dd0a1fa0b4d36d7e9c4ba2f46`),
B is index/policy checkpoint `cf06d8fd` (SHA-256
`3ed624e44f110fc104654d55dcf31d1065c09058040b431546ba1ae2ef3507ef`).
The comparison includes intervening accounting/readiness and integrated changes;
it does not isolate only the final initializer policy.

Four frozen inputs reuse the earlier study's ordinary 512-function control and
16/64/256 required-root family. Independent pinned Rust object qualification
passes, with value assertions for repeated-root inputs. Every A/B output hash
agrees before timing and for all observations. Input and binary hashes are checked
before/after measurement. The exclusive run retained 344 process observations:
warmups, four A/A blocks and five ABBA blocks, three fresh processes per position.
Observed process time totals 3.74 seconds; the resource receipt is
`e07e3308cf1749159148d4d005c2175a`, exit zero.

| Input | Median paired B/A | A/A ratio range | Median RSS KiB A → B |
| --- | ---: | ---: | ---: |
| ordinary 512 | 0.9912 | 0.9466–1.0755 | 11604 → 11868 |
| roots 16 | 1.0226 | 0.7323–0.9526 | 9328 → 9234 |
| roots 64 | 1.0629 | 1.0077–1.1338 | 10056 → 9914 |
| roots 256 | 0.9934 | 0.8586–1.1481 | 10726 → 10586 |

Calibration variation is large relative to these differences; this run supports
no small wall-time or RSS improvement claim. The ordinary control's observed
264 KiB RSS increase remains an observation requiring care. The earlier study's
594 KiB scalar-control RSS increase also remains unresolved; that scalar control
was not rerun here. Every sample is retained.

Actual B work for 16/64/256 roots is respectively: one preparation visit per
input, N index publications, 2*N shared-index lookups, N completion visits,
N validations, N expression executions, two function CFGs, two peak frames and
eight peak value/cell slots. Interpreter steps are 370/1474/5890; charged code
payload is 8304/31536/124464 bytes. Index payload is 196/388/1156 bytes, exactly
four per definition. Expression counts are unchanged after runtime lowering,
demonstrating reuse of eager results. The ordinary control has zero constant
preparation, publications, lookups, validation/visits, expressions and index
bytes. These are logical work/storage counters, not heap measurements.

Evidence is under `obj/pa25-index-performance/`: pinned executables, exact
commands in `pin.py`/`measure.py`, qualification diagnostics, helper provenance,
`work.json`, `observations.jsonl` and `summary.json`. SHA-256 digests:

- inputs: `1c4c27ea5467ea4a50f5c55139fed2699d2df5c3f930842bad017e0f67270eb3`
- work: `5d4f59438ea2846b785463d02e8acf02899c0514a1af9051e03c9b1ddfd46858`
- observations: `8040c357d10a3ee3d935e8366fb39e4088b0ec911a27b72eca5afe335e4fdbde`
- summary: `7a52982ded7760232446f89ad9b9864cbf6a5523899f241cf2afb6c67c14615c`

PA25's remaining implementation checklist is retained in
[the initializer checkpoint](pa25-initializer-validation.md#remaining-pa25-work).
