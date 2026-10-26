"""Exact predeclared specialization CLI/native outcomes, using only locked tools."""
import hashlib,json,re,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/"scripts"))
from backend import load_tools
root=Path.cwd();plan=json.loads(Path('.github/native_plan.json').read_text());out=root/'obj/parallel/all-attribute-selected-output-v1/native';out.mkdir(parents=True,exist_ok=True)
started=time.monotonic();receipt={'state':'running','plan_sha256':hashlib.sha256(Path('.github/native_plan.json').read_bytes()).hexdigest(),'commands':[],'cases':[]};errors=[]
cli=root/'obj/rust/dev-fast/rustgm';receipt['cli_sha256']=hashlib.sha256(cli.read_bytes()).hexdigest()
def save(): (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
def run(argv,label,folder):
 remaining=65-(time.monotonic()-started)
 if remaining<=0:raise TimeoutError('native bounded work exhausted')
 try:r=subprocess.run(list(map(str,argv)),capture_output=True,timeout=min(10,remaining))
 except subprocess.TimeoutExpired as error:raise TimeoutError(f'{label}: command exceeded bound') from error
 (folder/(label+'.stdout')).write_bytes(r.stdout);(folder/(label+'.stderr')).write_bytes(r.stderr)
 receipt['commands'].append({'argv':list(map(str,argv)),'status':r.returncode,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest()});save();return r
try:
 for p,key in [('backend.lock.json','backend_lock_sha256'),('backend.distribution.json','backend_distribution_sha256')]:assert hashlib.sha256(Path(p).read_bytes()).hexdigest()==plan[key],p
 # setup_backend authenticates the pinned published distribution; no backend build.
 setup=run([sys.executable,'scripts/setup_backend.py'],'backend-setup',out);assert setup.returncode==0,setup.stderr
 tools=load_tools();receipt['backend']={name:hashlib.sha256(Path(path).read_bytes()).hexdigest() for name,path in tools.items()};save()
 for lane in ['native','negative']:
  for case in plan[lane]:
   source=root/case['path'];assert hashlib.sha256(source.read_bytes()).hexdigest()==case['sha256'],source
   for edition in plan['editions']:
    folder=out/(source.stem+'-'+edition);folder.mkdir(exist_ok=True);record={'path':case['path'],'edition':edition,'lane':lane,'passed':False};receipt['cases'].append(record)
    try:
     r=run([cli,'--edition='+edition,'--emit-lowir',source],'frontend',folder)
     assert r.returncode==case['compiler_status'],f'compiler status {r.returncode}: {r.stderr!r}'
     if lane=='negative':
      assert not r.stdout and r.stderr,'negative must have empty stdout/diagnostic'
      text=r.stderr.decode()
      if 'RPIT' in case['diagnostic_category']:assert 'associated function with return-position impl Trait in trait cannot be specialized' in text,text
      else:assert re.search('lifetime|outlive|region',text,re.I),text
     else:
      assert r.stdout and not r.stderr,f'positive LowIR/diagnostics {r.stderr!r}'
      lowir=folder/'output.lowir';lowir.write_bytes(r.stdout);validated=folder/'validated.lowir';exe=folder/'program'
      for command,label in [([tools['lowir'],'-o',validated,lowir],'validate'),([tools['lowir2native'],'-O0','-o',exe,validated],'backend'),([exe],'native')]:
       r=run(command,label,folder);assert r.returncode==0 and not r.stderr,f'{label}: {r.returncode} {r.stderr!r}'
       if label=='native':assert not r.stdout,r.stdout
      for path in [lowir,validated,exe]:path.unlink(missing_ok=True)
     record['passed']=True
    except (AssertionError,ValueError,TimeoutError,OSError) as error:
     record['error']=str(error);errors.append(f"{source.name}/{edition}: {error}")
    finally:save()
 receipt['seconds']=time.monotonic()-started
 receipt['state']='passed' if not errors else 'failed';save()
 if errors:raise ValueError('\n'.join(errors))
 assert len(receipt['cases'])==22 and sum(c['lane']=='native' for c in receipt['cases'])==14
 print('Specialization native14 and negative8 passed',flush=True)
except BaseException as error:
 receipt.update(state='failed',error=str(error),seconds=time.monotonic()-started);save();raise
