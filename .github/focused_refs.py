import hashlib,json,os,pathlib,shutil,subprocess,sys
root=pathlib.Path.cwd();sys.path.insert(0,str(root/'scripts'))
from fastcheck_run import fingerprint
EXPECTED='fdf200a9d90ccec3bfdd106ce3671947136daa43358dd06bc4f472446e7649fa'
CANDIDATE='58fa8509fceadba9cd7bc40788fed8f9a644f433'
if '--worker' not in sys.argv:
 raise SystemExit(subprocess.run(['timeout','--signal=TERM','--kill-after=3s','190s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=False).returncode)
app=root/'obj/rust/dev-fast/rustgm';out=root/'obj/parallel/all-attribute-selected-output-v1';out.mkdir(parents=True,exist_ok=False)
copy=out/'candidate/rustgm';copy.parent.mkdir();shutil.copy2(app,copy);copy.chmod(0o755);(copy.parent/'.keep-artifacts').touch()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if sha(copy)!=EXPECTED:raise ValueError('producer CLI differs from caller-declared qualified bytes')
before=fingerprint(root)
record={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':EXPECTED,'compiler_source_sha256':{str(p):sha(p) for p in sorted(pathlib.Path('dev').rglob('*.rs'))},'profile':'dev-fast','rustc':subprocess.check_output(['rustc','--version']).decode().strip(),'sources':before,'functional_gate':'40214141112; must pass exact4 before this observer launches'}
(out/'binding.json').write_text(json.dumps(record,indent=2)+'\n')
status=subprocess.run([sys.executable,'scripts/observe_original_core.py','--worker','--expected-cli-sha256',EXPECTED,'--cli',str(copy),'--producer-binding',str(out/'binding.json'),'--out',str(out/'whole-core-lib026-v1')],check=False).returncode
if fingerprint(root)!=before:raise ValueError('compiler sources changed during observation')
raise SystemExit(status)
