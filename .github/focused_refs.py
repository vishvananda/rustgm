"""Frozen private namespace qualification; no public source admission changes."""
import hashlib,json,os,signal,subprocess,sys,time,shutil
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import check,fingerprint
CANDIDATE='d92636997e171d806d56c551464eb738491cce95'
PLAN_SHA='6da2752c94f1408bbb3a033d8b6eab6a10499150e16076cd6b8dbe5effa5cec5'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline,folder):
 plan_path=ROOT/'.github/namespace-plan.json'
 if sha(plan_path)!=PLAN_SHA:raise ValueError('frozen plan changed')
 plan=json.loads(plan_path.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources']:raise ValueError('source fingerprint differs from frozen candidate')
 if any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('fixture changed')
 OUT.mkdir(parents=True,exist_ok=True)
 target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True)
 (target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 binding={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':sha(target),'sources':sources,'compiler_source_sha256':{p:h for p,h in sources.items() if p.endswith('.rs') and p.startswith(('dev/src/','dev/bin/'))},'fixtures':plan['fixtures'],'profile':'dev-fast','build':'mandatory unchanged workflow CLI; helper does not rebuild CLI','qualification':{'state':'pending'}}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1'
 args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[],suites=[],lib_case=plan['selectors'],ignored=False,pa=[])
 status=1
 try:
  status=check(ROOT,args)
  return status
 finally:
  after=fingerprint(ROOT)
  unchanged=after==sources and all(sha(ROOT/p)==h for p,h in plan['fixtures'].items())
  receipt=json.loads((folder/'receipt.json').read_bytes())
  receipt['sources_after']=after
  receipt['fixture_before_after_unchanged']=unchanged
  (folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
  if status==0 and (receipt['state']!='passed' or sorted(receipt['selected']['lib'])!=sorted(plan['selectors'])):raise ValueError('focused selection differs')
  binding['qualification']={'state':'passed' if status==0 and unchanged else 'failed','receipt':str(folder.relative_to(ROOT)),'receipt_sha256':sha(folder/'receipt.json'),'selectors':plan['selectors']}
  (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
  (OUT/'result.json').write_text(json.dumps({'state':binding['qualification']['state'],'candidate':CANDIDATE,'plan_sha256':PLAN_SHA,'receipt':binding['qualification'],'source_before_after_unchanged':unchanged},indent=2)+'\n')
  if not unchanged:raise ValueError('source/fixture changed after check')

def main():
 if len(sys.argv)==4 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]),Path(sys.argv[3]))
 start=time.monotonic();folder=ROOT/f'obj/qualification/fast/{time.time_ns()}'
 command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(start+185),str(folder)]
 process=subprocess.Popen(command,cwd=ROOT,start_new_session=True);status=1
 try:
  status=process.wait(timeout=max(.01,start+187-time.monotonic()));return status
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=max(.01,start+189-time.monotonic()))
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  status=124;return status
 finally:
  if status:
   try:os.killpg(process.pid,signal.SIGKILL)
   except ProcessLookupError:pass
  folder.parent.mkdir(parents=True,exist_ok=True)
  folder.with_suffix('.launcher.json').write_text(json.dumps({'status':status,'seconds':time.monotonic()-start,'absolute_budget_seconds':190,'command':command},indent=2)+'\n')
if __name__=='__main__':sys.exit(main())
