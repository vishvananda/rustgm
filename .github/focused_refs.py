"""One actual CLI provider/consumer group; authenticated private checks retained."""
import hashlib,json,os,re,shutil,signal,subprocess,sys,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
CANDIDATE='6e510e9bf97eaf723205c5a069a7928d19897f76';PLAN_SHA='2f4b1202054d6302fbacfaba965ba8d3d972a972d15fbf5ca7f0728a942feb99'
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline,folder):
 pp=ROOT/'.github/module-cli-plan.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('private frozen plan changed')
 plan=json.loads(pp.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources'] or any(sha(ROOT/p)!=h for p,h in plan['fixtures'].items()):raise ValueError('source/fixture fingerprint changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('pinned tool mismatch')
 mandatory_hash=sha(ROOT/'obj/rust/dev-fast/rustgm')
 if plan['expected_cli_sha256'] is not None and mandatory_hash!=plan['expected_cli_sha256']:raise ValueError('mandatory CLI differs from qualified compiler')
 OUT.mkdir(parents=True,exist_ok=True);target=OUT/'cli/candidate/rustgm';target.parent.mkdir(parents=True,exist_ok=True);(target.parent/'.keep-artifacts').touch();shutil.copy2(ROOT/'obj/rust/dev-fast/rustgm',target);target.chmod(0o755)
 binding={'source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'candidate_sha256':sha(target),'sources':sources,'compiler_source_sha256':{p:h for p,h in sources.items() if p.endswith('.rs') and p.startswith(('dev/src/','dev/bin/'))},'fixtures':plan['fixtures'],'profile':'dev-fast','qualification':{'state':'pending'},'build':'mandatory unchanged workflow CLI; no helper CLI rebuild'}
 (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 os.environ['RUST_TEST_NOCAPTURE']='1';status=1;steps=None
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());args=SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]);steps=Steps(ROOT,args)
   steps.receipt.update(sources=sources,fixtures=plan['fixtures'],groups={'suite_linking':plan['selectors']},rustc_sha256=sha(tool))
   try:
    steps.receipt['component_evidence']=plan['component_evidence'];steps.save()
    common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
    steps.run('assignment-metadata',[sys.executable,'scripts/assignment_metadata.py'])
    steps.run('format',['cargo','fmt','--manifest-path','dev/Cargo.toml','--all','--','--check'])
    audit=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=folder/'architecture',prune_under=[]))
    with ThreadPoolExecutor(max_workers=1) as pool:
     pending=pool.submit(audit.run,'architecture',[ROOT/'obj/audit-venv/bin/python','scripts/rust_audit.py','--root','.','--cache','obj/fast-audit.json']);errors=[]
     for label,command in [('clippy-production',['cargo','clippy',*common,'--lib','--bin','rustgm','--','-D','warnings']),('clippy-integration',['cargo','clippy',*common,'--test','suite_linking','--','-D','warnings'])]:
      try:steps.run(label,command)
      except BaseException as error:errors.append(str(error))
     try:pending.result();audit.receipt['state']='passed'
     except BaseException as error:audit.receipt.update(state='failed',error=str(error));errors.append(str(error))
     finally:audit.save()
     steps.receipt['architecture']={'receipt':str((audit.folder/'receipt.json').relative_to(ROOT)),'sha256':sha(audit.folder/'receipt.json')};steps.save()
     if errors:raise ValueError('static checks failed: '+'; '.join(errors))
    built=steps.run('build-integration',['cargo','test',*common,'--test','suite_linking','--no-run','--message-format=json']);groups={'suite_linking':plan['selectors']};binary=artifacts(built,groups)['suite_linking']
    records=[json.loads(line) for line in built.splitlines()]
    compiled=[r for r in records if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('kind')==['bin'] and r['target'].get('name')=='rustgm' and not r.get('profile',{}).get('test')]
    libraries=[r for r in records if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('kind')==['lib'] and r['target'].get('name')=='rustgm' and not r.get('profile',{}).get('test')]
    if len(compiled)!=1 or len(libraries)!=1:raise ValueError('ambiguous actual production compiler/library artifacts')
    actual_cli=Path(compiled[0]['executable']);rlibs=[Path(p) for p in libraries[0]['filenames'] if p.endswith('.rlib')]
    if actual_cli!=ROOT/'obj/rust/dev-fast/rustgm' or len(rlibs)!=1:raise ValueError('unexpected CARGO_BIN_EXE or real library path')
    library=rlibs[0];actual_hash=sha(actual_cli);library_hash=sha(library)
    mandatory=OUT/'mandatory-cli/candidate/rustgm';mandatory.parent.mkdir(parents=True);(mandatory.parent/'.keep-artifacts').touch();shutil.move(target,mandatory)
    if sha(mandatory)!=mandatory_hash:raise ValueError('mandatory production CLI changed')
    shutil.copy2(actual_cli,target)
    binding.update(candidate_sha256=actual_hash,build='actual production bin+rlib artifacts from one necessary suite_linking Cargo build; profile.test=false',mandatory_cli={'path':str(mandatory.relative_to(ROOT)),'sha256':sha(mandatory),'build':'unchanged workflow mandatory production CLI'},actual_production={'cli_path':str(actual_cli),'cli_sha256':actual_hash,'bin_profile':compiled[0]['profile'],'rlib_path':str(library),'rlib_sha256':library_hash,'rlib_profile':libraries[0]['profile']})
    (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');steps.receipt['actual_production']=binding['actual_production'];steps.save()
    listed=steps.run('integration-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run('integration-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');ignored_rows=set(ignored.splitlines());selected=inventory('\n'.join(x for x in listed.splitlines() if x not in ignored_rows),'lib',plan['selectors'])
    if selected!=plan['selectors']:raise ValueError('actual nonignored integration selector differs')
    steps.receipt.update(selected={'suite_linking':selected},binaries={'suite_linking':{'path':str(binary),'sha256':sha(binary)}});steps.save()
    before={'actual_cli':sha(actual_cli),'immutable_pin':sha(target),'rlib':sha(library)};expected={'actual_cli':actual_hash,'immutable_pin':actual_hash,'rlib':library_hash}
    if before!=expected:raise ValueError('actual compiler changed before test run')
    steps.receipt['compiler_before_test']=before;steps.save()
    try:output=steps.run('integration',[binary,'--exact',*selected],cwd=ROOT/'dev');print(output,flush=True)
    finally:
     after_cli={'actual_cli':sha(actual_cli),'immutable_pin':sha(target),'rlib':sha(library)};steps.receipt['compiler_after_test']=after_cli;steps.save()
     if after_cli!=expected:raise ValueError('actual compiler/library changed through test run')
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
