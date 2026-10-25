"""CI-only exact qualification of one frozen combined candidate target."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
STARTED=time.monotonic()
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
EXPECTED_SHA='1a3219ac59d7509dd23111b86f32d46235aee33853d53e9edbb1e5ba07d9d13c'
EXPECTED_COUNT=2584
ARGS=['generic_functions::lifetime_binding_does_not_renumber_rigid_parameter_declarations', 'generic_functions::function_lifetime_formals_keep_dense_arguments_through_actual_calls', 'generic_aliases::source_aliases_substitute_types_lifetimes_consts_and_defaults']
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
