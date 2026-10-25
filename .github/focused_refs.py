from pathlib import Path
import hashlib,json,os,subprocess,sys,time
if '--worker' not in sys.argv:
 subprocess.run(['timeout','150s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True,timeout=155)
 sys.exit(0)
started=time.monotonic();planpath=Path('.github/graph_plan.json');plan=json.loads(planpath.read_text());out=Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir(parents=True)
rustc=Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True,timeout=5).strip());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(rustc)==plan['rustc_sha256']
rows=[];infra=[];disagreements=[]
def invoke(name,phase,command,expected_status,source,expected_stdout=None,expected_stderr=None):
 assert time.monotonic()-started<120
 result=subprocess.run(command,env={**os.environ,**plan['environment']},capture_output=True,timeout=5)
 (out/(name+'.'+phase+'.stdout')).write_bytes(result.stdout);(out/(name+'.'+phase+'.stderr')).write_bytes(result.stderr)
 diagnostics=[] if phase=='native' else [json.loads(line) for line in result.stderr.splitlines() if line.startswith(b'{')]
 warnings=[d['code']['code'] if d.get('code') else None for d in diagnostics if d.get('level')=='warning']
 mismatch=result.returncode!=expected_status or phase!='native' and warnings!=[] or expected_stdout is not None and result.stdout!=expected_stdout.encode() or expected_stderr is not None and result.stderr!=expected_stderr.encode()
 row={'name':name,'phase':phase,'command':command,'source_sha256':sha(source),'status':result.returncode,'expected_status':expected_status,'warning_codes':warnings,'errors':[d.get('message') for d in diagnostics if d.get('level')=='error'],'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest(),'disagreement':mismatch};rows.append(row)
 if mismatch:disagreements.append(row)
 if phase!='native' and result.returncode not in (0,1) or result.returncode==86:infra.append(row)
 return result.returncode
try:
 for case in plan['cases']:
  source=Path('.github/graph_sources')/(case['name']+'.rs');assert sha(source)==case['sha256']
  for edition in plan['editions']:
   name=case['name']+'-'+edition;metadata=out/(name+'.rmeta')
   command=[str(rustc),'--edition='+edition,*plan['metadata_flags'],str(source),'-o',str(metadata)]
   invoke(name,'metadata',command,case['expected_metadata_status'],source);metadata.unlink(missing_ok=True)
   if case['native'] is not None:
    binary=out/(name+'.native');command=[str(rustc),'--edition='+edition,*plan['native_flags'],str(source),'-o',str(binary)]
    build=invoke(name,'native-build',command,case['native']['build_status'],source)
    if build==0:
     status=invoke(name,'native',[str(binary.resolve())],case['native']['exit_status'],source,case['native']['stdout'],case['native']['stderr'])
     if status==0:binary.unlink()
finally:
 binding={'plan':plan,'plan_sha256':sha(planpath),'ci_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'rustc_sha256':sha(rustc),'seconds':time.monotonic()-started,'observations':rows,'infra_failures':infra,'disagreements':disagreements,'native_executed':sum(r['phase']=='native' for r in rows)}
 (out/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 print(json.dumps({'commands':len(rows),'native_executed':binding['native_executed'],'infra':len(infra),'seconds':binding['seconds'],'disagreements':disagreements},indent=2),flush=True)
assert sum(r['phase']=='metadata' for r in rows)==plan['planned_metadata_observations'] and not infra
