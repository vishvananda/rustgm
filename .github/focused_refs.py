from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys,time
started=time.monotonic()
source='41b60e3125c2a4e4f31eb8f180f7f2ed1041fc02'
folder=Path("obj/parallel/all-attribute-selected-output-v1")
folder.mkdir(parents=True,exist_ok=True)
command=[sys.executable,"scripts/storage.py","run","--no-prune","--",sys.executable,"-m","unittest","discover","-s","scripts/tests","-p","test_source_attributes.py","-v"]
with (folder/"python.stdout").open("wb") as out,(folder/"python.stderr").open("wb") as err:
 subprocess.run(command,stdout=out,stderr=err,check=True,timeout=15)
remaining=200-(time.monotonic()-started)
command=[sys.executable,"scripts/fastcheck.py","--seconds",str(remaining),*['crate_linking::packet_libraries::cli_source_client_flags_reject_missing_identity_and_conflicting_provider_mode', 'crate_linking::versioned_clients::cli_function_contracts_reach_fresh_consumers_with_exact_diagnostics']]
subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True,timeout=remaining)
cli=Path("obj/rust/dev-fast/rustgm")
shutil.copyfile(cli,folder/"rustgm")
for path in Path("obj/parallel/pa31/source-cli-contracts").glob("*"):
 if path.is_dir():shutil.copytree(path,folder/path.name)
 elif path.is_file():shutil.copyfile(path,folder/path.name)
(folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest(),"case_contract_sha256":hashlib.sha256(Path("pa31/tests/spec/source-attribute-transport/cli-contract.json").read_bytes()).hexdigest(),"seconds":time.monotonic()-started,"selectors":['crate_linking::packet_libraries::cli_source_client_flags_reject_missing_identity_and_conflicting_provider_mode', 'crate_linking::versioned_clients::cli_function_contracts_reach_fresh_consumers_with_exact_diagnostics'],"native":False},indent=2)+"\n")
