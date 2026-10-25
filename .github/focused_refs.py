"""CI-only exact six private Environment controls; one direct 190s budget."""
import hashlib,json,os,re,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
SOURCE='b380e3c35dfe8d23c125b9a694e04ebc536761b1'
SOURCE_SHA='c2222bbf5dd2540f8e35c49d40f553fe0fe0d121a9b9d1ad3213ae5581c70dce'
COUNT=2606
CASES=['sema::crates::typed_headers::unpaid_provided::template_demands::assumption_environments::tests::private_defining_environments_use_real_zero_domain_assumptions', 'sema::crates::typed_headers::unpaid_provided::template_demands::assumption_environments::tests::private_defining_environments_reject_changed_owners_and_failed_budgets', 'sema::crates::typed_headers::unpaid_provided::template_demands::assumption_environments::tests::private_defining_environments_bound_selected_and_warm_work', 'sema::crates::typed_headers::unpaid_provided::template_demands::assumption_environments::tests::private_trait_input_costs_cover_real_lazy_growth_and_capacity_hits', 'sema::function_context::assumption_environment::tests::defining_environment_ordinary_path_keeps_the_actual_declaration_arc', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof']

def worker(deadline,folder):
    with exclusive(ROOT):
        check_space(ROOT,minimum_reserve())
        steps=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]))
        try:
            sources=fingerprint(ROOT)
            digest=hashlib.sha256(json.dumps(sources,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            if len(sources)!=COUNT or digest!=SOURCE_SHA: raise ValueError('frozen Environment source fingerprint changed')
            steps.receipt.update(sources=sources,groups={'lib':CASES},candidate=SOURCE)
            steps.environment['RUST_TEST_NOCAPTURE']='1';steps.save()
            steps.run('assignment-metadata',[sys.executable,'scripts/assignment_metadata.py'])
            steps.run('format',['cargo','fmt','--manifest-path','dev/Cargo.toml','--all','--','--check'])
            steps.run('architecture',[ROOT/'obj/audit-venv/bin/python','scripts/rust_audit.py','--root','.',
                                      '--cache','obj/fast-audit.json'])
            common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
            steps.run('clippy-private',['cargo','clippy',*common,'--lib','--','--test','-D','warnings'],
                      overrides={'CARGO_TARGET_DIR':str(ROOT/'obj/rust/private-lint')})
            built=steps.run('build',['cargo','test',*common,'--lib','--no-run','--message-format=json'])
            binary=artifacts(built,{'lib':CASES})['lib']
            listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev')
            ignored=set(steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev').splitlines())
            selected=inventory('\n'.join(row for row in listed.splitlines() if row not in ignored),'lib',CASES)
            if selected!=CASES: raise ValueError('actual nonignored inventory differs from exact six')
            steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}});steps.save()
            output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev')
            if re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)!=[('6','0','0')]:
                raise ValueError('actual exact six count differs')
            if fingerprint(ROOT)!=sources: raise ValueError('source changed during execution')
            steps.receipt['state']='passed';print('Passed actual exact6 Environment controls and required static checks.',flush=True)
            return 0
        except BaseException as error:
            steps.receipt.update(state='failed',error=str(error));raise
        finally:
            steps.save();print('Environment evidence: '+str(folder),flush=True)

def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        return worker(float(sys.argv[2]), Path(sys.argv[3]))
    started = time.monotonic()
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", "0", "--",
               str(ROOT / "obj/audit-venv/bin/python"), ".github/focused_refs.py", "--worker", str(started + 187), str(folder)]
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    status = 1
    try:
        status = process.wait(timeout=max(0.01, started + 188 - time.monotonic()))
        return status
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=max(0.01, started + 189 - time.monotonic()))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=max(0.01, started + 190 - time.monotonic()))
        status = 124
        return status
    finally:
        if status:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        folder.parent.mkdir(parents=True, exist_ok=True)
        folder.with_suffix(".launcher.json").write_text(json.dumps({
            "state": "passed" if status == 0 else "failed", "status": status,
            "seconds": time.monotonic() - started, "absolute_budget_seconds": 190,
            "command": command, "candidate": SOURCE}, indent=2) + "\n")
        print(f"Environment gate total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
