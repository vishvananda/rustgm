"""Frozen sparse-site checkpoint: both lint modes precede every test build."""
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,os,re,shutil,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path.cwd();sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1'
if '--worker' not in sys.argv:
 raise SystemExit(subprocess.run(['timeout','--signal=TERM','--kill-after=3s','285s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=False).returncode)
started=time.monotonic();OUT.mkdir(parents=True,exist_ok=False)
plan=json.loads((ROOT/'.github/sites-plan.json').read_text());assert fingerprint(ROOT)==plan['sources']
for p,h in plan['fixtures'].items():assert hashlib.sha256((ROOT/p).read_bytes()).hexdigest()==h,p
formatted=subprocess.run(['cargo','fmt','--manifest-path','dev/Cargo.toml','--all'],capture_output=True,timeout=15,check=False)
(OUT/'source-format.stdout').write_bytes(formatted.stdout);(OUT/'source-format.stderr').write_bytes(formatted.stderr);assert formatted.returncode==0
changed=subprocess.check_output(['git','diff','--name-only'],text=True).splitlines();assert set(changed)<=set(plan['format_paths']),changed
for p in changed:
 target=OUT/'formatted'/p;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/p,target)
sources=fingerprint(ROOT);args=SimpleNamespace(evidence_root=ROOT/'obj/qualification/fast/library-contracts-integration-v1',deadline=started+275,prune_under=[])
steps=Steps(ROOT,args);steps.receipt.update(sources=sources,fixtures=plan['fixtures']);steps.save()
common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast'];binding={'state':'running','input_commit':plan['input_commit'],'ci_commit':os.environ.get('GITHUB_SHA'),'sources':sources,'reference_writes':False,'checkpoint':'combined aliases, collected source headers, actual Not/layout/Clone and specialization; full Provided and alias execution/export remain open'}
os.environ['RUST_TEST_NOCAPTURE']='1';steps.environment['RUST_TEST_NOCAPTURE']='1'
try:
 retained=json.loads((ROOT/'.github/integration-retained/receipt.json').read_text());old_audit=json.loads((ROOT/'.github/integration-retained/architecture.json').read_text())
 for path,wanted in plan['retained_files'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==wanted,path
 assert retained['sources']==sources and retained['state']=='failed' and 'selected' not in retained
 assert retained['error']=='selected_tests() takes 3 positional arguments but 82 were given'
 names=['metadata','format','clippy-production','clippy-private','clippy-frontend']
 old_steps=[next(s for s in retained['steps'] if s['name']==name)for name in names]
 assert all(s['status']==0 for s in old_steps)
 assert old_audit['state']=='passed' and all(s['status']==0 for s in old_audit['steps'])
 steps.receipt['retained_static']={'run':40275440268,'all_source_tool_fixtures_exact':True,'files':plan['retained_files'],'steps':old_steps,'architecture':old_audit,'rule':'unchanged compiler/test/tool/fixture bytes; helper-only inventory selector fix; original failed run retained; no test results retained'};steps.save()
 groups={'lib':plan['selected'],'suite_frontend':plan['frontend']}
 built=steps.run('build',['cargo','test',*common,'--lib','--test','suite_frontend','--no-run','--message-format=json']);binaries=artifacts(built,groups)
 selected={}
 for group,binary in binaries.items():
  listed=steps.run(group+'-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=steps.run(group+'-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev');omitted=set(ignored.splitlines());selected[group]=inventory('\n'.join(line for line in listed.splitlines()if line not in omitted),group,groups[group] if group=='lib' else [p.rsplit('::',1) for p in groups[group]]);assert set(selected[group])==set(groups[group])
 steps.receipt.update(selected=selected,binaries={group:{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}for group,binary in binaries.items()});steps.save()
 for group,binary in binaries.items():
  output=steps.run(group+'-runtime',[binary,'--exact',*selected[group]],cwd=ROOT/'dev');print(output,flush=True)
  counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output);assert len(counts)==1 and tuple(map(int,counts[0]))==(len(selected[group]),0,0)
 actual={}
 for family,files in plan['retained_outputs'].items():
  actual[family]={name:hashlib.sha256((OUT/'native'/family/name).read_bytes()).hexdigest()for name in files}
  assert actual[family]==files,(family,actual[family],files)
 steps.receipt['actual_source_lowir_equality']=actual;binding['actual_source_lowir_equality']=actual
 assert fingerprint(ROOT)==sources
 steps.receipt.update(state='passed',sources_after=sources);binding.update(state='passed',selected=selected)
except Exception as error:
 steps.receipt.update(state='failed',error=str(error));binding.update(state='failed',error=str(error));raise
finally:
 steps.save();binding['seconds']=time.monotonic()-started;(OUT/'binding.json').write_text(json.dumps(binding,indent=2)+'\n');print(json.dumps({k:v for k,v in binding.items() if k!='sources'}),flush=True)
