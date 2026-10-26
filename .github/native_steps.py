"""Pinned Rust only: frozen language hypotheses and one genuine native provider."""
from pathlib import Path
import hashlib,json,subprocess,time,shutil
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def execute(steps,root,out,plan,tool):
 independent=json.loads((root/'.github/independent-plan.json').read_bytes());folder=out/'independent';folder.mkdir(exist_ok=True)
 sysroot=Path(subprocess.check_output([tool,'--print','sysroot'],text=True,timeout=5).strip())
 libraries=sorted((sysroot/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('*.rlib'))
 if not 1<=len(libraries)<=128:raise ValueError('pinned target library inventory differs')
 library_hashes={p.name:sha(p) for p in libraries}
 steps.receipt['rust_target_libraries']=library_hashes;steps.save()
 rows=[];disagreements=[];positive_object=None
 for ordinal,case in enumerate(independent['cases']):
  source=root/'.github/independent'/case['source']
  if sha(source)!=case['sha256']:raise ValueError('frozen source changed: '+case['name'])
  for mode in independent['modes']:
   destination=folder/f'libtransmute_{ordinal}.{"o" if mode=="obj" else "rmeta"}'
   argv=[str(tool),'--edition=2024','--target','x86_64-unknown-linux-gnu','--crate-type=lib','--crate-name',f'transmute_{ordinal}','-C','panic=abort','--error-format=json','--emit='+mode,str(source),'-o',str(destination)]
   env={**steps.environment};env.pop('RUSTC_BOOTSTRAP',None)
   if case['bootstrap']:env['RUSTC_BOOTSTRAP']='1'
   remaining=steps.args.deadline-time.monotonic()
   if remaining<=0:raise TimeoutError('independent deadline before command')
   result=subprocess.run(argv,cwd=root,env=env,capture_output=True,timeout=min(10,remaining),check=False)
   tag=case['name']+'-'+mode
   (steps.folder/(tag+'.stdout')).write_bytes(result.stdout);(steps.folder/(tag+'.stderr')).write_bytes(result.stderr)
   diagnostics=[]
   for line in result.stderr.decode().splitlines():
    item=json.loads(line)
    if item.get('$message_type')=='diagnostic':diagnostics.append(item)
   errors=[x for x in diagnostics if x['level']=='error'];codes=[x['code']['code'] for x in errors if x.get('code')]
   if result.returncode not in (0,1) or 'E0463' in codes or any('internal compiler error' in x['message'] for x in diagnostics):raise ValueError('independent infrastructure failure: '+tag)
   predicted=case['prediction'];accept=result.returncode==0
   passed=accept==predicted['accept'] and (accept or all(code in codes for code in predicted['diagnostic_codes']))
   row={'name':case['name'],'mode':mode,'argv':argv,'status':result.returncode,'source_sha256':sha(source),'bootstrap':case['bootstrap'],'prediction':predicted,'actual_accept':accept,'diagnostics':[{'message':x['message'],'code':x.get('code'),'level':x['level']} for x in diagnostics],'passed':passed,'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest(),'artifact_sha256':sha(destination) if destination.exists() else None}
   rows.append(row)
   if not passed:disagreements.append(tag)
   if case['name']=='positive-owned-native' and mode=='obj' and accept:positive_object=destination
   elif destination.exists():destination.unlink()
   steps.receipt.update(independent_observations=rows,independent_disagreements=disagreements);steps.save()
 if len(rows)!=independent['observations']:raise ValueError('independent count differs from frozen plan')
 native=None
 if positive_object:
  cc=Path(shutil.which('cc') or '').resolve();cc_hash=sha(cc);exe=folder/'rustc-native';observer=root/'.github/independent/observer.c'
  if sha(observer)!=independent['observer_sha256']:raise ValueError('observer changed')
  steps.run('rustc-native-link',[cc,'-std=c11','-Wall','-Wextra','-Werror',observer,positive_object,'-o',exe]);actual=steps.run('rustc-native-run',[exe])
  if actual!=independent['native_prediction']['expected_stdout'] or (steps.folder/'rustc-native-run.stderr').read_bytes():raise ValueError('independent native disagreed with frozen prediction')
  native={'cc':str(cc),'cc_sha256':cc_hash,'object_sha256':sha(positive_object),'executable_sha256':sha(exe),'provider_sha256':independent['cases'][0]['sha256'],'observer_sha256':sha(observer),'stdout':actual,'status':0}
  if sha(cc)!=cc_hash:raise ValueError('observer tool changed')
  exe.unlink();positive_object.unlink()
 if {p.name:sha(p) for p in libraries}!=library_hashes:raise ValueError('pinned target libraries changed')
 steps.receipt.update(independent_native=native,independent_complete=True);steps.save()
 if disagreements:raise ValueError('retained language hypothesis disagreements: '+', '.join(disagreements))
 return {'state':'passed','observations':len(rows),'native':native,'scope':'pinned Rust contracts ONLY; candidate transmute remains unavailable'}
def candidate(steps,root,out,plan,independent):
 steps.receipt['baseline_scope']='actual old private continuation layouts +independent Rust only; NO candidate transmute admission/execution';steps.save()
