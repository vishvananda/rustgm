"""CI-only exact2 continuation, with authenticated retained source/function proofs."""
import hashlib,json,os,re,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
SOURCE_SHA='0edad093a1b376c8eafd0589f52eef7cf0985a4c1e963111a0d8a8544759ef19'
CANDIDATE='7a258d6c1d22e5a221887765f31f5ac5579bd2ac'
PROOF_SHA='2595968e5ba3d69c778c332934934ee454b16e4e63a01045173f8ab5bb706802'
OLD_SOURCE_SHA='08734eadbd645347834a3bb5bb378fadea5259d94e36ad769e4ecd989a3cfa32'
OUTPUT_SHA='8182dfb6b3b15fccba77ec3ada6148985c06c15118e0a1368d2206a1f0dee568'
CASES=['sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_bound_selected_work_and_warm_storage', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_keep_failed_attempt_and_peak_costs']
RETAINED=['sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_retain_full_family_and_unused_formals', 'sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_reject_changed_provenance_and_incomplete_formals', 'support::crates::codec::packet_envelope::tests::actual_source_functions_cross_public_codec_and_fresh_client_proof', 'resolve::body_origin::catalog::freezing_tests::clone_drop_and_unique_access_never_reopen_frozen_installation']
TEST='dev/tests/invariants/prepared_callables.rs'

def hashed(data): return hashlib.sha256(data).hexdigest()
def worker(deadline,folder):
    with exclusive(ROOT):
        check_space(ROOT,minimum_reserve())
        steps=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=folder,prune_under=[]))
        try:
            proof_raw=(ROOT/'.github/template-retained-receipt.json').read_bytes()
            if hashed(proof_raw)!=PROOF_SHA: raise ValueError('prior receipt identity changed')
            proof=json.loads(proof_raw)
            sources=fingerprint(ROOT)
            if hashed(json.dumps(sources,sort_keys=True,separators=(',',':')).encode())!=SOURCE_SHA:
                raise ValueError('current frozen source changed')
            if set(sources)!=set(proof['sources']) or [p for p in sources if sources[p]!=proof['sources'][p]]!=[TEST]:
                raise ValueError('change exceeds the single corrected test owner')
            if any(row['status']!=0 for row in proof['steps'][:7]) or proof['steps'][7]['name']!='lib' or proof['steps'][7]['status']!=101:
                raise ValueError('prior static/build/test evidence differs')
            old=(ROOT/'.github/template-retained-source.rs').read_bytes()
            if hashed(old)!=OLD_SOURCE_SHA or OLD_SOURCE_SHA!=proof['sources'][TEST]:
                raise ValueError('prior test source does not bind static proof')
            output=(ROOT/'.github/template-retained-lib.stdout').read_bytes()
            if hashed(output)!=OUTPUT_SHA or any(('test '+case+' ... ok') not in output.decode() for case in RETAINED):
                raise ValueError('four retained test results differ')
            # Actual Rust syntax nodes, not line edits or rendered diagnostics,
            # identify every unchanged helper and test in the one changed owner.
            from rust_audit import PARSER
            def functions(data):
                tree=PARSER.parse(data).root_node
                if tree.has_error: raise ValueError('test source syntax cannot be authenticated')
                return {node.child_by_field_name('name').text.decode():data[node.start_byte:node.end_byte]
                        for node in tree.named_children if node.type=='function_item'}
            new=(ROOT/TEST).read_bytes()
            def declarations(data):
                return [(node.type,data[node.start_byte:node.end_byte]) for node in PARSER.parse(data).root_node.named_children
                        if node.type not in {'function_item','line_comment','block_comment'}]
            if declarations(old)!=declarations(new): raise ValueError('nonfunction test owner declarations changed')
            previous,current=functions(old),functions(new)
            changed={name for name in current if previous.get(name)!=current[name]}
            allowed={case.rsplit('::',1)[1] for case in CASES}|{'blocked_body_dependency'}
            if changed!=allowed or set(previous)-set(current):
                raise ValueError('test/helper edits exceed the exact two corrected controls')
            steps.receipt.update(sources=sources,groups={'lib':CASES},candidate=CANDIDATE,
                retained={'run':40219671209,'receipt_sha256':PROOF_SHA,'passed':RETAINED,
                          'steps':proof['steps'][:4],'old_test_source_sha256':OLD_SOURCE_SHA,
                          'function_changes':sorted(changed),
                          'attribution':'Prior four passes and unchanged metadata retained. Changed source file formatting/global duplicate audit/private Clippy invalidated and rerun.'})
            steps.environment['RUST_TEST_NOCAPTURE']='1';steps.save()
            steps.run('format-changed',['rustfmt','--edition','2024','--check',TEST])
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
            if selected!=CASES: raise ValueError('actual nonignored inventory differs')
            steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':hashed(binary.read_bytes())}});steps.save()
            output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev')
            if re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)!=[('2','0','0')]:
                raise ValueError('actual exact two count differs')
            if fingerprint(ROOT)!=sources: raise ValueError('source changed during execution')
            steps.receipt['state']='passed';print('Passed exact2; prior four actual passes retained by source/AST proof.',flush=True)
            return 0
        except BaseException as error:
            steps.receipt.update(state='failed',error=str(error));raise
        finally:
            steps.save();print('Continuation evidence: '+str(folder),flush=True)

def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        return worker(float(sys.argv[2]), Path(sys.argv[3]))
    started = time.monotonic()
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", "0", "--",
               sys.executable, ".github/focused_refs.py", "--worker", str(started + 187), str(folder)]
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
            "command": command, "retained_run": 40219671209}, indent=2) + "\n")
        print(f"Missing-only total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
