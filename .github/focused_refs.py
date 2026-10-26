"""Exact v2 private controls; one Rust command at a time, independent audit joined."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='d86f4d861dd767feb4addf410055a8d2b6ca16ce';PLAN_SHA='35fb410b0b3f18fc37cf7da8477865369c6bd63ab1f4009b32b5b09dd99ac1c3'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def retain_checks(steps,plan,sources,tool):
 prior=ROOT/'.github/retained/receipt.json';architecture=ROOT/'.github/retained/architecture/receipt.json'
 if sha(prior)!=plan['retained_receipt_sha256'] or sha(architecture)!=plan['retained_architecture_sha256']:raise ValueError('retained receipts changed')
 old=json.loads(prior.read_bytes());audit=json.loads(architecture.read_bytes())
 if old['sources']!=sources or old['sources_after']!=sources or old['rustc_sha256']!=sha(tool):raise ValueError('retained full source/tool mismatch')
 if old['state']!='failed' or audit['state']!='passed' or any(s['status']!=0 for s in audit['steps']):raise ValueError('unexpected predecessor states')
 names=['assignment-metadata','format','after-layout-compile','after-layout','clippy-production','clippy-private']
 successful=[s for s in old['steps'] if s['name'] in names]
 if len(successful)!=len(names) or any(s['status']!=0 for s in successful):raise ValueError('retained static/layout steps did not pass')
 cli=ROOT/'obj/rust/dev-fast/rustgm';layout=old['after_layout'];library=ROOT/layout['rlib']
 if sha(cli)!=old['cli']['candidate_sha256'] or sha(library)!=layout['rlib_sha256']:raise ValueError('current mandatory producer differs from retained exact layout producer')
 steps.receipt['retained_checks']={'run':40258235902,'prior_state':old['state'],'receipt_sha256':sha(prior),'architecture_sha256':sha(architecture),'source_tool_equality':'every before/after fingerprint exact','steps':successful,'raw_files':plan['fixtures']}
 steps.receipt['after_layout']={**layout,'retained_from_run':40258235902};steps.save()
def worker(deadline,folder):
 pp=ROOT/'.github/assert-plan.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('private frozen plan changed')
 plan=json.loads(pp.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources'] or any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('source/fixture fingerprint changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('pinned tool mismatch')
 OUT.mkdir(parents=True,exist_ok=True);target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True);(target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 binding={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':sha(target),'sources':sources,'compiler_source_sha256':{p:h for p,h in sources.items() if p.endswith('.rs') and p.startswith(('dev/src/','dev/bin/'))},'fixtures':plan['fixtures'],'profile':'dev-fast','qualification':{'state':'pending'},'build':'mandatory unchanged workflow CLI; no helper CLI rebuild'}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1';status=1;steps=None;audit=None
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]);steps=Steps(ROOT,args)
   steps.receipt.update(sources=sources,fixtures=plan['fixtures'],groups={'lib':plan['selectors']},rustc_sha256=sha(tool))
   try:
    retain_checks(steps,plan,sources,tool)
    common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
    built=steps.run('build',['cargo','test',*common,'--lib','--no-run','--message-format=json']);groups={'lib':plan['selectors']};binary=artifacts(built,groups)['lib']
    listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');ignored_rows=set(ignored.splitlines());selected=inventory('\n'.join(x for x in listed.splitlines() if x not in ignored_rows),'lib',plan['selectors'])
    if sorted(selected)!=sorted(plan['selectors']):raise ValueError('nonignored exact selectors differ')
    steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':sha(binary)}});steps.save()
    output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev');print(output,flush=True)
    counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)
    if len(counts)!=1 or tuple(map(int,counts[0]))!=(6,0,0):raise ValueError('executed count differs from exact6')
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
