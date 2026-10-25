from pathlib import Path
from types import SimpleNamespace
import hashlib,json,re,subprocess,sys,time
sys.path.insert(0,str(Path.cwd()/"scripts"))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
root=Path.cwd();started=time.monotonic()
raw=Path(".github/retained_static.json").read_bytes()
assert hashlib.sha256(raw).hexdigest()=='dcd70fb25a58eacbb11429a0891bb2e43afa6cba4c83edd162d935e7913fcce1'
prior=json.loads(raw)
for name in ["assignment-metadata","format","architecture","clippy-private"]:
 assert next(s for s in prior["steps"] if s["name"]==name)["status"]==0,name
current=fingerprint(root);assert current==prior["sources"],"source changed since retained static gate"
cli=root/"obj/rust/dev-fast/rustgm"
assert hashlib.sha256(cli.read_bytes()).hexdigest()=='dea04dfbbfc7bd36f8c7f0f1136632cfe4d634ff941bab3c2834475ae9063c38',"compiled CLI changed"
tool=subprocess.check_output(["rustup","which","--toolchain","1.98.0","rustc"],text=True,timeout=5).strip()
assert hashlib.sha256(Path(tool).read_bytes()).hexdigest()=="3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2"
args=SimpleNamespace(evidence_root=root/("obj/qualification/fast/"+str(time.time_ns())+"-missing"),deadline=started+165,prune_under=[])
steps=Steps(root,args);selectors=prior["groups"]["lib"]
steps.receipt.update(state="running",sources=current,groups={"lib":selectors},retained_static={"run":40221382616,"receipt_sha256":'dcd70fb25a58eacbb11429a0891bb2e43afa6cba4c83edd162d935e7913fcce1',"profile":"dev-fast","source":"19a961b805cf06e0084226338fb682651b997c5f","compiled_cli_sha256":'dea04dfbbfc7bd36f8c7f0f1136632cfe4d634ff941bab3c2834475ae9063c38'})
steps.save()
try:
 command=["cargo","test","--manifest-path","dev/Cargo.toml","--locked","--profile","dev-fast","--lib","--no-run","--message-format=json"]
 assert next(s for s in prior["steps"] if s["name"]=="build")["command"]==command
 built=steps.run("build",command);binary=artifacts(built,{"lib":selectors})["lib"]
 listed=steps.run("lib-inventory",[binary,"--list"],cwd=root/"dev")
 ignored=steps.run("lib-ignored",[binary,"--ignored","--list"],cwd=root/"dev")
 ignored=set(ignored.splitlines());listed="\n".join(line for line in listed.splitlines() if line not in ignored)
 selected=inventory(listed,"lib",selectors);assert len(selected)==6
 steps.receipt.update(selected={"lib":selected},binaries={"lib":{"path":str(binary),"sha256":hashlib.sha256(binary.read_bytes()).hexdigest()}});steps.save()
 output=steps.run("lib",[binary,"--exact",*selected],cwd=root/"dev",overrides={"RUST_TEST_NOCAPTURE":"1"})
 counts=re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",output)
 assert counts==[("6","0","0")],counts
 assert fingerprint(root)==current,"source changed during missing-only completion"
 steps.receipt["state"]="passed";print("Passed all6 exact missing controls; retained static proof authenticated.")
except BaseException as error:
 steps.receipt.update(state="failed",error=str(error));raise
finally:
 steps.save();print("Missing-only evidence:",steps.folder,flush=True)
