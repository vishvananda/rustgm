import pathlib,subprocess,shutil,json,hashlib
subprocess.run(['timeout','30s','python3','scripts/storage.py','run','--no-prune','--','python3','scripts/setup_backend.py'],check=True)
command=['timeout','--kill-after=5s','185s','python3','scripts/resources.py','run','--wait-seconds','0','--','python3','.github/check_selected_metadata_native.py','--review','.github/selected_reference_review.json','--app',str(pathlib.Path('obj/rust/dev-fast/rustgm').resolve()),'--seconds','180']
subprocess.run(command,check=True)
root=pathlib.Path('obj/parallel/pa31/attribute-native-outcome-closure-v1/runs')
bundles=list(root.glob('*.tar.gz'));assert len(bundles)==1
out=pathlib.Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir()
for bundle in bundles:shutil.copyfile(bundle,out/bundle.name)
(out/'archive-sha256.json').write_text(json.dumps({p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in bundles},indent=2)+'\n')
