"""Execute only the candidate; the frozen pinned Rust source/native evidence is retained."""
import hashlib,json,shutil
from pathlib import Path
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def candidate(steps,root,out,plan):
 folder=out/'native/source-transmute';provider=folder/'provider.rs';lowir=folder/'provider.lowir'
 retained=plan['retained_independent'];old=retained['native'];observer=root/'.github/native-observer.c'
 if sha(provider)!=old['provider_sha256'] or sha(observer)!=old['observer_sha256'] or old['status']!=0 or old['stdout']!=plan['expected_native_stdout']:raise ValueError('candidate source/observer differs from genuine frozen independent native proof')
 cc=Path(shutil.which('cc') or '').resolve()
 if str(cc)!=old['cc'] or sha(cc)!=old['cc_sha256']:raise ValueError('candidate observer compiler differs from frozen independent tool')
 steps.receipt['retained_independent_native']=retained;steps.save()
 steps.run('backend-setup',['python3','scripts/setup_backend.py'])
 import backend
 tools=backend.load_tools(('cppgm++',))
 steps.run('candidate-native-object',[tools['cppgm++'],'-O0','-c',lowir,'-o',folder/'candidate.o']);backend.verify_object(folder/'candidate.o')
 steps.run('candidate-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',observer,folder/'candidate.o','-o',folder/'candidate-native'])
 text=steps.run('candidate-native-run',[folder/'candidate-native'])
 actual=(steps.folder/'candidate-native-run.stdout').read_bytes();error=(steps.folder/'candidate-native-run.stderr').read_bytes()
 if text!=plan['expected_native_stdout'] or actual!=old['stdout'].encode() or error:raise ValueError('candidate native bits/ownership/provenance outcome differs from frozen independent observation')
 result={'state':'passed','provider_sha256':sha(provider),'lowir_sha256':sha(lowir),'observer_sha256':sha(observer),'cc':str(cc),'cc_sha256':sha(cc),'backend_sha256':sha(Path(tools['cppgm++'])),'object_sha256':sha(folder/'candidate.o'),'executable_sha256':sha(folder/'candidate-native'),'stdout_sha256':hashlib.sha256(actual).hexdigest(),'stderr_sha256':hashlib.sha256(error).hexdigest(),'retained_independent':retained,'scope':plan['scope']}
 steps.receipt['native']=result;steps.save();(folder/'native-result.json').write_text(json.dumps(result,indent=2)+'\n')
 for name in ['candidate.o','candidate-native']:(folder/name).unlink()
