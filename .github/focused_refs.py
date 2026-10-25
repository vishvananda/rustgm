from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys,time
source='19a961b805cf06e0084226338fb682651b997c5f'
selectors=['resolve::expansion::unnamed_visibility::tests::module_block_macro_and_inactive_items_preserve_real_namespace_identities', 'resolve::expansion::unnamed_visibility::tests::evaluation::unused_initializers_are_checked_and_public_placeholders_do_not_export', 'resolve::expansion::unnamed_visibility::tests::levels::visibility_levels_origins_and_mixed_failures_preserve_exact_warning_contracts', 'resolve::expansion::unnamed_visibility::tests::proofs::visibility_source_failures_keep_primary_errors_and_policy_costs_are_measured', 'resolve::package::diagnostic_warnings::tests::retention_limit_keeps_the_original_failure_and_one_bounded_note', 'resolve::package::diagnostic_warnings::tests::unavailable_warning_origin_note_is_bounded_and_survives_later_budget_merges']
folder=Path("obj/parallel/all-attribute-selected-output-v1")
folder.mkdir(parents=True,exist_ok=True)
started=time.monotonic();status=1
try:
 command=[sys.executable,"scripts/fastcheck.py","--seconds","200"]
 for name in selectors:command += ["--lib-case",name]
 subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True,timeout=200)
 status=0
finally:
 cli=Path("obj/rust/dev-fast/rustgm")
 if cli.exists():shutil.copyfile(cli,folder/"rustgm")
 (folder/"binding.json").write_text(json.dumps({"product_source":source,"ci_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"cli_sha256":hashlib.sha256(cli.read_bytes()).hexdigest() if cli.exists() else None,"seconds":time.monotonic()-started,"state":"passed" if status==0 else "failed","selectors":selectors,"pending_refs":"8activePA6/PA8cases; no reference writes authorized in helper; integrator owns selected generation"},indent=2)+"\n")
