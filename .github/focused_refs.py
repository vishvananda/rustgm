"""New pinned no_core Not observations; no RustGM private build or native execution."""
import hashlib,io,json,os,signal,struct,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import fingerprint
from storage import exclusive,check_space,minimum_reserve
OUT=ROOT/'obj/parallel/all-attribute-selected-output-v1';PLAN_SHA='8dac9eb30319ee4a13a5e3f61a41545eeef294a9f42b595e18a4ee3db3c0c9e0';SOURCE='93fe092ffd50759b9849f5735d9cc72a9ed2b534'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def static_byte(path):
 # The retained LIB024 exported-static inspection algorithm, bounded to one byte.
 data=path.read_bytes()
 if len(data)>1024*1024 or data[:6]!=b'\x7fELF\x02\x01':raise ValueError('expected bounded real ELF64 little endian')
 offset=struct.unpack_from('<Q',data,40)[0];stride,count=struct.unpack_from('<HH',data,58)
 if stride!=64 or not 0<count<4096:raise ValueError('unexpected ELF section shape')
 sections=[struct.unpack_from('<IIQQQQIIQQ',data,offset+i*stride)for i in range(count)];found=[]
 for table in sections:
  if table[1]!=2:continue
  strings=sections[table[6]];names=data[strings[4]:strings[4]+strings[5]]
  if table[9]!=24:raise ValueError('unexpected ELF symbol shape')
  for pos in range(table[4],table[4]+table[5],24):
   name,info,_,index,value,size=struct.unpack_from('<IBBHQQ',data,pos);end=names.find(b'\0',name)
   if names[name:end]!=b'ANSWER':continue
   if info&15!=1 or not 0<index<len(sections):raise ValueError('nonobject ANSWER symbol')
   owner=sections[index]
   if size!=1 or owner[1]==8 or value+size>owner[5]:raise ValueError('ANSWER has no genuine bool byte')
   raw=data[owner[4]+value:owner[4]+value+size]
   if len(raw)!=1:raise ValueError('truncated static')
   found.append({'section':index,'offset':value,'actual_hex':raw.hex()})
 if len(found)!=1:raise ValueError('missing/duplicate ANSWER object symbol')
 return found[0]
def worker(deadline):
 pp=ROOT/'.github/not-oracle.json'
 if sha(pp)!=PLAN_SHA:raise ValueError('frozen oracle plan changed')
 plan=json.loads(pp.read_bytes());sources=fingerprint(ROOT)
 if sources!=plan['sources']:raise ValueError('compiler source binding changed')
 tool=Path(subprocess.check_output(['rustup','which','rustc'],text=True,timeout=10).strip())
 if sha(tool)!=plan['tool_sha256']:raise ValueError('pinned rustc changed')
 if any(sha(ROOT/r['source'])!=r['sha256'] for r in plan['rows']):raise ValueError('source reducers changed')
 OUT.mkdir(parents=True,exist_ok=True);observations=[];infra=[]
 with exclusive(ROOT):
  check_space(ROOT,minimum_reserve())
  for row in plan['rows']:
   for emit in plan['emits']:
    folder=OUT/'observed'/(row['id']+'-'+emit);folder.mkdir(parents=True,exist_ok=False);artifact=folder/('actual.o' if emit=='obj' else 'actual.rmeta')
    command=[str(tool),'--edition='+plan['edition'],'--crate-type=lib','--crate-name=not_source_contract','--target='+plan['target'],'-Cpanic=abort','--error-format=json','--emit='+emit,str(ROOT/row['source']),'-o',str(artifact)]
    observed={'id':row['id'],'emit':emit,'source_sha256':row['sha256'],'command':command,'expected_status':row['expected_status'],'expected_error_code':row['expected_error_code']}
    try:
     completed=subprocess.run(command,env={**os.environ,'RUSTC_BOOTSTRAP':'1'},capture_output=True,timeout=max(.1,min(30,deadline-time.monotonic())),check=False)
     (folder/'stdout').write_bytes(completed.stdout);(folder/'stderr').write_bytes(completed.stderr);codes=[]
     for line in completed.stderr.splitlines():
      try:diag=json.loads(line)
      except (ValueError,UnicodeDecodeError):continue
      if diag.get('code'):codes.append(diag['code']['code'])
     observed.update(status=completed.returncode,error_codes=codes,stdout_sha256=hashlib.sha256(completed.stdout).hexdigest(),stderr_sha256=hashlib.sha256(completed.stderr).hexdigest())
     observed['matches_prediction']=completed.returncode==row['expected_status'] and (row['expected_error_code'] is None or row['expected_error_code'] in codes)
     if completed.returncode not in (0,1) or b'internal compiler error' in completed.stderr:infra.append(row['id']+': nonlanguage failure')
     if artifact.exists():
      observed.update(artifact_sha256=sha(artifact),artifact_bytes=artifact.stat().st_size)
      if completed.returncode==0 and emit=='obj' and row['answer_byte_hex'] is not None:
       observed['actual_static']=static_byte(artifact);observed['static_matches']=observed['actual_static']['actual_hex']==row['answer_byte_hex'];observed['matches_prediction'] &= observed['static_matches']
      artifact.unlink();observed['compiler_output_removed_after_hash']=True
    except (OSError,ValueError,IndexError,struct.error,subprocess.TimeoutExpired) as error:infra.append(row['id']+': '+str(error));observed['infrastructure_error']=str(error)
    observations.append(observed)
    (OUT/'results.json').write_text(json.dumps({'observations':observations,'infrastructure_errors':infra,'runtime_execution':False},indent=2)+'\n')
  unchanged=fingerprint(ROOT)==sources and sha(tool)==plan['tool_sha256'] and all(sha(ROOT/r['source'])==r['sha256'] for r in plan['rows'])
  result={'state':'completed' if not infra and unchanged and len(observations)==plan['commands'] else 'infrastructure-failed','source_commit':SOURCE,'ci_commit':os.environ.get('GITHUB_SHA'),'source_before':sources,'source_after':fingerprint(ROOT),'rustc_sha256':sha(tool),'plan_sha256':sha(pp),'sources_unchanged':unchanged,'observations':observations,'infrastructure_errors':infra,'predictions_match':all(r.get('matches_prediction',False) for r in observations),'runtime_execution':False}
  (OUT/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['source_before','source_after']}),flush=True)
  return 0 if result['state']=='completed' else 1
def main():
 if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
 start=time.monotonic();command=[sys.executable,'scripts/resources.py','run','--wait-seconds','0','--',sys.executable,'.github/focused_refs.py','--worker',str(start+90)]
 process=subprocess.Popen(command,cwd=ROOT,start_new_session=True)
 try:return process.wait(timeout=95)
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=2)
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=1)
  return 124
if __name__=='__main__':sys.exit(main())
