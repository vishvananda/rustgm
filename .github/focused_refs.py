"""CI-only exact qualification of one frozen combined candidate target."""
import hashlib,json,os,shutil,signal,stat,subprocess,sys,time
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
# The preceding unchanged workflow step completed the actual Cargo CLI build.
# Retain that exact executable before the selected harness can touch the cache.
prebuilt=ROOT/'obj/rust/dev-fast/rustgm'
if prebuilt.is_symlink() or not prebuilt.is_file():
    raise ValueError('mandatory prebuilt CLI is absent or indirect')
info=prebuilt.stat()
if not info.st_mode & stat.S_IXUSR or prebuilt.open('rb').read(4)!=b'\x7fELF':
    raise ValueError('mandatory prebuilt CLI is not an executable ELF')
output=ROOT/'obj/parallel/all-attribute-selected-output-v1/cli/candidate'
output.mkdir(parents=True,exist_ok=False)
(output/'.keep-artifacts').write_text('Pinned combined8191 prebuilt CLI for root scoped review.\n')
sha=hashlib.sha256(prebuilt.read_bytes()).hexdigest()
retained=output/'rustgm'
shutil.copy2(prebuilt,retained)
if hashlib.sha256(retained.read_bytes()).hexdigest()!=sha:
    raise ValueError('retained CLI copy differs from mandatory prebuilt binary')
fixture=ROOT/'pa18/tests/spec/functions/accept-lifetime-argument-indices.rs'
workflow=subprocess.check_output(['git','show','HEAD:.github/workflows/check.yml'],cwd=ROOT,text=True)
binding={'candidate':'8191e6186147505915b5f47e43a75a2e77a8bc52',
         'ci_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
         'run_id':os.environ.get('GITHUB_RUN_ID'),'sources':fingerprint(ROOT),
         'canonical_source_sha256':EXPECTED_SHA,'source_count':EXPECTED_COUNT,
         'build':{'attribution':'Preceding successful mandatory workflow CLI step; no helper rebuild.',
                  'workflow_blob':'0f93cdd0a41e04e93ac194967b72196e300480a9',
                  'workflow_command':next(line.strip()[5:] for line in workflow.splitlines() if '--bin rustgm' in line),
                  'path':str(prebuilt),'mtime_ns':info.st_mtime_ns},
         'binary':{'path':'cli/candidate/rustgm','sha256':sha,'bytes':info.st_size,
                   'mode':stat.S_IMODE(info.st_mode),'hydrate':'Verify SHA-256 then restore this executable mode.'},
         'pa18_fixture':{'path':str(fixture.relative_to(ROOT)),
                         'sha256':hashlib.sha256(fixture.read_bytes()).hexdigest()}}
(output/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
print(f'Retained mandatory prebuilt CLI sha256={sha} bytes={info.st_size} mode={stat.S_IMODE(info.st_mode):o}',flush=True)
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
