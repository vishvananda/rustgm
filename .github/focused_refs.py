"""Observe actual public predecessor layouts through its real compiled Rust library."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
from storage import exclusive,check_space,minimum_reserve
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
PLAN_SHA='2db5c97a84e371fab4c6d2b18f776f2ff4aef6701fc82c054519bff3f7f1fc8b';CANDIDATE='7d8b60a466032fd0e3518594d7678ba1741df6b3';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline):
 plan_path=ROOT/'.github/layout-plan.json'
 if sha(plan_path)!=PLAN_SHA:raise ValueError('layout plan changed')
 plan=json.loads(plan_path.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources']:raise ValueError('predecessor full-source mismatch')
 observer=ROOT/'.github/layout_observer.rs'
 if sha(observer)!=plan['observer_sha256']:raise ValueError('observer source mismatch')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['rustc_sha256']:raise ValueError('pinned rustc changed')
 deps=ROOT/'obj/rust/dev-fast/deps';libraries=list(deps.glob('librustgm-*.rlib'))
 if len(libraries)!=1:raise ValueError('ambiguous actual compiled library: '+repr(libraries))
 library=libraries[0];lh=sha(library);cli=ROOT/'obj/rust/dev-fast/rustgm';ch=sha(cli)
 with exclusive(ROOT):
  check_space(ROOT,minimum_reserve());OUT.mkdir(parents=True,exist_ok=True)
  executable=OUT/'layout-observer';command=[str(tool),str(observer),'--edition=2024','--target=x86_64-unknown-linux-gnu','-Cdebuginfo=0','--extern','rustgm='+str(library),'-L','dependency='+str(deps),'-o',str(executable)]
  result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=max(.1,deadline-time.monotonic()),check=False)
  (OUT/'compile.stdout').write_bytes(result.stdout);(OUT/'compile.stderr').write_bytes(result.stderr)
  if result.returncode:raise ValueError('layout observer compile failed: '+result.stderr.decode(errors='replace'))
  executed=subprocess.run([str(executable)],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=max(.1,deadline-time.monotonic()),check=False)
  (OUT/'run.stdout').write_bytes(executed.stdout);(OUT/'run.stderr').write_bytes(executed.stderr)
  if executed.returncode or executed.stderr:raise ValueError('layout observer run failed')
  layouts=json.loads(executed.stdout)
  if layouts['ValidatedRows']!=120 or layouts['Attributes']!=48 or layouts['pointer_bytes']!=8:raise ValueError('qualified predecessor shape differs')
  binding={'state':'passed-real-predecessor-layout-only','source_commit':CANDIDATE,'ci_commit':os.environ.get('GITHUB_SHA'),'sources':sources,'sources_after':fingerprint(ROOT),'rustc_sha256':sha(tool),'target':'x86_64-unknown-linux-gnu','rlib':str(library.relative_to(ROOT)),'rlib_sha256':lh,'cli_sha256':ch,'observer_sha256':sha(observer),'observer_elf_sha256':sha(executable),'command':command,'layouts':layouts,'compile_status':result.returncode,'run_status':executed.returncode,'seconds_scope':'no compiler private tests or independent language observations repeated'}
  if binding['sources_after']!=sources or sha(library)!=lh or sha(cli)!=ch:raise ValueError('source/library/CLI changed')
  executable.unlink();binding['observer_elf_removed_after_verified_terminal']=True
  (OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');print(json.dumps(layouts),flush=True)
 return 0
def main():
 if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
 start=time.monotonic();command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(start+180)]
 process=subprocess.Popen(command,cwd=ROOT,start_new_session=True);status=1
 try:status=process.wait(timeout=max(.01,start+185-time.monotonic()));return status
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=max(.01,start+189-time.monotonic()))
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  status=124;return status
 finally:
  if status:
   try:os.killpg(process.pid,signal.SIGKILL)
   except ProcessLookupError:pass
  OUT.mkdir(parents=True,exist_ok=True);(OUT/'launcher.json').write_text(json.dumps({'status':status,'seconds':time.monotonic()-start,'budget':190},indent=2)+'\n')
if __name__=='__main__':sys.exit(main())
