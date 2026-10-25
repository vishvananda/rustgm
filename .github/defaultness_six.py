from pathlib import Path
from types import SimpleNamespace
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,re,subprocess,sys,time
sys.path.insert(0,str(Path.cwd()/"scripts"))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
root=Path.cwd();started=time.monotonic();freeze=json.loads(Path(".github/defaultness_freeze.json").read_text());current=fingerprint(root)
assert current==freeze["sources"],"source differs from frozen checkpoint"
for path,sha in freeze["fixtures"].items():assert hashlib.sha256((root/path).read_bytes()).hexdigest()==sha,path
assert hashlib.sha256((root/"pa31/tests/spec/defaultness/contract.json").read_bytes()).hexdigest()==freeze["independent_contract_sha256"]
tool=subprocess.check_output(["rustup","which","--toolchain","1.98.0","rustc"],text=True,timeout=5).strip()
assert hashlib.sha256(Path(tool).read_bytes()).hexdigest()=="3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2"
folder=root/("obj/qualification/fast/"+str(time.time_ns())+"-defaultness")
args=SimpleNamespace(evidence_root=folder,deadline=started+180,prune_under=[])
steps=Steps(root,args);selectors=freeze["selectors"]
steps.receipt.update(state="running",sources=current,product_source=freeze["source"],groups={"lib":selectors});steps.save()
audit=Steps(root,SimpleNamespace(evidence_root=Path(str(folder)+"-audit"),deadline=args.deadline,prune_under=[]))
audit.receipt.update(sources=current,product_source=freeze["source"],state="running");audit.save()
def architecture():
 try:
  audit.run("architecture",[root/"obj/audit-venv/bin/python","scripts/rust_audit.py","--root",".","--cache","obj/fast-audit.json"])
  audit.receipt["state"]="passed"
 except BaseException as error:
  audit.receipt.update(state="failed",error=str(error));raise
 finally:audit.save()
try:
 steps.run("assignment-metadata",[sys.executable,"scripts/assignment_metadata.py"])
 steps.run("format",["cargo","fmt","--manifest-path","dev/Cargo.toml","--all","--","--check"])
 with ThreadPoolExecutor(max_workers=1) as executor:
  future=executor.submit(architecture)
  common=["--manifest-path","dev/Cargo.toml","--locked","--profile","dev-fast"]
  steps.run("clippy-production",["cargo","clippy",*common,"--lib","--bin","rustgm","--","-D","warnings"])
  steps.run("clippy-private",["cargo","clippy",*common,"--lib","--","--test","-D","warnings"],overrides={"CARGO_TARGET_DIR":str(root/"obj/rust/private-lint")})
  future.result()
  built=steps.run("build",["cargo","test",*common,"--lib","--no-run","--message-format=json"])
 binary=artifacts(built,{"lib":selectors})["lib"]
 listed=steps.run("lib-inventory",[binary,"--list"],cwd=root/"dev")
 ignored=set(steps.run("lib-ignored",[binary,"--ignored","--list"],cwd=root/"dev").splitlines())
 selected=inventory("\n".join(line for line in listed.splitlines() if line not in ignored),"lib",selectors);assert len(selected)==6
 cli=root/"obj/rust/dev-fast/rustgm"
 steps.receipt.update(selected={"lib":selected},binaries={"lib":{"path":str(binary),"sha256":hashlib.sha256(binary.read_bytes()).hexdigest()},"cli":{"path":str(cli),"sha256":hashlib.sha256(cli.read_bytes()).hexdigest()}},architecture_receipt=str(audit.folder.relative_to(root)/"receipt.json"));steps.save()
 output=steps.run("lib",[binary,"--exact",*selected],cwd=root/"dev",overrides={"RUST_TEST_NOCAPTURE":"1"});print(output,flush=True)
 assert re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",output)==[("6","0","0")]
 assert fingerprint(root)==current,"source changed during check"
 steps.receipt["state"]="passed"
except BaseException as error:
 steps.receipt.update(state="failed",error=str(error));raise
finally:
 steps.save();print("Defaultness evidence:",steps.folder,flush=True)
