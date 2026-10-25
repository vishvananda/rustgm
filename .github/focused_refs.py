"""Generate only two independently predeclared PA7 text oracles."""
import hashlib,json,pathlib,shutil,subprocess,sys
root=pathlib.Path.cwd();output=root/'obj/parallel/pa7-cfg-test-output-v1';output.mkdir(parents=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
expected_manifest='a0e1243c9236775481c356b13c0a1efdb99a740aad0af765aa425ea2a746a8d2'
base=root/'pa7/reference.json';base_sha='314c2f82d7631995893e68e1d882c748b8e12be139b71dde32d8d3b3477a070f'
assert sha(base)==base_sha and sha(root/'pa7/tests/manifest.json')==expected_manifest
cases=['tests/spec/scope/cfg-test-disabled.rs','tests/spec/scope/cfg-test-enabled.rs']
source_sha='b16ebe3c03fe49d80d7d016ad07541de8b08912ed01eb04ebda59adbdcb5b90e'
for case in cases:assert sha(root/'pa7'/case)==source_sha
before={p.relative_to(root).as_posix():sha(p) for p in (root/'pa7').rglob('*') if p.is_file() and '.ref' in p.name}
previous=set((root/'pa7/reference-batches').glob('*.json'))
app=root/'obj/rust/dev-fast/rustgm';assert app.is_file()
command=['timeout','90s',sys.executable,'scripts/course.py','pa7','--app',str(app),'--jobs','2','--timeout','10','--evidence-root','obj/parallel/pa31/pa7-cfg-test-selected-reference-v1','--bless']
for case in cases:command+=['--case',case]
subprocess.run(command,check=True)
paths=[]
for case,chosen,other in zip(cases,['9','7'],['7','9']):
 p=root/'pa7'/case
 result=pathlib.Path(str(p)+'.ref').read_bytes()
 assert result.count(('token definition integer "'+chosen+'u32"\n').encode())==1
 assert ('token definition integer "'+other+'u32"\n').encode() not in result
 assert result.startswith(b'macro-matching 1\n')
 assert result.endswith(b'end-macro-matching definitions 1 invocations 1\n')
 lines=result.splitlines()
 assert sum(x.startswith(b'definition ') for x in lines)==1
 assert sum(x.startswith(b'invocation ') for x in lines)==1
 assert pathlib.Path(str(p)+'.ref.stderr').read_bytes()==b''
 assert pathlib.Path(str(p)+'.ref.exit_status').read_bytes()==b'0\n'
 paths.extend(pathlib.Path(str(p)+suffix) for suffix in ['.ref','.ref.stderr','.ref.exit_status'])
assert sha(base)==base_sha
for relative,digest in before.items():
 if root/relative not in paths:assert sha(root/relative)==digest,relative
batches=set((root/'pa7/reference-batches').glob('*.json'))-previous
assert len(batches)==1
paths+=list(batches)
for path in paths:
 destination=output/path.relative_to(root);destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,destination)
evidence=root/'obj/parallel/pa31/pa7-cfg-test-selected-reference-v1'
shutil.copytree(evidence,output/'raw-evidence')
source_map={p.as_posix():sha(p) for p in map(pathlib.Path,subprocess.check_output(['git','ls-files','dev']).decode().splitlines()) if p.is_file()}
binding={'ci_commit':subprocess.check_output(['git','rev-parse','HEAD']).decode().strip(),'app_sha256':sha(app),'manifest_sha256':expected_manifest,'base_reference_sha256':base_sha,'selected_paths':cases,'source_sha256':source_sha,'independent_plan_sha256':'476c61e43a28be9744f831c18028183e81c23c9ed4e943123ccb91c4e7214025','independent_results_sha256':'0ba5fc253e350d50f5e4ca74aedd6db13fbdaf505b524070e389576585cf7544','output_sha256':{p.relative_to(root).as_posix():sha(p) for p in paths},'dev_sources':source_map}
(output/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
print('Generated and independently reviewed exactly seven selected PA7 outputs')
