import pathlib,subprocess,shutil,json,hashlib
command=['timeout','185s','python3','scripts/resources.py','run','--wait-seconds','0','--','python3','.github/generate_checked_selection.py','--review','.github/selected_reference_review.json','--app',str(pathlib.Path('obj/rust/dev-fast/rustgm').resolve()),'--seconds','180']
subprocess.run(command,check=True)
root=pathlib.Path('obj/parallel/pa31/all-attribute-selected-generation-v1')
bundles=list(root.glob('*.tar.gz'));assert len(bundles)==1
out=pathlib.Path('obj/parallel/all-attribute-selected-output-v1');out.mkdir()
for bundle in bundles:shutil.copyfile(bundle,out/bundle.name)
(out/'archive-sha256.json').write_text(json.dumps({p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in bundles},indent=2)+'\n')
