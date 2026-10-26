"""Tiny source/backend termination capability probe, not RustGM runtime permission."""
import hashlib,json,os,resource,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from storage import exclusive,check_space,minimum_reserve
import backend
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';PLAN_SHA='98d5b11f15301fe80a225550005ccdd0bfc16b4710563bbd9d063a78f85b0a04'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def program(steps,name,exe,trip):
 argv=[str(exe),*(['trip'] if trip else [])]
 def disable_core():resource.setrlimit(resource.RLIMIT_CORE,(0,0))
 result=subprocess.run(argv,capture_output=True,timeout=5,check=False,preexec_fn=disable_core)
 (steps.folder/(name+'.stdout')).write_bytes(result.stdout);(steps.folder/(name+'.stderr')).write_bytes(result.stderr)
 synchronous={signal.SIGABRT,signal.SIGBUS,signal.SIGFPE,signal.SIGILL,signal.SIGSEGV,signal.SIGTRAP}
 expected=b'before\n' if trip else b'interposed\n7 9\n'
 valid=result.stdout==expected and not result.stderr and ((result.returncode<0 and -result.returncode in synchronous) if trip else result.returncode==0)
 row={'name':name,'argv':argv,'returncode':result.returncode,'class':'intentional-native-abnormal' if trip and valid else 'ordinary-return' if valid else 'unexpected-or-infrastructure','exact_signal_required':False,'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest(),'passed':valid}
 steps.receipt.setdefault('programs',[]).append(row);steps.save();print(json.dumps(row),flush=True)
 if not valid:raise ValueError('native program did not match frozen normal/abnormal class: '+name)
def worker(deadline,folder):
 pp=ROOT/'.github/abort-probe-plan.json';plan=json.loads(pp.read_bytes());before=fingerprint(ROOT)
 if sha(pp)!=PLAN_SHA or before!=plan['sources'] or any(sha(ROOT/p)!=h for p,h in {**plan['fixtures'],**plan['inputs']}.items()):raise ValueError('frozen source/input mismatch')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True).strip());cli=ROOT/'obj/rust/dev-fast/rustgm';cc=Path(subprocess.check_output(['which','cc'],text=True).strip()).resolve();cc_before=sha(cc)
 if sha(tool)!=plan['rustc_sha256'] or sha(cli)!=plan['expected_cli_sha256']:raise ValueError('actual tool/mandatory CLI mismatch')
 OUT.mkdir(parents=True,exist_ok=True);native=OUT/'native';native.mkdir(exist_ok=True);steps=None;status=1
 try:
  with exclusive(ROOT):
   check_space(ROOT,minimum_reserve());steps=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]));steps.receipt.update(sources=before,fixtures=plan['fixtures'],rustc_sha256=sha(tool),cli_sha256=sha(cli),cc={'path':str(cc),'sha256':sha(cc)})
   steps.run('backend-setup',[sys.executable,'scripts/setup_backend.py']);tools=backend.load_tools(('cppgm++',));steps.receipt['backend_sha256']=sha(Path(tools['cppgm++']))
   steps.run('backend-object',[tools['cppgm++'],'-O0','-c',ROOT/'.github/probe.lowir','-o',native/'backend.o']);backend.verify_object(native/'backend.o')
   steps.run('rustc-object',[tool,'--edition=2024','--target','x86_64-unknown-linux-gnu','--crate-type=lib','--crate-name','source_abort_capability','-C','panic=abort','--emit=obj',ROOT/'.github/provider.rs','-o',native/'rustc.o'],overrides={'RUSTC_BOOTSTRAP':'1'})
   for tag in ('rustc','backend'):
    steps.run(tag+'-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',ROOT/'.github/observer.c',native/(tag+'.o'),'-o',native/tag]);program(steps,tag+'-normal',native/tag,False);program(steps,tag+'-trip',native/tag,True)
   steps.receipt['binaries']={p.name:sha(p) for p in native.iterdir() if p.is_file()};steps.receipt['state']='passed';steps.save();status=0
   for p in native.iterdir():
    if p.is_file():p.unlink()
  return status
 finally:
  unchanged=sha(cc)==cc_before and before==fingerprint(ROOT) and sha(tool)==plan['rustc_sha256'] and sha(cli)==plan['expected_cli_sha256'] and all(sha(ROOT/p)==h for p,h in {**plan['fixtures'],**plan['inputs']}.items())
  if steps:
   steps.receipt.update(sources_after=fingerprint(ROOT),source_tool_fixture_prepost_exact=unchanged,state='passed' if status==0 and unchanged else 'failed');steps.save()
   if status==0 and unchanged:(OUT/'probe-result.json').write_text(json.dumps({'state':'passed','receipt_sha256':sha(folder/'receipt.json'),'programs':steps.receipt['programs'],'scope':'actual pinned Rust abort/source symbol control +standalone locked backend Unreachable capability only; RustGM source Abort remains denied until authenticated consumer'},indent=2)+'\n')
  if not unchanged:raise ValueError('probe source/tool changed')
def main():
 if len(sys.argv)==4 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]),Path(sys.argv[3]))
 started=time.monotonic();folder=ROOT/f'obj/qualification/fast/{time.time_ns()}';cmd=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(started+120),str(folder)];p=subprocess.Popen(cmd,start_new_session=True)
 try:return p.wait(timeout=125)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=2)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=1)
  return 124
if __name__=='__main__':sys.exit(main())
