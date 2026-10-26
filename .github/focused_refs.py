"""Pinned independent typed-alias observations; no candidate compiler execution."""
from pathlib import Path
import hashlib,json,os,subprocess,sys,time
ROOT=Path.cwd();OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
if '--worker' not in sys.argv:
 raise SystemExit(subprocess.run(['timeout','--signal=TERM','--kill-after=3s','80s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=False).returncode)
OUT.mkdir(parents=True,exist_ok=False);started=time.monotonic();plan_path=ROOT/'pa31/tests/spec/trait-aliases/semantic-v1/contract.json';plan_bytes=plan_path.read_bytes();plan=json.loads(plan_bytes)
rustc=Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True,timeout=5).strip());digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert digest(rustc)==plan['rustc_sha256'];results=[];differences=[];infra=[]
def command(name,phase,cmd,source,expected_status,expected_stdout=None):
 start=time.monotonic();r=subprocess.run(list(map(str,cmd)),cwd=ROOT,capture_output=True,env={**os.environ,'RUSTC_BOOTSTRAP':'1'},timeout=5,check=False)
 (OUT/(name+'.'+phase+'.stdout')).write_bytes(r.stdout);(OUT/(name+'.'+phase+'.stderr')).write_bytes(r.stderr)
 row={'name':name,'phase':phase,'command':list(map(str,cmd)),'source_sha256':digest(source),'status':r.returncode,'predicted_status':expected_status,'seconds':time.monotonic()-start,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest()}
 if phase!='execute':
  diagnostics=[json.loads(line) for line in r.stderr.decode().splitlines() if line.startswith('{')];row['warning_codes']=[d.get('code',{}).get('code') if d.get('code') else None for d in diagnostics if d.get('level')=='warning'];row['error_codes']=[d.get('code',{}).get('code') if d.get('code') else None for d in diagnostics if d.get('level')=='error']
 if r.returncode not in ((0,1) if phase!='execute' else (0,)):infra.append(row)
 if r.returncode!=expected_status or expected_stdout is not None and r.stdout!=expected_stdout.encode():differences.append(row)
 results.append(row);return r.returncode
for case in plan['cases']:
 source=ROOT/case['path'];assert digest(source)==case['sha256']
 for edition in plan['editions']:
  name=case['name']+'-'+edition;output=OUT/(name+'.rmeta');common=[rustc,'--edition='+edition,'--target=x86_64-unknown-linux-gnu','-Cpanic=abort','--error-format=json','--crate-name=trait_alias_contract',source]
  command(name,'metadata',[*common,'--emit=metadata','-o',output],source,case['predicted_status']);output.unlink(missing_ok=True)
  if case['native']:
   binary=OUT/name
   status=command(name,'build',[*common,'-o',binary],source,0)
   if status==0:command(name,'execute',[binary],source,0,case['native_stdout'])
   binary.unlink(missing_ok=True)
 assert digest(source)==case['sha256']
assert plan_path.read_bytes()==plan_bytes
binding={'state':'observed' if not infra else 'infrastructure-failed','ci_commit':os.environ.get('GITHUB_SHA'),'plan_sha256':hashlib.sha256(plan_bytes).hexdigest(),'rustc_sha256':digest(rustc),'cases':len(plan['cases']),'commands':len(results),'metadata':sum(r['phase']=='metadata' for r in results),'native_builds':sum(r['phase']=='build' for r in results),'native_executions':sum(r['phase']=='execute' for r in results),'infrastructure_failures':len(infra),'prediction_disagreements':differences,'results':results,'seconds':time.monotonic()-started,'candidate_execution':False,'source_edits':False,'retained_binaries':False}
(OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');print(json.dumps({k:v for k,v in binding.items() if k not in ('results','prediction_disagreements')},indent=2));print('Prediction differences',[(r['name'],r['phase'],r['status'],r['error_codes'] if 'error_codes' in r else []) for r in differences]);assert not infra
