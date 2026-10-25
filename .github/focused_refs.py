import hashlib,json,os,subprocess,sys
from pathlib import Path
source = '2b3dff021a6e25d03bd4acf27bb3823fb61955cd'
selectors = ['lower::transported_source::publication_tests::plain_and_configured_absent_headers_publish_with_the_existing_profile', 'sema::header_attributes::owner_compatibility_tests::real_local_crate_owner_with_imports_is_unsupported_owner_not_unknown_header', 'sema::header_attributes::owner_compatibility_tests::forged_crate_root_kind_scope_and_package_origin_do_not_waive_contract_queries', 'lower::transported_source::publication_tests::old_functions_metadata_keeps_imported_attribute_state_unknown', 'sema::header_attributes::tests::marked_and_plain_providers_roundtrip_into_fresh_used_and_discarded_consumers']
folder=Path("obj/parallel/all-attribute-selected-output-v1")
folder.mkdir(parents=True,exist_ok=True)
cli=Path("obj/rust/dev-fast/rustgm")
(folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest(),"selectors":selectors},indent=2)+"\n")
command=[sys.executable,"scripts/fastcheck.py","--seconds","200"]
for name in selectors: command += ["--lib-case",name]
subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True)
