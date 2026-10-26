import hashlib,json,os,pathlib,subprocess,time
root=pathlib.Path.cwd();source=root/'pa31/tests/spec/trait-aliases/naming-v1'
out=root/'obj/parallel/all-attribute-selected-output-v1/independent-naming';out.mkdir(parents=True)
rustc=pathlib.Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True).strip())
version=subprocess.check_output([rustc,'-vV'],text=True)
assert '88d9e12ae178fab0fb5cc050a94da85685d449ea' in version,version
plan=json.loads((source/'contract.json').read_text());observed=[];started=time.monotonic()
prior_raw=(root/'.github/retained-naming.json').read_bytes();prior=json.loads(prior_raw)
assert prior['rustc_sha256']==hashlib.sha256(rustc.read_bytes()).hexdigest() and prior['infrastructure_failures']==0
for row in plan['cases']:
 path=source/row['source']
 if 'observation' in row:
  previous=next(v for v in prior['observed'] if v['source']==row['source'])
  assert hashlib.sha256(path.read_bytes()).hexdigest()==previous['source_sha256']
  assert previous['success']==row['rustc_success']
  observed.append({**previous,'retained_from':40278863399,'current_expectation_matches':True});continue
 dest=out/path.stem;dest.mkdir()
 command=[str(rustc),'--edition=2024','--crate-type=lib','--emit=metadata','--crate-name','naming_case',str(path),'-o',str(dest/'case.rmeta')]
 result=subprocess.run(command,env={**os.environ,'RUSTC_BOOTSTRAP':'1'},capture_output=True,timeout=15,check=False)
 (dest/'stdout').write_bytes(result.stdout);(dest/'stderr').write_bytes(result.stderr)
 assert result.returncode in(0,1),(row['source'],result.returncode)
 actual=result.returncode==0
 observed.append({'source':row['source'],'source_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'command':command,'status':result.returncode,'success':actual,'expected_success':row['rustc_success'],'matches_prediction':actual==row['rustc_success'],'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest()})
 (dest/'case.rmeta').unlink(missing_ok=True)
receipt={'rustc_sha256':hashlib.sha256(rustc.read_bytes()).hexdigest(),'version':version,'observed':observed,'seconds':time.monotonic()-started,'commands':sum('retained_from'not in row for row in observed),'retained_commands':sum('retained_from'in row for row in observed),'prior_receipt_sha256':hashlib.sha256(prior_raw).hexdigest(),'infrastructure_failures':0}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
assert all(row.get('current_expectation_matches',row['matches_prediction'])for row in observed),'pinned Rust disagrees with frozen naming hypotheses; retain observations before changing expectations'
