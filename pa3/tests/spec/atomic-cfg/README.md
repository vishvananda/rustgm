# Atomic target fact versus authorization

PA3 computes the true 8-bit and false 128-bit load/store facts. It retains the
attributes as syntax and does not grant compiler feature permission. The same
inputs are rejected by pinned Rust and the paired PA8 controls because the
actual `cfg_target_has_atomic` feature was not requested. This deliberate
phase distinction is recorded in both manifests.
