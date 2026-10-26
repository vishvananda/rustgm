"""Exact v2 private controls; one Rust command at a time, independent audit joined."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='15b8acddc9df46feddc928c06fe7ba607f4ef075';PLAN_SHA='dff0fce42b893c32d30f006ddc5f5c536b72277fce56a7080b7ec3672df5d2da'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def observe_layout(steps,tool,plan):
 observer=ROOT/'.github/layout_observer.rs'
 if sha(observer)!=plan['observer_sha256']:raise ValueError('identical after observer changed')
 deps=ROOT/'obj/rust/dev-fast/deps';libraries=list(deps.glob('librustgm-*.rlib'))
 if len(libraries)!=1:raise ValueError('ambiguous actual production rlib')
 library=libraries[0];library_hash=sha(library);cli=ROOT/'obj/rust/dev-fast/rustgm';cli_hash=sha(cli)
 executable=OUT/'layout-observer'
 command=[str(tool),str(observer),'--edition=2024','--target=x86_64-unknown-linux-gnu','-Cdebuginfo=0','--extern','rustgm='+str(library),'-L','dependency='+str(deps),'-o',str(executable)]
 steps.run('after-layout-compile',command)
 actual=json.loads(steps.run('after-layout',[executable]))
 if actual['pointer_bytes']!=8:raise ValueError('unexpected observer target')
 for name in ['SourceCore','Resolution','Package','Declaration','IntrinsicDeclaration','LangItem','SourceIntrinsic','Intrinsic']:
  if actual[name]!=plan['before_layouts'][name]:raise ValueError('unexpected enclosing layout growth: '+name)
 if actual['Reservations']!=832:raise ValueError('unexpected cold Reservations capacity')
 binding={'state':'actual-after-layout-claims-only','layouts':actual,'before_binding_sha256':plan['before_binding_sha256'],'observer_sha256':sha(observer),'rlib':str(library.relative_to(ROOT)),'rlib_sha256':library_hash,'cli_sha256':cli_hash,'rustc_sha256':sha(tool),'target':'x86_64-unknown-linux-gnu','command':command,'observer_elf_sha256':sha(executable)}
 if sha(library)!=library_hash or sha(cli)!=cli_hash:raise ValueError('observer changed producer artifacts')
 executable.unlink();binding['observer_elf_removed_after_terminal']=True
 (OUT/'after-layout.json').write_text(json.dumps(binding,indent=2)+'\n');steps.receipt['after_layout']=binding;steps.save()
 print(json.dumps(binding),flush=True)

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
    steps.run('assignment-metadata',[sys.executable,'scripts/assignment_metadata.py'])
    steps.run('format',['cargo','fmt','--manifest-path','dev/Cargo.toml','--all','--','--check'])
    observe_layout(steps,tool,plan)
    audit=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=folder/'architecture',prune_under=[]))
    with ThreadPoolExecutor(max_workers=1) as pool:
     pending=pool.submit(audit.run,'architecture',[ROOT/'obj/audit-venv/bin/python','scripts/rust_audit.py','--root','.','--cache','obj/fast-audit.json'])
     try:
      common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
      static_errors=[]
      for label,command,overrides in [
       ('clippy-production',['cargo','clippy',*common,'--lib','--bin','rustgm','--','-D','warnings'],{}),
       ('clippy-private',['cargo','clippy',*common,'--lib','--','--test','-D','warnings'],{'CARGO_TARGET_DIR':str(ROOT/'obj/rust/private-lint')})]:
       try:steps.run(label,command,overrides=overrides)
       except BaseException as error:static_errors.append(str(error))
      try:pending.result();audit.receipt['state']='passed'
      except BaseException as error:audit.receipt.update(state='failed',error=str(error));static_errors.append(str(error))
      if static_errors:raise ValueError('static checks failed: '+'; '.join(static_errors))
      built=steps.run('build',['cargo','test',*common,'--lib','--no-run','--message-format=json']);groups={'lib':plan['selectors']};binary=artifacts(built,groups)['lib']
      listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');ignored_rows=set(ignored.splitlines());selected=inventory('\n'.join(x for x in listed.splitlines() if x not in ignored_rows),'lib',plan['selectors'])
      if sorted(selected)!=sorted(plan['selectors']):raise ValueError('nonignored exact selectors differ')
      steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':sha(binary)}});steps.save()
      output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev');print(output,flush=True)
      counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)
      if len(counts)!=1 or tuple(map(int,counts[0]))!=(6,0,0):raise ValueError('executed count differs from exact6')
      pending.result();audit.receipt['state']='passed'
     except BaseException as error:
      try:pending.result()
      except BaseException as audit_error:audit.receipt.update(state='failed',error=str(audit_error))
      raise error
     finally:audit.save()
    steps.receipt['architecture']={'receipt':str((audit.folder/'receipt.json').relative_to(ROOT)),'sha256':sha(audit.folder/'receipt.json')}
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
