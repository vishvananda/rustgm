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
 retained=root/'.github/retained-v3';prior=json.loads((retained/'receipt.json').read_bytes())
 if sha(retained/'receipt.json')!=plan['retained_independent']['receipt_sha256'] or prior['sources']!=prior['sources_after'] or not prior['fixture_before_after_unchanged']:raise ValueError('retained independent source proof changed')
 actual=prior['independent_native']
 if actual['provider_sha256']!=sha(root/'.github/native-provider.rs') or actual['observer_sha256']!=sha(root/'.github/native-observer.c') or actual['rustc_sha256']!=sha(tool):raise ValueError('retained independent source/tool/observer changed')
 rows=actual['observations']
 if len(rows)!=2 or not all(row['passed'] for row in rows):raise ValueError('retained independent observations did not pass')
 for row in rows:
  for suffix in ('stdout','stderr'):
   if sha(retained/(row['name']+'.'+suffix))!=row[suffix+'_sha256']:raise ValueError('independent raw observation changed')
 cc=Path(shutil.which('cc') or '').resolve()
 if str(cc)!=actual['cc'] or sha(cc)!=actual['cc_sha256']:raise ValueError('actual observer compiler differs from retained independent tool')
 steps.receipt['retained_independent_native']={**actual,'run':40280027733,'receipt_sha256':sha(retained/'receipt.json')};steps.save()
 print('RETAIN genuine owned-source Rust normal/trip native from40280027733',flush=True)
 return actual

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
