from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys,time
started=time.monotonic()
source='41b60e3125c2a4e4f31eb8f180f7f2ed1041fc02'
folder=Path("obj/parallel/all-attribute-selected-output-v1")
folder.mkdir(parents=True,exist_ok=True)
with (folder/"backend-setup.stdout").open("wb") as out,(folder/"backend-setup.stderr").open("wb") as err:
 subprocess.run([sys.executable,"scripts/storage.py","run","--no-prune","--",sys.executable,"scripts/setup_backend.py"],stdout=out,stderr=err,check=True,timeout=60)
remaining=200-(time.monotonic()-started)
command=[sys.executable,"scripts/fastcheck.py","--seconds",str(remaining),"--ignored","--prune-under","obj/parallel/pa31/source-cli-contracts","--prune-under","obj/parallel/pa29-source-client-staging/lib007/packet-cli",*['crate_linking::packet_libraries::cli_source_artifact_to_fresh_native_consumer_returns_seven', 'crate_linking::versioned_clients::cli_function_contracts_reach_fresh_native_consumers_returning_seven']]
subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True,timeout=remaining)
cli=Path("obj/rust/dev-fast/rustgm")
shutil.copyfile(cli,folder/"rustgm")
for root in [Path("obj/parallel/pa31/source-cli-contracts"),Path("obj/parallel/pa29-source-client-staging/lib007/packet-cli")]:
 if root.exists():shutil.copytree(root,folder/root.name)
(folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest(),"case_contract_sha256":hashlib.sha256(Path("pa31/tests/spec/source-attribute-transport/cli-contract.json").read_bytes()).hexdigest(),"seconds":time.monotonic()-started,"selectors":['crate_linking::packet_libraries::cli_source_artifact_to_fresh_native_consumer_returns_seven', 'crate_linking::versioned_clients::cli_function_contracts_reach_fresh_native_consumers_returning_seven'],"native":True,"backend_local":json.loads(Path("backend.local.json").read_text())},indent=2)+"\n")
