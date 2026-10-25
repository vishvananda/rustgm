"""Exact v2 private controls; one Rust command at a time, independent audit joined."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='0fa3cf5b4369a0e8a850d5b770078b03de84c364';PLAN_SHA='5834208ccc32c247d73f5e86a96cc690692ce02d5fa21ac51419441f6cc4f340'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline,folder):
 pp=ROOT/'.github/module-private-lint-plan.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('private frozen plan changed')
 plan=json.loads(pp.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources'] or any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('source/fixture fingerprint changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('pinned tool mismatch')
 if sha(ROOT/'obj/rust/dev-fast/rustgm') not in plan['known_source_cli_variants']:raise ValueError('unexpected current-source prebuilt CLI variant')
 OUT.mkdir(parents=True,exist_ok=True);target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True);(target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 binding={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':sha(target),'sources':sources,'compiler_source_sha256':{p:h for p,h in sources.items() if p.endswith('.rs') and p.startswith(('dev/src/','dev/bin/'))},'fixtures':plan['fixtures'],'profile':'dev-fast','qualification':{'state':'pending'},'build':'mandatory unchanged workflow current-source CLI; private lint-only target, actual runtime compiler qualification retained separately'}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1';status=1;steps=None;audit=None
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]);steps=Steps(ROOT,args)
   steps.receipt.update(sources=sources,fixtures=plan['fixtures'],groups={'private-lint-only':['cargo clippy --lib -- --test -D warnings']},rustc_sha256=sha(tool))
   try:
    retained=plan['retained_global']
    if hashlib.sha256(json.dumps(sources,sort_keys=True).encode()).hexdigest()!=retained['source_fingerprint_sha256']:raise ValueError('combined global source proof changed')
    steps.receipt['retained_checks']=retained;steps.save()
    common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
    steps.run('clippy-private',['cargo','clippy',*common,'--lib','--','--test','-D','warnings'],overrides={'CARGO_TARGET_DIR':str(ROOT/'obj/rust/private-lint')})
    steps.receipt['state']='passed';status=0
   except BaseException as error:steps.receipt.update(state='failed',error=str(error));raise
   finally:steps.save()
  return status
 finally:
  after=fingerprint(ROOT);unchanged=after==sources and all(sha(ROOT/p)==h for p,h in plan['fixtures'].items()) and sha(tool)==plan['rustc_sha256'] and sha(target)==binding['candidate_sha256']
  if steps is not None:
   steps.receipt.update(sources_after=after,fixture_before_after_unchanged=unchanged,cli={**binding,'qualification':{'state':'passed' if status==0 and unchanged else 'failed'}});steps.save()
   binding['qualification']={'state':'passed' if status==0 and unchanged else 'failed','receipt':str(folder.relative_to(ROOT)),'receipt_sha256':sha(folder/'receipt.json'),'selectors':plan['selectors']}
   (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');(OUT/'result.json').write_text(json.dumps({'candidate':CANDIDATE,'state':binding['qualification']['state'],'receipt':binding['qualification'],'source_before_after_unchanged':unchanged},indent=2)+'\n')
   print(json.dumps({'candidate':CANDIDATE,'cli_sha256':binding['candidate_sha256'],'state':binding['qualification']['state']}),flush=True)
  if not unchanged:raise ValueError('source/tool/CLI changed during qualification')
def main():
 if len(sys.argv)==4 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]),Path(sys.argv[3]))
 start=time.monotonic();folder=ROOT/f'obj/qualification/fast/{time.time_ns()}';command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(start+185),str(folder)]
 process=subprocess.Popen(command,cwd=ROOT,start_new_session=True);status=1
 try:status=process.wait(timeout=max(.01,start+187-time.monotonic()));return status
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=max(.01,start+189-time.monotonic()))
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  status=124;return status
 finally:
  if status:
   try:os.killpg(process.pid,signal.SIGKILL)
   except ProcessLookupError:pass
  folder.parent.mkdir(parents=True,exist_ok=True);folder.with_suffix('.launcher.json').write_text(json.dumps({'status':status,'seconds':time.monotonic()-start,'budget':190,'command':command},indent=2)+'\n')
if __name__=='__main__':sys.exit(main())
