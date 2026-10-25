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
priorraw=Path(".github/prior_defaultness.json").read_bytes();assert hashlib.sha256(priorraw).hexdigest()==freeze["retained_static"]["receipt_sha256"]
prior=json.loads(priorraw);changed=sorted(k for k in set(current)|set(prior["sources"]) if current.get(k)!=prior["sources"].get(k))
assert changed==freeze["retained_static"]["changed_only_private_sources"]
assert next(s for s in prior["steps"] if s["name"]=="clippy-production")["status"]==0
raw=Path(".github/prior_defaultness_audit.json").read_bytes();assert hashlib.sha256(raw).hexdigest()==freeze["retained_static"]["audit_receipt_sha256"] and json.loads(raw)["state"]=="passed"
raw=Path(".github/prior_six.stdout").read_bytes();assert hashlib.sha256(raw).hexdigest()==freeze["retained_static"]["output_sha256"]
for selector in freeze["retained_four"]:assert raw.decode().count("test "+selector+" ... ok")==1
metadata=Path(".github/prior_metadata.json").read_bytes();assert hashlib.sha256(metadata).hexdigest()==freeze["retained_assignment_metadata"]["receipt_sha256"]
metadata=json.loads(metadata);assert next(s for s in metadata["steps"] if s["name"]=="assignment-metadata")["status"]==0
assert all(path.endswith(".rs") for path in set(current)|set(metadata["sources"]) if current.get(path)!=metadata["sources"].get(path))
tool=subprocess.check_output(["rustup","which","--toolchain","1.98.0","rustc"],text=True,timeout=5).strip();assert hashlib.sha256(Path(tool).read_bytes()).hexdigest()=="3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2"
folder=root/("obj/qualification/fast/"+str(time.time_ns())+"-defaultness")
args=SimpleNamespace(evidence_root=folder,deadline=started+180,prune_under=[])
steps=Steps(root,args);selectors=freeze["selectors"]
steps.receipt.update(state="running",sources=current,product_source=freeze["source"],groups={"lib":selectors},retained_static=freeze["retained_static"],retained_four=freeze["retained_four"],retained_assignment_metadata=freeze["retained_assignment_metadata"]);steps.save()
audit=Steps(root,SimpleNamespace(evidence_root=Path(str(folder)+"-audit"),deadline=args.deadline,prune_under=[]));audit.receipt.update(sources=current,product_source=freeze["source"],state="running");audit.save()
def architecture():
 try:
  audit.run("architecture",[root/"obj/audit-venv/bin/python","scripts/rust_audit.py","--root",".","--cache","obj/fast-audit.json"]);audit.receipt["state"]="passed"
 except BaseException as error:
  audit.receipt.update(state="failed",error=str(error));raise
 finally:audit.save()
try:
 steps.run("format",["cargo","fmt","--manifest-path","dev/Cargo.toml","--all","--","--check"])
 with ThreadPoolExecutor(max_workers=1) as executor:
  future=executor.submit(architecture);errors=[]
  common=["--manifest-path","dev/Cargo.toml","--locked","--profile","dev-fast"]
  try:steps.run("clippy-private",["cargo","clippy",*common,"--lib","--","--test","-D","warnings"],overrides={"CARGO_TARGET_DIR":str(root/"obj/rust/private-lint")})
  except (ValueError,TimeoutError) as error:errors.append(str(error))
  try:future.result()
  except (ValueError,TimeoutError) as error:errors.append(str(error))
  if errors:raise ValueError("; ".join(errors))
  built=steps.run("build",["cargo","test",*common,"--lib","--no-run","--message-format=json"])
 binary=artifacts(built,{"lib":selectors})["lib"]
 listed=steps.run("lib-inventory",[binary,"--list"],cwd=root/"dev");ignored=set(steps.run("lib-ignored",[binary,"--ignored","--list"],cwd=root/"dev").splitlines())
 selected=inventory("\n".join(line for line in listed.splitlines() if line not in ignored),"lib",selectors);assert len(selected)==2
 cli=root/"obj/rust/dev-fast/rustgm"
 steps.receipt.update(selected={"lib":selected},binaries={"lib":{"path":str(binary),"sha256":hashlib.sha256(binary.read_bytes()).hexdigest()},"cli":{"path":str(cli),"sha256":hashlib.sha256(cli.read_bytes()).hexdigest()}},architecture_receipt=str(audit.folder.relative_to(root)/"receipt.json"));steps.save()
 output=steps.run("lib",[binary,"--exact",*selected],cwd=root/"dev",overrides={"RUST_TEST_NOCAPTURE":"1"});print(output,flush=True)
 assert re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",output)==[("2","0","0")]
 assert fingerprint(root)==current,"source changed during check"
 steps.receipt["state"]="passed"
except BaseException as error:
 steps.receipt.update(state="failed",error=str(error));raise
finally:steps.save();print("Defaultness evidence:",steps.folder,flush=True)
