import os,subprocess,sys
selectors=['literal::tests::wide_decimals_match_frozen_ieee_boundaries_without_double_rounding', 'literal::tests::numeric_shape_preserves_known_width_and_rejects_real_invalid_suffixes', 'literal::wide::tests::significant_prefix_scratch_stops_growing_for_long_zero_and_sticky_tails', 'resolve::wide_floats::tests::active_wide_features_and_inactive_token_opponents_use_real_authority', 'resolve::wide_floats::tests::wide_values_keep_exact_shapes_and_unsupported_later_operations_closed', 'types::wide_float_tests::two_new_primitives_preserve_existing_ids_and_enclosing_hot_layouts']
command=[sys.executable,"scripts/fastcheck.py","--seconds","200"]
for name in selectors:command += ["--lib-case",name]
subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True)
