from pathlib import Path
import hashlib,json,shutil,subprocess,sys,time
source='f0b79b561880f1cb254c2da77b3d510462d3bf99'
folder=Path("obj/parallel/all-attribute-selected-output-v1");folder.mkdir(parents=True,exist_ok=True)
status=1;started=time.monotonic()
try:
 subprocess.run([sys.executable,"scripts/storage.py","run","--no-prune","--","timeout","190s","obj/audit-venv/bin/python",".github/edition_execution.py"],check=True,timeout=195)
 status=0
finally:
 cli=Path("obj/rust/dev-fast/rustgm")
 if cli.exists():shutil.copyfile(cli,folder/"rustgm")
 (folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"state":"passed" if status==0 else "failed","cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest() if cli.exists() else None,"seconds":time.monotonic()-started,"scope":"edition frontend2 and only missing natives11/negatives4; originalsClone/layout remain pendingB"},indent=2)+"\n")
