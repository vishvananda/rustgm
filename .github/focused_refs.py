"""CI-only exact one raw pre-adapter ordinary-profile capture; one direct 190s budget."""
import hashlib,json,os,re,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory
from storage import exclusive,check_space,minimum_reserve
SOURCE='a121f5f1b9a73cf547f39e4a6dec97b4dfde10e7'
SOURCE_SHA='5261916d8fcbca75037954daf51728aaa98c5533dba76927c74d36316ad07785'
COUNT=2606
CASES=['support::crates::codec::packet_envelope::tests::ordinary_source_profiles_capture_plain_contract_bytes_before_adapter']

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
            old_bytes=(ROOT/'.github/baseline_static.json').read_bytes()
            if hashlib.sha256(old_bytes).hexdigest()!='2a2d043d4c9656e3aca4aef9b074297ff0079c501974978998ba712642459c86': raise ValueError('prior metadata receipt changed')
            prior=json.loads(old_bytes)
            if prior['steps'][0]['name']!='assignment-metadata' or prior['steps'][0]['status']!=0: raise ValueError('prior metadata did not pass')
            changed={path for path in sources if sources[path]!=prior['sources'].get(path)}
            if changed!={'dev/tests/invariants/packet_consumers.rs'} or sources.keys()!=prior['sources'].keys(): raise ValueError('metadata reuse source delta widened')
            steps.receipt['retained_steps']={'run':40228886283,'receipt_sha256':hashlib.sha256(old_bytes).hexdigest(),'steps':['assignment-metadata'],'prior_format_architecture':'passed for c425, rerun here because actual test changed'};steps.save()
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
            if selected!=CASES: raise ValueError('actual nonignored inventory differs from exact one')
            steps.receipt.update(selected={'lib':selected},binaries={'lib':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}});steps.save()
            output=steps.run('lib',[binary,'--exact',*selected],cwd=ROOT/'dev')
            if re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)!=[('1','0','0')]:
                raise ValueError('actual exact one count differs')
            capture=ROOT/'obj/parallel/all-attribute-selected-output-v1/empty-contract-baseline'
            if hashlib.sha256((capture/'provider.rs').read_bytes()).hexdigest()!='5a121f0b6341e1f71d8a1188bbe3d460cb94a8037fa060799f53ce88537d2f80': raise ValueError('actual capture source changed')
            files={str(path.relative_to(capture)):{'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size} for path in sorted(capture.iterdir()) if path.is_file()}
            if not all(name in files for name in ['v0.rgcrate','v1.rgcrate','v0.rows.txt','v1.rows.txt','v0.binding.txt','v1.binding.txt','provider.rs']): raise ValueError('actual first-encoding capture incomplete')
            binding={'candidate':SOURCE,'source_fingerprints':sources,'source_sha256':SOURCE_SHA,'api':'build_source_library_profile -> crates::encode(first actual output)','binary':steps.receipt['binaries']['lib'],'capture':files,'profiles':['LegacyFunctions v0','FunctionContracts v1'],'producer_config':'test_source::source_provider(source,plain_contract_pair); Config::default; actual SourceCore identity minicore/plain_contract_pair; Rust2024; source-provider.rs'}
            (capture/'source-binding.json').write_text(json.dumps(binding,indent=2)+'\n')
            steps.receipt['baseline_capture']=files;steps.save()
            if fingerprint(ROOT)!=sources: raise ValueError('source changed during execution')
            steps.receipt['state']='passed';print('Passed actual exact1 raw ordinary-profile capture controls and required static checks.',flush=True)
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
