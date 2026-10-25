from pathlib import Path
import hashlib,json,shutil,subprocess,sys,time
source='72cf23043a3ad5af0c9e159ce257af00f4ea368b';expected='123138617b80b8bdf39ad7c9bce4ec0df48618e6e649efbadf3755e983ac817c'
if '--worker' not in sys.argv:
 subprocess.run(['timeout','190s',sys.executable,'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=True,timeout=195)
 sys.exit(0)
root=Path.cwd();folder=root/'obj/parallel/all-attribute-selected-output-v1';folder.mkdir(parents=True,exist_ok=False)
app=root/'obj/rust/dev-fast/rustgm';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(app)==expected,'fresh CLI differs from actual qualified CLI'
freeze=json.loads(Path('.github/observer_freeze.json').read_text())
assert all(sha(root/path)==digest for path,digest in freeze['sources'].items()),'source differs from six-control qualified freeze'
copy=folder/'rustgm';shutil.copy2(app,copy)
record={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'product_source':source,'candidate_sha256':expected,'compiler_source_sha256':{path:digest for path,digest in freeze['sources'].items() if path.startswith('dev/') and path.endswith('.rs')},'profile':'dev-fast','qualified_test_run':40223575234,'qualified_receipt_sha256':'e5522a088e26d036c2d53337649c15bb186551a84931ed544353ee93400ff7a4'}
(folder/'binding.json').write_text(json.dumps(record,indent=2)+'\n')
subprocess.run([sys.executable,'scripts/observe_original_core.py','--worker','--expected-cli-sha256',expected,'--cli',str(copy),'--producer-binding',str(folder/'binding.json'),'--out',str(folder/'whole-core-lib027-v1')],check=True)
