import os,subprocess,sys
selectors=['crate_linking::records::standalone_records_export_and_unsupported_semantic_contracts_stay_explicit', 'crate_linking::non_exhaustive::local_non_exhaustive_contracts_preserve_construction_and_complete_matches', 'crate_linking::non_exhaustive::non_exhaustive_owners_forms_duplicates_and_unknown_attributes_stay_checked', 'crate_linking::non_exhaustive::non_exhaustive_contracts_reject_a_different_current_source_image']
subprocess.run([sys.executable,"scripts/fastcheck.py","--seconds","190",*selectors],env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True)
