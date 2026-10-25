"""CI-only exact qualification of one frozen combined candidate target."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
STARTED=time.monotonic()
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
EXPECTED_SHA='9eb9e5f8f64ec5a1f4ef262b905dbbe12467089546b1ba4bf9032c8264e77d6d'
EXPECTED_COUNT=2585
ARGS=['--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_artifacts_reach_fresh_closed_pointer_instances', '--lib-case', 'sema::crates::authored_export::canonical_nodes::tests::anonymous_function_fresh_callers_reject_safety_arity_and_result_mismatches', '--lib-case', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof']
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
