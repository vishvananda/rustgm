"""Frozen independent and candidate terminal/owned-scope native observations."""
import hashlib,json,resource,shutil,signal,subprocess,time
from pathlib import Path
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def program(steps,name,exe,trip,plan):
 def no_core():resource.setrlimit(resource.RLIMIT_CORE,(0,0))
 argv=[str(exe),*(['trip'] if trip else [])]
 remaining=steps.args.deadline-time.monotonic()
 if remaining<=0:raise TimeoutError('native program deadline exhausted before launch')
 result=subprocess.run(argv,capture_output=True,timeout=min(5,remaining),check=False,preexec_fn=no_core)
 (steps.folder/(name+'.stdout')).write_bytes(result.stdout);(steps.folder/(name+'.stderr')).write_bytes(result.stderr)
 synchronous={signal.SIGABRT,signal.SIGBUS,signal.SIGFPE,signal.SIGILL,signal.SIGSEGV,signal.SIGTRAP}
 expected=b'before\n' if trip else b'interposed\ndropped\n7 9 11\n'
 valid=result.stdout==expected and not result.stderr and ((result.returncode<0 and -result.returncode in synchronous) if trip else result.returncode==0)
 row={'name':name,'argv':argv,'returncode':result.returncode,'class':'intentional-native-abnormal' if trip and valid else 'ordinary-return' if valid else 'unexpected-or-infrastructure','exact_signal_required':False,'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest(),'passed':valid,'provider_sha256':plan['provider_sha256']}
 steps.receipt.setdefault('programs',[]).append(row);steps.save();print(json.dumps(row),flush=True)
 if not valid:raise ValueError('native program disagreed with frozen classification: '+name)
 return row

def execute(steps,root,out,plan,tool):
 folder=out/'native/source-abort';folder.mkdir(parents=True,exist_ok=True)
 cc=Path(shutil.which('cc') or '').resolve();before=sha(cc)
 steps.run('rustc-native-object',[tool,'--edition=2024','--target','x86_64-unknown-linux-gnu','--crate-type=lib','--crate-name','source_abort_contract','-C','panic=abort','--emit=obj',root/'.github/native-provider.rs','-o',folder/'rustc.o'],overrides={'RUSTC_BOOTSTRAP':'1'})
 steps.run('rustc-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',root/'.github/native-observer.c',folder/'rustc.o','-o',folder/'rustc-native'])
 observations=[program(steps,'rustc-native-normal',folder/'rustc-native',False,plan),program(steps,'rustc-native-trip',folder/'rustc-native',True,plan)]
 result={'cc':str(cc),'cc_sha256':before,'rustc_object_sha256':sha(folder/'rustc.o'),'rustc_executable_sha256':sha(folder/'rustc-native'),'observations':observations,'provider_sha256':sha(root/'.github/native-provider.rs'),'observer_sha256':sha(root/'.github/native-observer.c'),'rustc_sha256':sha(tool)}
 if sha(cc)!=before:raise ValueError('independent observer compiler changed')
 steps.receipt['independent_native']=result;steps.save()
 for name in ('rustc.o','rustc-native'):(folder/name).unlink()
 return result

def candidate(steps,root,out,plan,independent):
 folder=out/'native/source-abort';actual=folder/'provider.rs';lowir=folder/'provider.lowir'
 if sha(actual)!=plan['provider_sha256']:raise ValueError('actual checked provider differs from frozen independent source')
 steps.run('backend-setup',['python3','scripts/setup_backend.py'])
 import backend
 tools=backend.load_tools(('cppgm++',));cc=Path(independent['cc'])
 if sha(cc)!=independent['cc_sha256']:raise ValueError('candidate observer compiler changed')
 steps.run('candidate-native-object',[tools['cppgm++'],'-O0','-c',lowir,'-o',folder/'candidate.o']);backend.verify_object(folder/'candidate.o')
 steps.run('candidate-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',root/'.github/native-observer.c',folder/'candidate.o','-o',folder/'candidate-native'])
 observations=[program(steps,'candidate-native-normal',folder/'candidate-native',False,plan),program(steps,'candidate-native-trip',folder/'candidate-native',True,plan)]
 if sha(cc)!=independent['cc_sha256']:raise ValueError('candidate observer compiler changed')
 result={'state':'passed','provider_sha256':sha(actual),'lowir_sha256':sha(lowir),'cppgm_object_backend_sha256':sha(Path(tools['cppgm++'])),'object_sha256':sha(folder/'candidate.o'),'executable_sha256':sha(folder/'candidate-native'),'observations':observations,'independent':independent,'scope':'actual source Abort only after checked current header, caller/original/instance and terminal never proof; no Assert/panic/format/CTFE/import/reification/factory/wire/backend grant'}
 (folder/'native-result.json').write_text(json.dumps(result,indent=2)+'\n');steps.receipt['native']=result;steps.save()
 for name in ('candidate.o','candidate-native'):(folder/name).unlink()
