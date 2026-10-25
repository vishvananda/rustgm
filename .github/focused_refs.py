"""Hosted pinned-rustc metadata/object observations; no RustGM feature execution."""
import hashlib,json,os,re,signal,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
INPUT=ROOT/'.github/lib026-namespace-oracle'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
PLAN_SHA='5ab7fbce3c906e63e5bbd9355298d99caa2113bdafd5e8d09334a9fa3370778e'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def worker(deadline):
 plan_path=INPUT/'plan.json'
 if sha(plan_path)!=PLAN_SHA:raise ValueError('frozen plan changed')
 plan=json.loads(plan_path.read_bytes())
 if sha(INPUT/'prior-results.json')!=plan['prior_results_sha256']:raise ValueError('prior provider proof changed')
 sources={p:sha(INPUT/'inputs'/p) for p in plan['inputs']}
 if sources!=plan['inputs']:raise ValueError('frozen source mismatch')
 rustc=Path(subprocess.check_output(['rustup','which','rustc','--toolchain','1.98.0'],text=True,timeout=10).strip())
 if sha(rustc)!=plan['rustc_sha256']:raise ValueError('pinned rustc bytes mismatch')
 OUT.mkdir(parents=True,exist_ok=True);records=[];providers={};infra=[];disagreements=[]
 (OUT/'plan.json').write_bytes(plan_path.read_bytes())
 for number,row in enumerate(plan['observations']):
  folder=OUT/'raw'/str(number);folder.mkdir(parents=True)
  source=INPUT/'inputs'/row['source'];target=folder/('libdomain_namespace.rmeta' if row['kind']=='provider' else ('result.rmeta' if row['emit']=='metadata' else 'result.o'))
  command=[str(rustc),str(source),'--edition=2024','--target=x86_64-unknown-linux-gnu','--crate-type=lib','--crate-name',row['crate_name'],'--emit='+row['emit'],'-Cpanic=abort','-Cmetadata='+row['id'],'-o',str(target)]
  if row['kind']=='client':command+=['--extern','base='+str(providers[row['provider']])]
  remaining=deadline-time.monotonic()
  if remaining<=0:raise TimeoutError('oracle worker deadline')
  started=time.monotonic()
  try:
   result=subprocess.run(command,env={**os.environ,'RUSTC_BOOTSTRAP':'1'},stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=min(15,remaining),check=False)
   status=result.returncode;stdout=result.stdout;stderr=result.stderr
  except subprocess.TimeoutExpired:
   status=124;stdout=b'';stderr=b'worker timeout'
  (folder/'stdout').write_bytes(stdout);(folder/'stderr').write_bytes(stderr)
  text=stderr.decode(errors='replace');messages='\n'.join(re.findall(r'^error(?:\[[A-Z0-9]+\])?: (.+)',text,re.MULTILINE))
  bad=status not in [0,1] or 'error[E0463]' in text or 'extern location' in messages or 'file name should be lib' in messages
  actual='accept' if status==0 else 'reject'
  matched=not bad and actual==row['expected'] and (not row['diagnostic'] or row['diagnostic'].lower() in messages.lower())
  artifact={'sha256':sha(target),'bytes':target.stat().st_size} if target.exists() else None
  record={**row,'command':command,'status':status,'seconds':time.monotonic()-started,'matches_prediction':matched,'infrastructure_failure':bad,'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'stderr_sha256':hashlib.sha256(stderr).hexdigest(),'artifact':artifact,'prior_artifact_sha256':row.get('prior_artifact_sha256'),'reconstructed_bytes_equal_prior':artifact['sha256']==row['prior_artifact_sha256'] if artifact and row.get('prior_artifact_sha256') else None}
  if bad:infra.append(row['id'])
  if not matched and not bad:disagreements.append(row['id'])
  if row['kind']=='provider' and row['emit']=='metadata' and status==0:providers[row['provider']]=target
  elif target.exists():target.unlink();record['artifact_removed']=True
  records.append(record);(folder/'result.json').write_text(json.dumps(record,indent=2)+'\n')
 for target in providers.values():
  if target.exists():target.unlink()
 if {p:sha(INPUT/'inputs'/p) for p in plan['inputs']}!=sources or sha(rustc)!=plan['rustc_sha256']:raise ValueError('inputs/tool changed after observations')
 summary={'state':'passed' if not infra and not disagreements else 'failed','plan_sha256':PLAN_SHA,'executed_commands':len(records),'fresh_client_observations':24,'retained_provider_observations':16,'semantic_observations_covered':40,'infrastructure_failures':infra,'disagreements':disagreements,'sources_before_after':sources,'rustc_sha256':sha(rustc),'records':records,'claim':'metadata/object only; no linked runtime/native/CTFE claim'}
 (OUT/'results.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:summary[k] for k in ['state','executed_commands','fresh_client_observations','retained_provider_observations','infrastructure_failures','disagreements']}),flush=True)
 return 0 if summary['state']=='passed' else 1

def main():
 if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
 start=time.monotonic();command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,'.github/focused_refs.py','--worker',str(start+180)]
 process=subprocess.Popen(command,cwd=ROOT,start_new_session=True);status=1
 try:status=process.wait(timeout=max(.01,start+187-time.monotonic()));return status
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=1)
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  status=124;return status
 finally:
  if status:
   try:os.killpg(process.pid,signal.SIGKILL)
   except ProcessLookupError:pass
  OUT.mkdir(parents=True,exist_ok=True);(OUT/'launcher.json').write_text(json.dumps({'status':status,'seconds':time.monotonic()-start,'absolute_budget_seconds':190,'command':command},indent=2)+'\n')
if __name__=='__main__':sys.exit(main())
