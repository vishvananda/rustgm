from pathlib import Path
import hashlib,json,shutil,subprocess,sys,time
source="19a961b805cf06e0084226338fb682651b997c5f"
folder=Path("obj/parallel/all-attribute-selected-output-v1");folder.mkdir(parents=True,exist_ok=True)
status=1;started=time.monotonic()
try:
 subprocess.run([sys.executable,"scripts/storage.py","run","--no-prune","--","timeout","190s","obj/audit-venv/bin/python",".github/missing_six.py"],check=True,timeout=195)
 status=0
finally:
 cli=Path("obj/rust/dev-fast/rustgm")
 if cli.exists():shutil.copyfile(cli,folder/"rustgm")
 (folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"state":"passed" if status==0 else "failed","cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest() if cli.exists() else None,"seconds":time.monotonic()-started,"retained_static_receipt_sha256":'dcd70fb25a58eacbb11429a0891bb2e43afa6cba4c83edd162d935e7913fcce1',"scope":"private build/inventory/exact6 missing only; no static,oracle,reference,fulltext repeat"},indent=2)+"\n")
