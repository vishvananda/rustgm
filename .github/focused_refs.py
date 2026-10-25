from pathlib import Path
import hashlib,json,os,subprocess,sys,time
if '--worker' not in sys.argv:
 subprocess.run(['timeout','150s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True,timeout=155)
 sys.exit(0)
started=time.monotonic();plan=json.loads(Path('.github/defaultness_plan.json').read_text());out=Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir(parents=True)
rustc=Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True,timeout=5).strip());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(rustc)==plan['rustc_sha256']
rows=[];infra=[]
for case in plan['cases']:
 source=Path('.github/defaultness_sources')/(case['name']+'.rs');assert sha(source)==case['sha256']
 for edition in plan['editions']:
  assert time.monotonic()-started<120
  name=case['name']+'-'+edition;metadata=out/(name+'.rmeta');command=[str(rustc),'--edition='+edition,*plan['flags'],str(source),'-o',str(metadata)]
  result=subprocess.run(command,env={**os.environ,**plan['environment']},capture_output=True,timeout=5)
  (out/(name+'.stdout')).write_bytes(result.stdout);(out/(name+'.stderr')).write_bytes(result.stderr)
  diagnostics=[json.loads(line) for line in result.stderr.splitlines() if line.startswith(b'{')];warnings=[d.get('code',{}).get('code') if d.get('code') else None for d in diagnostics if d.get('level')=='warning']
  row={'case':case['name'],'edition':edition,'command':command,'source_sha256':sha(source),'status':result.returncode,'expected_status':case['expected_status'],'warning_codes':warnings,'expected_warning_codes':case['expected_warning_codes'],'disagreement':result.returncode!=case['expected_status'] or warnings!=case['expected_warning_codes'],'errors':[d.get('message') for d in diagnostics if d.get('level')=='error'],'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest()};rows.append(row)
  if result.returncode not in (0,1):infra.append(row)
  metadata.unlink(missing_ok=True)
(out/'binding.json').write_text(json.dumps({'plan':plan,'plan_sha256':sha(Path('.github/defaultness_plan.json')),'ci_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'rustc_sha256':sha(rustc),'seconds':time.monotonic()-started,'observations':rows,'infra_failures':infra,'disagreements':[r for r in rows if r['disagreement']]},indent=2)+'\n')
print(json.dumps({'count':len(rows),'infra':len(infra),'seconds':time.monotonic()-started,'disagreements':[r for r in rows if r['disagreement']]},indent=2));assert len(rows)==20 and not infra
