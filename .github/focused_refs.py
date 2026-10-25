import os,subprocess,sys
selectors=['lower::transported_source::publication_tests::plain_and_configured_absent_headers_publish_with_the_existing_profile', 'lower::transported_source::publication_tests::all_selected_function_and_trait_contracts_reject_before_payload_publication', 'lower::transported_source::publication_tests::staged_identity_and_existing_query_failures_are_preserved']
command=[sys.executable,"scripts/fastcheck.py","--seconds","200"]
for name in selectors:command += ["--lib-case",name]
subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True)
