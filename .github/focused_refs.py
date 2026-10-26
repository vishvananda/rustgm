"""Exact six shared input kernels and unchanged source controls; static audit overlaps one sequential Rust lane."""
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,os,re,signal,subprocess,sys,time
from pathlib import Path
from types import SimpleNamespace
ROOT=Path.cwd()
sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def worker(started):
    freeze=json.loads((ROOT/'.github/inherited_numbering_freeze.json').read_text())
    sources=fingerprint(ROOT)
    if sources!=freeze['sources']:raise ValueError('frozen source changed')
    out=ROOT/f'obj/qualification/fast/{time.time_ns()}-shared-input-kernels'
    deadline=started+185
    steps=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=out,prune_under=[]))
    cases=freeze['selected']
    cli=ROOT/'obj/rust/dev-fast/rustgm';cli_sha=sha(cli)
    steps.receipt.update(sources=sources,candidate=freeze['source_commit'],groups={'lib':cases},compiler_sha256=cli_sha,baseline=freeze['baseline'])
    steps.save()
    def independent(name,commands):
        branch=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=out/name,prune_under=[]))
        branch.receipt.update(sources=sources)
        try:
            findings=[]
            for step,command,overrides in commands:
                try:branch.run(step,command,overrides=overrides)
                except Exception as error:findings.append(str(error))
            if findings:raise ValueError('; '.join(findings))
            branch.receipt['state']='passed'
        except BaseException as error:
            branch.receipt.update(state='failed',error=str(error));raise
        finally:branch.save()
    try:
        retained=json.loads((ROOT/'.github/shared_kernel_retained.json').read_text())
        previous=retained['sources']
        changed={p for p in sources.keys()|previous.keys() if sources.get(p)!=previous.get(p)}
        if changed!=set(retained['changed']):raise ValueError('retained source changes escaped exact test-only delta')
        files=retained['old_files']
        for path,data in files.items():
            if hashlib.sha256(data.encode()).hexdigest()!=previous[path]:raise ValueError('retained old source binding changed')
        path='dev/src/sema/crates/typed_headers.rs'
        if files[path].replace('../../../tests/invariants/environment_origins.rs','../../../tests/invariants/origin_relationships.rs')!=(ROOT/path).read_text():raise ValueError('cfg(test) path-only proof changed')
        path='dev/tests/invariants/remapped_inputs.rs'
        if files[path].replace('let actual = crate::unpaid_cases::trait_extended(inherited_tests::HOLDER)?;','let mut actual = crate::unpaid_cases::trait_extended(inherited_tests::HOLDER)?;')!=(ROOT/path).read_text():raise ValueError('actual owner mutability proof changed')
        if files['dev/tests/invariants/environment_origins.rs']!=(ROOT/'dev/tests/invariants/origin_relationships.rs').read_text():raise ValueError('renamed origin test changed')
        if retained['receipt']['sources']!=previous or retained['lint']['sources']!=previous or retained['receipt']['candidate']!=retained['source_commit']:raise ValueError('retained actual receipt source mismatch')
        if next(x for x in retained['receipt']['steps'] if x['name']=='metadata')['status']!=0 or next(x for x in retained['lint']['steps'] if x['name']=='clippy-production')['status']!=0:raise ValueError('retained successful static step missing')
        if cli_sha!=retained['compiler_sha256'] or cli_sha!=retained['receipt']['compiler_sha256']:raise ValueError('actual mandatory CLI does not match retained production source')
        steps.receipt['retained_static']=freeze['retained_static'];steps.save()
        steps.run('format',['cargo','fmt','--manifest-path','dev/Cargo.toml','--all','--','--check'])
        common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
        audit=[('architecture',[ROOT/'obj/audit-venv/bin/python','scripts/rust_audit.py','--root','.','--cache','obj/fast-audit.json'],{})]
        lint=[('clippy-private',['cargo','clippy',*common,'--lib','--','--test','-D','warnings'],{'CARGO_TARGET_DIR':str(ROOT/'obj/rust/private-lint')})]
        failures=[];start=time.monotonic()
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures=[executor.submit(independent,'audit',audit),executor.submit(independent,'lint',lint)]
            for future in futures:
                try:future.result()
                except BaseException as error:failures.append(str(error))
        steps.receipt.update(parallel_static_seconds=time.monotonic()-start,independent_static_receipts={name:sha(out/name/'receipt.json') for name in ('audit','lint')});steps.save()
        if failures:raise ValueError('; '.join(failures))
        built=steps.run('build',['cargo','test',*common,'--lib','--no-run','--message-format=json'])
        binary=artifacts(built,{'lib':cases})['lib']
        listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev')
        ignored=set(steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev').splitlines())
        actual=inventory('\n'.join(row for row in listed.splitlines() if row not in ignored),'lib',cases)
        if actual!=cases or len(set(actual))!=6:raise ValueError('exact six inventory changed')
        steps.receipt.update(selected={'lib':actual},binaries={'lib':{'sha256':sha(binary)}});steps.save()
        output=steps.run('lib',[binary,'--exact',*actual],cwd=ROOT/'dev',overrides={'RUST_TEST_NOCAPTURE':'1'})
        if re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)!=[('6','0','0')]:raise ValueError('actual six count changed')
        capture=ROOT/'obj/parallel/all-attribute-selected-output-v1/empty-contract-baseline'
        baseline=freeze['baseline']['files']
        for name in baseline:
            path=capture/name
            if path.stat().st_size!=baseline[name]['bytes'] or sha(path)!=baseline[name]['sha256']:raise ValueError('actual first encoding differs from pre-adapter raw baseline: '+name)
        captured={path.name:{'sha256':sha(path),'bytes':path.stat().st_size} for path in capture.iterdir() if path.is_file() and not path.name.startswith('.')}
        binding={'candidate':freeze['source_commit'],'sources':sources,'source_sha256':freeze['source_sha256'],'binary':steps.receipt['binaries']['lib'],'compiler_sha256':cli_sha,'baseline_run':freeze['baseline']['run'],'comparison':'first actual encode v0/v1 raw artifacts and source/config bindings exactly equal authenticated pre-adapter capture; rows retained separately','captured':captured}
        (capture/'source-binding.json').write_text(json.dumps(binding,indent=2)+'\n')
        if fingerprint(ROOT)!=sources or sha(cli)!=cli_sha:raise ValueError('source or actual CLI changed')
        for path,digest in freeze['unchanged_public_guards'].items():
            if sha(ROOT/path)!=digest:raise ValueError('public guard changed')
        steps.receipt.update(state='passed',actual_raw_byte_comparison=captured,seconds=time.monotonic()-started)
    except BaseException as error:
        steps.receipt.update(state='failed',error=str(error),seconds=time.monotonic()-started);raise
    finally:
        steps.save();print(json.dumps({key:value for key,value in steps.receipt.items() if key!='sources'},indent=2),flush=True)

def main():
    if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
    start=time.monotonic()
    process=subprocess.Popen([sys.executable,'scripts/storage.py','run','--no-prune','--',str(ROOT/'obj/audit-venv/bin/python'),__file__,'--worker',str(start)],start_new_session=True)
    status=1
    try:
        status=process.wait(timeout=max(0.01,start+188-time.monotonic()));return status
    except subprocess.TimeoutExpired:
        os.killpg(process.pid,signal.SIGTERM)
        try:process.wait(timeout=max(0.01,start+189-time.monotonic()))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=max(0.01,start+190-time.monotonic()))
        status=124;return status
    finally:
        if status:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
        print(f'Shared input kernel gate total: {time.monotonic()-start:.2f}s status={status}',flush=True)

if __name__=='__main__':sys.exit(main())
