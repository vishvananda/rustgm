"""Frozen private namespace qualification; no public source admission changes."""
import hashlib,json,os,signal,subprocess,sys,time,shutil
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,execute,fingerprint
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='b09bf823f77e2a6cb3300179edb60e408c68ea1c'
PLAN_SHA='c02c20f3bb755bdac657f1a20ee75ec49e7230f0b599fac6f34d12c76ae458da'
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
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve())
   steps=Steps(ROOT,args)
   try:
    steps.run('clippy-production',['cargo','clippy','--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast','--lib','--bin','rustgm','--','-D','warnings'])
    execute(ROOT,args,steps)
    steps.receipt['state']='passed';status=0
   except BaseException as error:
    steps.receipt.update(state='failed',error=str(error));raise
   finally:steps.save()
  return status
 finally:
  after=fingerprint(ROOT)
  unchanged=after==sources and all(sha(ROOT/p)==h for p,h in plan['fixtures'].items())
  receipt=json.loads((folder/'receipt.json').read_bytes())
  receipt['sources_after']=after
  receipt['cli']={**binding,'qualification':{'state':'passed' if status==0 and unchanged else 'failed'}}
  receipt['fixture_before_after_unchanged']=unchanged
  (folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
  if status==0 and (receipt['state']!='passed' or sorted(receipt['selected']['lib'])!=sorted(plan['selectors'])):raise ValueError('focused selection differs')
  binding['qualification']={'state':'passed' if status==0 and unchanged else 'failed','receipt':str(folder.relative_to(ROOT)),'receipt_sha256':sha(folder/'receipt.json'),'selectors':plan['selectors']}
  (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
  (OUT/'result.json').write_text(json.dumps({'state':binding['qualification']['state'],'candidate':CANDIDATE,'plan_sha256':PLAN_SHA,'receipt':binding['qualification'],'source_before_after_unchanged':unchanged},indent=2)+'\n')
  print(json.dumps({'cli_sha256':binding['candidate_sha256'],'source_commit':CANDIDATE,'state':binding['qualification']['state']}),flush=True)
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
