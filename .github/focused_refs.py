import hashlib,json,os,pathlib,shutil,subprocess,sys
selected=['literal::tests::wide_decimals_match_frozen_ieee_boundaries_without_double_rounding', 'literal::tests::numeric_shape_preserves_known_width_and_rejects_real_invalid_suffixes', 'literal::decimal_rounding::tests::significant_prefix_scratch_stops_growing_for_long_zero_and_sticky_tails', 'resolve::float_features::tests::active_wide_features_and_inactive_token_opponents_use_real_authority', 'resolve::float_features::tests::wide_values_keep_exact_shapes_and_unsupported_later_operations_closed', 'types::wide_float_tests::two_new_primitives_preserve_existing_ids_and_enclosing_hot_layouts', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof', 'lower::transported_source::publication_tests::all_selected_function_and_trait_contracts_reject_before_payload_publication']
if "--bind" not in sys.argv:
 command=[sys.executable,"scripts/fastcheck.py","--seconds","200"]
 for name in selected:command += ["--lib-case",name]
 subprocess.run(command,env={**os.environ,"RUST_TEST_NOCAPTURE":"1"},check=True)
 subprocess.run(["timeout","30s",sys.executable,"scripts/storage.py","run","--no-prune","--",sys.executable,__file__,"--bind"],check=True)
 sys.exit(0)
root=pathlib.Path.cwd();out=root/"obj/parallel/all-attribute-selected-output-v1";out.mkdir(parents=True,exist_ok=True)
candidate=out/"candidate";candidate.mkdir();(candidate/".keep-artifacts").touch()
app=candidate/"rustgm";shutil.copy2(root/"obj/rust/dev-fast/rustgm",app)
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
record={"source_commit":subprocess.check_output(["git","rev-parse","HEAD"]).decode().strip(),"candidate_sha256":sha(app),"compiler_source_sha256":{str(p):sha(p) for p in sorted(pathlib.Path("dev").rglob("*.rs"))},"profile":"dev-fast","rustc":subprocess.check_output(["rustc","--version"]).decode().strip(),"selected":selected,"qualification":"all selected controls passed before this binding"}
(out/"binding.json").write_text(json.dumps(record,indent=2)+"\n")
print("Qualified candidate CLI",record["candidate_sha256"])
