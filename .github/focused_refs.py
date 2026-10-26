from pathlib import Path
import hashlib,json,os,subprocess,sys,time
if '--worker' not in sys.argv:
 subprocess.run(['timeout','245s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True,timeout=250)
 sys.exit(0)
root=Path.cwd();started=time.monotonic();deadline=started+240
out=Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir(parents=True,exist_ok=True)
plan=json.loads(Path('.github/predecessor-proof.json').read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
receipt={'source':plan['source'],'ci_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'sources':plan['sources'],'prior_qualified_run':plan['prior_green_run'],'prior_qualified_source':plan['prior_qualified_source'],'scope':plan['scope'],'state':'running','steps':[]};binary=None
try:
 for path,expected in plan['sources'].items():assert sha(Path(path))==expected,path
 rustc=Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True,timeout=5).strip())
 receipt['tool']={'path':str(rustc),'sha256':sha(rustc),'version':subprocess.check_output([str(rustc),'-vV'],text=True,timeout=5)}
 env={**os.environ,'CARGO_TARGET_DIR':str(root/'obj/rust'),'CARGO_BUILD_JOBS':'2','RUST_TEST_THREADS':'2','CARGO_INCREMENTAL':'1'}
 def run(name,cmd):
  begun=time.monotonic();result=subprocess.run(cmd,env=env,capture_output=True,timeout=max(1,deadline-begun))
  (out/(name+'.stdout')).write_bytes(result.stdout);(out/(name+'.stderr')).write_bytes(result.stderr)
  receipt['steps'].append({'name':name,'command':cmd,'status':result.returncode,'seconds':time.monotonic()-begun,'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest()})
  assert result.returncode==0, (name,result.stderr.decode()[-6000:]);return result.stdout
 raw=run('private-build',['cargo','test','--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast','--lib','--no-run','--message-format=json'])
 binaries=[Path(row['executable']) for line in raw.splitlines() if line.startswith(b'{') and (row:=json.loads(line)).get('reason')=='compiler-artifact' and row.get('executable') and row.get('target',{}).get('kind')==['lib']]
 assert len(binaries)==1,binaries;binary=binaries[0];receipt['binary_sha256']=sha(binary)
 output=run('predecessor-layout',[str(binary),plan['selector'],'--exact','--nocapture'])
 assert b'1 passed; 0 failed' in output and b'test '+plan['selector'].encode()+b' ... ok' in output,output.decode()
 assert b'Catalog=' in output and b'Reservations=' in output and b'Validated=' in output,output.decode()
 receipt['state']='passed'
finally:
 receipt['seconds']=time.monotonic()-started
 (out/'predecessor-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k!='sources'},indent=2),flush=True)
