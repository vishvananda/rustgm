"""Missing-only build/exact namespace controls, retaining authenticated same-source statics."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='b09bf823f77e2a6cb3300179edb60e408c68ea1c'
PLAN_SHA='98eb5876d0c4d628e83fa1c9c881d3aca60bb271d386b0ad56249f0e78136d2a'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def worker(deadline,folder):
 pp=ROOT/'.github/namespace-plan.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('continuation frozen plan changed')
 plan=json.loads(pp.read_bytes());proofroot=ROOT/'.github/namespace-proof'
 for p,h in plan['retained']['files'].items():
  if sha(proofroot/p)!=h:raise ValueError('retained raw proof changed '+p)
 prior=json.loads((proofroot/plan['retained']['receipt']).read_bytes())
 wanted=[('clippy-production',0),('assignment-metadata',0),('format',0),('architecture',0),('clippy-private',0),('build',124)]
 if prior['state']!='failed' or [(s['name'],s['status']) for s in prior['steps']]!=wanted:raise ValueError('original timeout/static status differs')
 sources=fingerprint(ROOT)
 if sources!=plan['sources'] or sources!=prior['sources'] or sources!=prior['sources_after']:raise ValueError('all-source equality failed')
 if not prior['fixture_before_after_unchanged'] or any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('source fixture changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('current pinned compiler tool changed')
 OUT.mkdir(parents=True,exist_ok=True);target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True)
 (target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 if sha(target)!=prior['cli']['candidate_sha256']:raise ValueError('same-source mandatory CLI differs from retained compiler')
 binding={**prior['cli'],'ci_commit':os.environ.get('GITHUB_SHA'),'source_commit':CANDIDATE,'qualification':{'state':'pending'},'retained_static_run':40226367780}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1';args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]);status=1;steps=None
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());steps=Steps(ROOT,args)
   steps.receipt.update(sources=sources,fixtures=plan['fixtures'],retained_static={'run':40226367780,'receipt_sha256':sha(proofroot/plan['retained']['receipt']),'raw':plan['retained'],'steps':prior['steps'][:-1]},rustc={'path':str(tool),'sha256':sha(tool)},groups={'lib':plan['selectors']})
   steps.save()
   try:
    groups={'lib':plan['selectors']}
    built=steps.run('build',['cargo','test','--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast','--lib','--no-run','--message-format=json'])
    binary=artifacts(built,groups)['lib']
    listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');ignored_rows=set(ignored.splitlines())
    selected=inventory('\n'.join(row for row in listed.splitlines() if row not in ignored_rows),'lib',plan['selectors'])
    if sorted(selected)!=sorted(plan['selectors']):raise ValueError('actual nonignored selectors differ')
    steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':sha(binary)}});steps.save()
    output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev');print(output,flush=True)
    counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)
    if len(counts)!=1 or tuple(map(int,counts[0]))!=(5,0,0):raise ValueError('executed count differs from exact5')
    steps.receipt['state']='passed';status=0
   except BaseException as error:steps.receipt.update(state='failed',error=str(error));raise
   finally:steps.save()
  return status
 finally:
  after=fingerprint(ROOT);unchanged=after==sources and all(sha(ROOT/p)==h for p,h in plan['fixtures'].items()) and sha(tool)==plan['rustc_sha256'] and sha(target)==binding['candidate_sha256']
  if steps is not None:
   steps.receipt.update(sources_after=after,fixture_before_after_unchanged=unchanged,cli={**binding,'qualification':{'state':'passed' if status==0 and unchanged else 'failed'}});steps.save()
   binding['qualification']={'state':'passed' if status==0 and unchanged else 'failed','receipt':str(folder.relative_to(ROOT)),'receipt_sha256':sha(folder/'receipt.json'),'selectors':plan['selectors']}
   (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');(OUT/'result.json').write_text(json.dumps({'candidate':CANDIDATE,'state':binding['qualification']['state'],'retained_static_run':40226367780,'receipt':binding['qualification'],'source_before_after_unchanged':unchanged},indent=2)+'\n')
   print(json.dumps({'source_commit':CANDIDATE,'cli_sha256':binding['candidate_sha256'],'state':binding['qualification']['state']}),flush=True)
  if not unchanged:raise ValueError('source/tool changed after continuation')

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
  folder.parent.mkdir(parents=True,exist_ok=True);folder.with_suffix('.launcher.json').write_text(json.dumps({'status':status,'seconds':time.monotonic()-start,'absolute_budget_seconds':190,'command':command},indent=2)+'\n')
if __name__=='__main__':sys.exit(main())
