from pathlib import Path
import hashlib,json,shutil,subprocess,sys,time
source='ddce34f146e556ef0ab859000d25b57b01d6ac53'
folder=Path("obj/parallel/all-attribute-selected-output-v1");folder.mkdir(parents=True,exist_ok=True)
status=1;started=time.monotonic()
try:
 subprocess.run([sys.executable,"scripts/storage.py","run","--no-prune","--","timeout","190s","obj/audit-venv/bin/python",".github/defaultness_six.py"],check=True,timeout=195)
 status=0
finally:
 cli=Path("obj/rust/dev-fast/rustgm")
 if cli.exists():shutil.copyfile(cli,folder/"rustgm")
 (folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"state":"passed" if status==0 else "failed","cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest() if cli.exists() else None,"seconds":time.monotonic()-started,"scope":"defaultness exact6/static; six root-owned PA5/19 references pending; no oracle/fulltext"},indent=2)+"\n")
