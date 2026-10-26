"""Driver-only native execution over actual checked source Not exports."""
import hashlib,json,shutil,subprocess
from pathlib import Path

def execute(steps,root,out,plan,tool):
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 folder=out/'native';folder.mkdir(parents=True,exist_ok=True)
 source=root/'.github/native-provider.rs';observer=root/'.github/native-observer.c'
 cc=Path(shutil.which('cc') or '');
 if not cc.is_file():raise ValueError('C linker/observer compiler is unavailable')
 cc=cc.resolve();cc_sha=sha(cc)
 steps.run('rustc-native-object',[tool,'--edition=2024','--target','x86_64-unknown-linux-gnu','--crate-type=lib','--crate-name','source_not_native','-C','panic=abort','--emit=obj',source,'-o',folder/'rustc.o'],overrides={'RUSTC_BOOTSTRAP':'1'})
 steps.run('rustc-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',observer,folder/'rustc.o','-o',folder/'rustc-native'])
 observed=steps.run('rustc-native-run',[folder/'rustc-native'])
 if observed.encode()!=plan['expected_stdout'].encode():raise ValueError('independent native result disagrees with frozen prediction')
 return {'cc':str(cc),'cc_sha256':cc_sha,'rustc_object_sha256':sha(folder/'rustc.o'),'rustc_executable_sha256':sha(folder/'rustc-native'),'rustc_stdout':observed}

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
 result={'state':'passed','provider_sha256':sha(actual),'lowir_sha256':sha(lowir),'cppgm_object_backend_sha256':sha(Path(tools['cppgm++'])),'candidate_object_sha256':sha(folder/'candidate.o'),'candidate_executable_sha256':sha(folder/'candidate-native'),'candidate_stdout':observed,'independent':independent,'permission':'same-source local actual Not/body/const proof; no foreign Not transport, Assert,panic or Abort grant'}
 (folder/'native-result.json').write_text(json.dumps(result,indent=2)+'\n');steps.receipt['native']=result;steps.save()
 for name in ('rustc.o','rustc-native','candidate.o','candidate-native'):(folder/name).unlink()
