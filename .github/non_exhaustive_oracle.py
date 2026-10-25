import pathlib,subprocess,json,hashlib
root=pathlib.Path.cwd();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();out=root/'obj/parallel/all-attribute-selected-output-v1';out.mkdir(parents=True)
contract=json.loads((root/'.github/non_exhaustive_sources/contract.json').read_text());rustc=pathlib.Path(subprocess.check_output(['rustup','which','--toolchain','1.98.0','rustc'],text=True).strip());assert sha(rustc)==contract['rustc_sha256'],sha(rustc)
records=[];mismatches=[]
for case in contract['cases']:
 source=root/'.github/non_exhaustive_sources'/pathlib.Path(case['path']).name;assert sha(source)==case['source_sha256']
 for edition in contract['editions']:
  folder=out/case['name']/str(edition);folder.mkdir(parents=True)
  command=[str(rustc),*contract['profile'],'--edition='+str(edition),'--crate-name','control_'+case['name'].replace('-','_'),str(source),'-o',str(folder/'output.rmeta')]
  result=subprocess.run(command,capture_output=True,timeout=15)
  for name,data in [('stdout',result.stdout),('stderr',result.stderr)]: (folder/name).write_bytes(data)
  messages=[json.loads(line) for line in result.stderr.decode().splitlines() if line.startswith('{')]
  warnings=sorted({m['code']['code'] for m in messages if m.get('level')=='warning' and m.get('code')})
  infrastructure=result.returncode not in(0,1) or (result.returncode==1 and not any(m.get('level')=='error' for m in messages)) or b'panicked at' in result.stderr
  row={'name':case['name'],'edition':edition,'source_sha256':sha(source),'command':command,'status':result.returncode,'warnings':warnings,'predicted_status':case['exit_status'],'predicted_warnings':case['warnings'],'infrastructure_failure':infrastructure,'stdout_sha256':sha(folder/'stdout'),'stderr_sha256':sha(folder/'stderr')}
  row['prediction_matched']=result.returncode==case['exit_status'] and warnings==sorted(case['warnings']);records.append(row)
  if not row['prediction_matched']:mismatches.append({k:row[k] for k in ['name','edition','status','warnings','predicted_status','predicted_warnings']})
  (out/'independent-results.json').write_text(json.dumps({'rustc_sha256':sha(rustc),'observations':records,'prediction_mismatches':mismatches},indent=2)+'\n')
  (folder/'output.rmeta').unlink(missing_ok=True)
assert len(records)==contract['expected_observations'] and not any(r['infrastructure_failure'] for r in records)
print('INDEPENDENT OBSERVATIONS',len(records),'prediction mismatches',json.dumps(mismatches))
