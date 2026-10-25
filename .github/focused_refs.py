"""CI-only exact qualification of one frozen combined candidate target."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
STARTED=time.monotonic()
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
EXPECTED_SHA='32fbe3c49c062295be46c363055b0a25ecaaeb3463cc0acba963b8345c3a5774'
EXPECTED_COUNT=2594
ARGS=['--lib-case', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_retain_full_family_and_unused_formals', '--lib-case', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_reject_changed_provenance_and_incomplete_formals', '--lib-case', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_bound_selected_work_and_warm_storage', '--lib-case', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_keep_failed_attempt_and_peak_costs', '--lib-case', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof', '--lib-case', 'resolve::body_origin::catalog::freezing_tests::clone_drop_and_unique_access_never_reopen_frozen_installation']
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
