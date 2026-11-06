# Diagnostic comparison default-body control

This authored provider uses the pinned complete PartialEq declaration/default
body from source-location, removing only its lang=eq tag. The diagnostic identity
and real eq/ne methods remain ordinary trait declarations; no operator identity
is granted. Its no_core foundation supplies real Freeze as required by pinned
Rust1.98.0. The first independent attempt without Freeze is retained as a failed
foundation setup, and the corrected frozen complete program independently passes.
See predictions.json and the recorded raw reference receipt. This regression
repairs the stale private negative in full CI42214520487 without changing any
course text oracle or weakening missing-owner/invalid-target rejection.
