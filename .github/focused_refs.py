"""CI-only exact qualification of one frozen combined candidate target."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
STARTED=time.monotonic()
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
EXPECTED_SHA='1a3219ac59d7509dd23111b86f32d46235aee33853d53e9edbb1e5ba07d9d13c'
EXPECTED_COUNT=2584
ARGS=['--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_headers_cross_codec_without_declaration_surrogates', '--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_context_and_binder_opponents_fail_before_publication', '--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_shared_roots_use_one_persistent_dag', '--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_work_and_storage_scale_with_actual_shallow_dag', '--lib-case', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof', '--lib-case', 'lower::instances::staged_originals::tests::ordinary_collector_failure_discards_partial_rows_before_finalization', '--lib-case', 'literal::tests::numeric_shape_preserves_known_width_and_rejects_real_invalid_suffixes', '--lib-case', 'types::wide_float_tests::two_new_primitives_preserve_existing_ids_and_enclosing_hot_layouts', '--lib-case', 'lower::transported_source::publication_tests::plain_and_configured_absent_headers_publish_with_the_existing_profile', '--lib-case', 'lower::transported_source::publication_tests::staged_identity_and_existing_query_failures_are_preserved', '--lib-case', 'support::crates::generics::source::scopes::tests::malformed_slots_reject_foreign_self_kind_owner_gap_and_missing_scope', '--lib-case', 'sema::header_attributes::tests::marked_and_plain_providers_roundtrip_into_fresh_used_and_discarded_consumers', '--lib-case', 'sema::header_attributes::tests::origins::inline_and_trait_contracts_retain_complete_actual_owner_applicability']
def authenticate():
    sources=fingerprint(ROOT)
    digest=hashlib.sha256(json.dumps(sources,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    if len(sources)!=EXPECTED_COUNT or digest!=EXPECTED_SHA:
        raise ValueError('combined candidate full source fingerprint changed')
authenticate()
command=[sys.executable,'scripts/fastcheck.py','--seconds','200',*ARGS]
process=subprocess.Popen(command,cwd=ROOT,env={**os.environ,'RUST_TEST_NOCAPTURE':'1'},start_new_session=True)
try:
    status=process.wait(timeout=max(0.01,STARTED+199-time.monotonic()))
except subprocess.TimeoutExpired:
    os.killpg(process.pid,signal.SIGTERM)
    try:process.wait(timeout=max(0.01,STARTED+199.5-time.monotonic()))
    except subprocess.TimeoutExpired:
        os.killpg(process.pid,signal.SIGKILL)
        process.wait(timeout=max(0.01,STARTED+200-time.monotonic()))
    status=124
authenticate()
print(f'CI helper total: {time.monotonic()-STARTED:.2f}s',flush=True)
sys.exit(status)
