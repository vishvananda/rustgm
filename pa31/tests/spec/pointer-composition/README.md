# Raw pointer composition controls

PA28 owns the intrinsic/validity contract; PA31 owns the genuine source binding.
`predictions.json` inventories each input and freezes outcomes before observation.
The oracle calls actual pinned core helpers. Candidate source controls must keep
those helper and UB-check bodies, rather than supply compiler library factories.
Preserve NOTICE and the Rust88d9 source pin. No invalid-access native execution is
permitted; only the valid runtime companion executes. Dynamic metadata and wire
transport remain explicit pending boundaries.
