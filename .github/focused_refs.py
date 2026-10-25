import pathlib,subprocess,hashlib,json,shutil,sys
if '--worker' not in sys.argv:
 subprocess.run(['timeout','190s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True)
 sys.exit(0)
root=pathlib.Path.cwd();app=root/'obj/rust/dev-fast/rustgm';out=root/'obj/parallel/all-attribute-selected-output-v1';out.mkdir(parents=True)
candidate=out/'candidate';candidate.mkdir();copy=candidate/'rustgm';shutil.copy2(app,copy);(candidate/'.keep-artifacts').touch()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
record={'source_commit':subprocess.check_output(['git','rev-parse','HEAD']).decode().strip(),'candidate_sha256':sha(copy),'compiler_source_sha256':{str(p):sha(p) for p in sorted(pathlib.Path('dev').rglob('*.rs'))},'profile':'dev-fast','rustc':subprocess.check_output(['rustc','--version']).decode().strip()}
(out/'binding.json').write_text(json.dumps(record,indent=2)+'\n')
print('Hydrated current compiler',record['candidate_sha256'])
subprocess.run([sys.executable,'scripts/observe_original_core.py','--worker','--expected-cli-sha256','da5d43430556c8b9dc451de3ebe179005b4b1b843246e54af100c8ab833a6f76','--cli',str(copy),'--producer-binding',str(out/'binding.json'),'--out',str(out/'whole-core-lib024-v1')],check=True)
