"""Driver-only native execution over actual checked source Not exports."""
import hashlib,json,shutil,subprocess
from pathlib import Path

def execute(steps,root,out,plan,tool):
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 retained=root/'.github/retained-native';prior=retained/'receipt.json';pp=retained/'prior-plan.json'
 if sha(prior)!=plan['retained_native_receipt_sha256'] or sha(pp)!=plan['retained_native_plan_sha256']:raise ValueError('retained independent native binding changed')
 old=json.loads(prior.read_bytes());before=json.loads(pp.read_bytes())
 if old['sources']!=old['sources_after'] or old['sources']!=before['sources'] or old['rustc_sha256']!=sha(tool):raise ValueError('retained source/tool fingerprints invalid')
 for path in ['.github/native-provider.rs','.github/native-observer.c']:
  if sha(root/path)!=before['fixtures'][path] or sha(root/path)!=plan['fixtures'][path]:raise ValueError('independent source/observer changed')
 names=['rustc-native-object','rustc-native-link','rustc-native-run'];passed=[s for s in old['steps'] if s['name'] in names]
 if len(passed)!=3 or any(s['status']!=0 for s in passed):raise ValueError('independent native did not actually pass')
 observed=(retained/'rustc-native-run.stdout').read_text()
 if observed!=plan['expected_stdout'] or (retained/'rustc-native-run.stderr').read_bytes():raise ValueError('retained native output differs')
 cc=Path(shutil.which('cc') or '').resolve()
 if str(cc)!=passed[1]['command'][0]:raise ValueError('new observer linker path differs from actual retained command')
 result={'cc':str(cc),'cc_sha256':sha(cc),'cc_hash_scope':'fresh candidate observer linker only; prior C linker SHA was not persisted; no byte-identity claim for prior C executable','rustc_object_sha256':None,'rustc_executable_sha256':None,'rustc_stdout':observed,'retained_run':40271737959,'receipt_sha256':sha(prior),'source_sha256':sha(root/'.github/native-provider.rs'),'tool_sha256':sha(tool),'steps':passed,'all_old_source_tool_prepost_exact':True,'actual_native_raw_files':{str(p.relative_to(retained)):sha(p) for p in retained.iterdir() if p.is_file()}}
 steps.receipt['retained_independent_native']=result;steps.save();print('RETAIN exact genuine pinned Rust source/object/link/run from40271737959',flush=True)
 return result

def candidate(steps,root,out,plan,independent):
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 folder=out/'native';actual=folder/'provider.rs';lowir=folder/'provider.lowir'
 if sha(actual)!=plan['provider_sha256']:raise ValueError('private checked provider differs from independently frozen source')
 steps.run('backend-setup',['python3','scripts/setup_backend.py'])
 import backend
 tools=backend.load_tools(('cppgm++',))
 steps.run('candidate-native-object',[tools['cppgm++'],'-O0','-c',lowir,'-o',folder/'candidate.o']);backend.verify_object(folder/'candidate.o')
 cc=Path(independent['cc'])
 if sha(cc)!=independent['cc_sha256']:raise ValueError('C observer compiler changed')
 steps.run('candidate-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',root/'.github/native-observer.c',folder/'candidate.o','-o',folder/'candidate-native'])
 observed=steps.run('candidate-native-run',[folder/'candidate-native'])
 if observed!=independent['rustc_stdout'] or observed!=plan['expected_stdout']:raise ValueError('candidate native differs from genuine independent native/frozen output')
 result={'state':'passed','provider_sha256':sha(actual),'lowir_sha256':sha(lowir),'cppgm_object_backend_sha256':sha(Path(tools['cppgm++'])),'candidate_object_sha256':sha(folder/'candidate.o'),'candidate_executable_sha256':sha(folder/'candidate-native'),'candidate_stdout':observed,'independent':independent,'permission':'same-source actual SizedTypeProperties constants/layout/Clone caller bodies; no bootstrap factory, imported contracts, Assert/panic/Abort or foreign Provided grant'}
 (folder/'native-result.json').write_text(json.dumps(result,indent=2)+'\n');steps.receipt['native']=result;steps.save()
 for name in ('rustc.o','rustc-native','candidate.o','candidate-native'):(folder/name).unlink(missing_ok=True)
