from pathlib import Path
import hashlib,json,shutil,subprocess,sys,time
source='4b195c578a35d98c8308fb265bc5d1ec88ee802d'
folder=Path("obj/parallel/all-attribute-selected-output-v1");folder.mkdir(parents=True,exist_ok=True)
status=1;started=time.monotonic()
try:
 subprocess.run([sys.executable,"scripts/storage.py","run","--no-prune","--","timeout","190s","obj/audit-venv/bin/python",".github/consumer_six.py"],check=True,timeout=195)
 status=0
finally:
 cli=Path("obj/rust/dev-fast/rustgm")
 if cli.exists():shutil.copyfile(cli,folder/"rustgm")
 (folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"state":"passed" if status==0 else "failed","cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest() if cli.exists() else None,"seconds":time.monotonic()-started,"scope":"member consumer statics and exact6; public execution fenced; no oracle/native/ref/full sweeps"},indent=2)+"\n")
