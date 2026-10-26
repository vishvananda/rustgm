from pathlib import Path
import hashlib,json,os,subprocess,sys,time
if '--worker' not in sys.argv:
 subprocess.run(['timeout','150s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True,timeout=155)
 sys.exit(0)
started=time.monotonic();planpath=Path('.github/pointee-plan.json');plan=json.loads(planpath.read_text());out=Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir(parents=True)
sha=lambda b:hashlib.sha256(b).hexdigest();rustc=Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True,timeout=5).strip());assert sha(rustc.read_bytes())==plan['rustc_sha256'];rows=[];infra=[];mismatches=[]
try:
 for case in plan['cases']:
  source=Path('.github/pointee-sources')/Path(case['source']).name;assert sha(source.read_bytes())==case['sha256']
  (out/source.name).write_bytes(source.read_bytes())
  for edition in plan['editions']:
   assert time.monotonic()-started<120
   name=source.stem+'-'+edition;metadata=out/(name+'.rmeta')
   command=[str(rustc),'--edition='+edition,*plan['flags'],'--crate-name=pointee_metadata_probe',str(source),'-o',str(metadata)]
   result=subprocess.run(command,env={**os.environ,**plan['environment']},capture_output=True,timeout=5)
   (out/(name+'.stdout')).write_bytes(result.stdout);(out/(name+'.stderr')).write_bytes(result.stderr)
   diagnostics=[json.loads(line) for line in result.stderr.splitlines() if line.startswith(b'{')]
   codes=lambda level:sorted({d['code']['code'] for d in diagnostics if d.get('level')==level and d.get('code')})
   warning_codes=codes('warning');error_codes=codes('error')
   mismatch={'status':result.returncode!=case['rustc_status_hypothesis'],'error_codes':error_codes!=sorted(case['rustc_error_codes_hypothesis']),'warning_codes':warning_codes!=sorted(case['rustc_warning_codes_hypothesis'])}
   row={'source':case['source'],'source_sha256':case['sha256'],'edition':edition,'command':command,'status':result.returncode,'stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr),'error_codes':error_codes,'warning_codes':warning_codes,'diagnostics':diagnostics,'disagreements':mismatch};rows.append(row)
   if any(mismatch.values()):mismatches.append(row)
   if result.returncode not in (0,1) or b'internal compiler error' in result.stderr:infra.append(row)
   metadata.unlink(missing_ok=True)
finally:
 binding={'plan':plan,'plan_sha256':sha(planpath.read_bytes()),'ci_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'rustc_sha256':sha(rustc.read_bytes()),'rustc_version':subprocess.check_output([str(rustc),'-vV'],text=True,timeout=5),'seconds':time.monotonic()-started,'observations':rows,'infra_failures':infra,'disagreements':mismatches,'native_executed':0}
 (out/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
 print(json.dumps({'metadata':len(rows),'infra':len(infra),'disagreements':len(mismatches),'seconds':binding['seconds'],'actual':[{'source':r['source'],'edition':r['edition'],'status':r['status'],'error_codes':r['error_codes'],'warning_codes':r['warning_codes'],'disagreements':r['disagreements']} for r in rows]},indent=2),flush=True)
assert len(rows)==plan['planned_metadata_observations'] and not infra
