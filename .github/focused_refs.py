from pathlib import Path
import hashlib,json,subprocess,time
plan={'state': 'frozen independent predictions before candidate or oracle execution', 'source': '88d9e12ae178fab0fb5cc050a94da85685d449ea', 'toolchain': '1.98.0', 'rustc_sha256': '3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2', 'editions': ['2021', '2024'], 'target': 'x86_64-unknown-linux-gnu', 'flags': ['-Cpanic=abort', '--emit=metadata', '--error-format=json'], 'original_frontier': {'run': 40215218089, 'file': 55, 'path': 'library/core/src/mem/alignment.rs', 'sha256': '4eb6651741fc8d8d84ce274219cc72399949aa9eba2a4592a3f90d9293d1b587', 'opened_sources_sha256': '358ca4b5bf120379c95f4f2a481fa61b617cac52179bfef1837aaa59d6e921c8', 'status': 1, 'diagnostic': "ERROR: file55:981: duplicate value name '_'", 'no_artifact': True}, 'time_budget_seconds': 150, 'cases': [{'name': 'literal-many-calls', 'path': 'literal-many-calls.rs', 'sha256': '3e885c9aa646479eaef7b80e51ba0093e8634fbc8e83417ee793a3fb5c77307c', 'status': 0, 'warning_codes': ['unused_visibilities', 'unused_visibilities'], 'required_error': None}, {'name': 'unexecuted-macro', 'path': 'unexecuted-macro.rs', 'sha256': '8211e885e4f7a46554e0c4299186ec015b1c97da78745902a10270699909cb97', 'status': 0, 'warning_codes': [], 'required_error': None}, {'name': 'disabled-owner', 'path': 'disabled-owner.rs', 'sha256': '2bdbe107c411a3aa50eb3ea3b3a9ab04ad3b845cf412670ac9a8dc4e304bc149', 'status': 0, 'warning_codes': [], 'required_error': None}, {'name': 'unresolved-initializer', 'path': 'unresolved-initializer.rs', 'sha256': '9d6205b8d05c2a57f5d26cb177a6386b831138e4e957fb6617eca00ab755e2c4', 'status': 1, 'warning_codes': [], 'required_error': 'E0425'}, {'name': 'public-library', 'path': 'public-library.rs', 'sha256': '3e160e345ef0edc2f31a877f998318302a1b3f93d53ae7cb1b09c2399477f106', 'status': 0, 'warning_codes': ['unused_visibilities'], 'required_error': None}], 'observations_planned': 10, 'scope': 'ONLY5newopponents; no38or28repeat; metadata-only; beforecandidateexecution', 'prior_observation': {'run': 40216397961, 'observations': 38, 'infrastructure_failures': 0, 'hypothesis_disagreements': 4, 'artifact_sha256': 'cc5bae8a2e719eaa534b7612cbb9879fe4fdf987219caa2c7ec6cb6452dc5ede'}}
sources={'literal-many-calls.rs': '#![allow(dead_code,unused_imports,unused_macros)]\nmacro_rules! make {()=>{pub const _:u8=1;}}\nmake!();make!();fn main(){}\n', 'unexecuted-macro.rs': '#![allow(dead_code,unused_imports,unused_macros)]\n#![deny(unused_visibilities)]\nmacro_rules! unused {()=>{pub const _:u8=missing;}}\nfn main(){}\n', 'disabled-owner.rs': '#![allow(dead_code,unused_imports,unused_macros)]\n#![deny(unused_visibilities)]\n#[cfg(any())]mod disabled{pub const _:u8=missing;}\nfn main(){}\n', 'unresolved-initializer.rs': '#![allow(dead_code,unused_imports,unused_macros)]\nconst _:u8=missing;fn main(){}\n', 'public-library.rs': '#![allow(dead_code,unused_imports,unused_macros)]\npub const _:u8=1;pub const NAMED:u8=7;pub fn value()->u8{NAMED}fn main(){}\n'}
folder=Path("obj/parallel/all-attribute-selected-output-v1")
folder.mkdir(parents=True,exist_ok=True)
(folder/"oracle-plan.json").write_text(json.dumps(plan,indent=2)+"\n")
source_dir=folder/"sources";source_dir.mkdir()
for name,source in sources.items():(source_dir/name).write_text(source)
started=time.monotonic();deadline=started+150
path=subprocess.check_output(["rustup","which","--toolchain",plan["toolchain"],"rustc"],text=True,timeout=10).strip()
assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==plan["rustc_sha256"]
rows=[];disagreements=[]
for case in plan["cases"]:
 source=source_dir/case["path"]
 assert hashlib.sha256(source.read_bytes()).hexdigest()==case["sha256"]
 for edition in plan["editions"]:
  name=case["name"]+"-"+edition;output=folder/(name+".rmeta")
  command=[path,"--edition="+edition,"--target="+plan["target"],"--crate-name","probe",*plan["flags"],str(source),"-o",str(output)]
  before=time.monotonic()
  remaining=deadline-before
  assert remaining>0,"oracle deadline"
  result=subprocess.run(command,capture_output=True,timeout=min(5,remaining))
  (folder/(name+".stdout")).write_bytes(result.stdout);(folder/(name+".stderr")).write_bytes(result.stderr)
  assert result.returncode in (0,1),name
  assert not result.stdout,name
  diagnostics=[json.loads(line) for line in result.stderr.splitlines()]
  codes=lambda level:[x["code"]["code"] for x in diagnostics if x.get("level")==level and x.get("code")]
  errors=codes("error");warnings=codes("warning")
  observed={"name":name,"source_sha256":case["sha256"],"edition":edition,"command":command,"status":result.returncode,"seconds":time.monotonic()-before,"warning_codes":warnings,"error_codes":errors,"stderr_sha256":hashlib.sha256(result.stderr).hexdigest(),"stdout_sha256":hashlib.sha256(result.stdout).hexdigest(),"expected_status":case["status"],"expected_warning_codes":case["warning_codes"],"required_error":case["required_error"]}
  if result.returncode!=case["status"] or warnings!=case["warning_codes"] or case["required_error"] and case["required_error"] not in errors:disagreements.append(observed)
  rows.append(observed)
  if output.exists():output.unlink()
  (folder/"results.json").write_text(json.dumps({"state":"running","rows":rows,"disagreements":disagreements},indent=2)+"\n")
record={"state":"completed","infrastructure_failures":0,"observations":len(rows),"seconds":time.monotonic()-started,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"rustc_sha256":plan["rustc_sha256"],"rows":rows,"disagreements":disagreements}
(folder/"results.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"observations":len(rows),"seconds":record["seconds"],"disagreements":[r["name"] for r in disagreements],"infrastructure_failures":0}))
