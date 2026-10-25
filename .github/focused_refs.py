"""One actual CLI provider/consumer group; authenticated private checks retained."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='8bb973fc823edd51dc870c9489bc1f6b5606ec45';PLAN_SHA='3e2b53d5ba02f3b65646f7307d236a81783d960f5073c21d3116f5eb1bed1ded'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline,folder):
 pp=ROOT/'.github/module-cli-plan.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('private frozen plan changed')
 plan=json.loads(pp.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources'] or any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('source/fixture fingerprint changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('pinned tool mismatch')
 if sha(ROOT/'obj/rust/dev-fast/rustgm')!=plan['expected_cli_sha256']:raise ValueError('mandatory CLI differs from actual qualified compiler')
 OUT.mkdir(parents=True,exist_ok=True);target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True);(target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 binding={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':sha(target),'sources':sources,'compiler_source_sha256':{p:h for p,h in sources.items() if p.endswith('.rs') and p.startswith(('dev/src/','dev/bin/'))},'fixtures':plan['fixtures'],'profile':'dev-fast','qualification':{'state':'pending'},'build':'mandatory unchanged workflow CLI; no helper CLI rebuild'}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1';status=1;steps=None
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]);steps=Steps(ROOT,args)
   steps.receipt.update(sources=sources,fixtures=plan['fixtures'],groups={'suite_linking':plan['selectors']},rustc_sha256=sha(tool))
   try:
    retained=plan['retained_private']
    if hashlib.sha256(json.dumps(sources,sort_keys=True).encode()).hexdigest()!=retained['source_fingerprint_sha256']:raise ValueError('retained exact6 source proof changed')
    steps.receipt['retained_checks']=retained;steps.save()
    common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
    steps.run('clippy-integration',['cargo','clippy',*common,'--test','suite_linking','--','-D','warnings'])
    built=steps.run('build-integration',['cargo','test',*common,'--test','suite_linking','--no-run','--message-format=json']);groups={'suite_linking':plan['selectors']};binary=artifacts(built,groups)['suite_linking']
    listed=steps.run('integration-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run('integration-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');ignored_rows=set(ignored.splitlines());selected=inventory('\n'.join(x for x in listed.splitlines() if x not in ignored_rows),'lib',plan['selectors'])
    if selected!=plan['selectors']:raise ValueError('actual nonignored integration selector differs')
    steps.receipt.update(selected={'suite_linking':selected},binaries={'suite_linking':{'path':str(binary),'sha256':sha(binary)}});steps.save()
    output=steps.run('integration',[binary,'--exact',*selected],cwd=ROOT/'dev');print(output,flush=True)
    counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)
    if len(counts)!=1 or tuple(map(int,counts[0]))!=(1,0,0):raise ValueError('fresh CLI group did not actually pass')
    steps.receipt['state']='passed';status=0
   except BaseException as error:steps.receipt.update(state='failed',error=str(error));raise
   finally:
    cases=ROOT/'obj/parallel/pa31/module-cli'
    if cases.exists():
     folders=list(cases.iterdir())
     if len(folders)!=1 or not folders[0].is_dir() or folders[0].is_symlink():raise ValueError('ambiguous actual CLI case output')
     rows={}
     for path in sorted(folders[0].iterdir()):
      if path.is_symlink() or not path.is_file() or path.stat().st_size>2*1024*1024:raise ValueError('unexpected CLI case artifact')
      rows[path.name]={'sha256':sha(path),'bytes':path.stat().st_size}
     if len(rows)>128:raise ValueError('unexpected CLI evidence cardinality')
     shutil.copytree(folders[0],OUT/'cli-cases');steps.receipt['actual_cli_case_outputs']=rows
    steps.save()
  return status
 finally:
  after=fingerprint(ROOT);unchanged=after==sources and all(sha(ROOT/p)==h for p,h in plan['fixtures'].items()) and sha(tool)==plan['rustc_sha256'] and sha(target)==binding['candidate_sha256'] and sha(ROOT/'obj/rust/dev-fast/rustgm')==binding['candidate_sha256']
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
