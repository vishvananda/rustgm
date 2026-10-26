import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
from storage import exclusive,check_space,minimum_reserve
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';PLAN_SHA='f209f32d8a84bf7ba98827de792d420c77ee007e638560f1a32b70498b7a05a9'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def worker(deadline):
 planfile=ROOT/'.github/factory-plan.json';source=ROOT/'.github/factory.rs';plan=json.loads(planfile.read_bytes());cli=ROOT/'obj/rust/dev-fast/rustgm';tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True).strip());before=fingerprint(ROOT)
 if sha(planfile)!=PLAN_SHA or sha(source)!=plan['source_sha256'] or sha(cli)!=plan['cli_sha256'] or sha(tool)!=plan['tool_sha256'] or before!=plan['sources']:raise ValueError('predecessor input/binary/tool binding mismatch')
 with exclusive(ROOT):
  check_space(ROOT,minimum_reserve());OUT.mkdir(parents=True,exist_ok=True)
  command=[str(cli),'--edition=2024','--crate-type=bin','--emit-lowir',str(source)]
  observed=subprocess.run(command,capture_output=True,timeout=max(.1,deadline-time.monotonic()),check=False)
  (OUT/'stdout').write_bytes(observed.stdout);(OUT/'stderr').write_bytes(observed.stderr)
  after=fingerprint(ROOT);unchanged=after==before and sha(source)==plan['source_sha256'] and sha(cli)==plan['cli_sha256'] and sha(tool)==plan['tool_sha256']
  result={'state':'completed' if observed.returncode in (0,1,2) and unchanged else 'infrastructure-failed','status':observed.returncode,'expected_status':1,'matches_prediction':observed.returncode==1 and b'imported must_use contract transport is not implemented' in observed.stderr,'command':command,'source_commit':plan['source_commit'],'ci_commit':os.environ.get('GITHUB_SHA'),'sources_before':before,'sources_after':after,'cli_sha256':sha(cli),'rustc_sha256':sha(tool),'source_sha256':sha(source),'stdout_sha256':hashlib.sha256(observed.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(observed.stderr).hexdigest(),'unchanged':unchanged}
  (OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['sources_before','sources_after']}));print(observed.stderr.decode(errors='replace'))
  return 0 if result['state']=='completed' else 1
def main():
 if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
 started=time.monotonic();command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(started+40)];process=subprocess.Popen(command,start_new_session=True)
 try:return process.wait(timeout=45)
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=2)
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  return 124
if __name__=='__main__':sys.exit(main())
