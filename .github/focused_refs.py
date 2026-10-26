"""One corrected positive; five actual passes retained by exact source/AST/CLI proof."""
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,os,re,signal,subprocess,sys,time,zipfile
from pathlib import Path
from types import SimpleNamespace
ROOT=Path.cwd();sys.path.insert(0,str(ROOT/'scripts'))
from fastcheck_run import Steps,fingerprint
from maintainer_batch import artifacts,inventory

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def worker(started):
    freeze=json.loads((ROOT/'.github/collected-freeze.json').read_text())
    out=ROOT/f'obj/qualification/fast/{time.time_ns()}-collected-one'
    deadline=started+185
    steps=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=out,prune_under=[]))
    try:
        sources=fingerprint(ROOT)
        if sources!=freeze['sources']:raise ValueError('current frozen source changed')
        # The mandatory workflow really built this CLI. Authenticate it BEFORE
        # accepting any retained production/static or actual test evidence.
        cli=ROOT/'obj/rust/dev-fast/rustgm';cli_sha=sha(cli)
        if cli_sha!=freeze['expected_cli_sha256']:raise ValueError('actual prebuilt production CLI differs')
        prior={}
        for name,digest in freeze['proof_files'].items():
            path=ROOT/'.github'/('collected-'+name)
            if sha(path)!=digest:raise ValueError('retained evidence changed: '+name)
            if name.endswith('.json'):prior[name]=json.loads(path.read_text())
        proof=prior['retained-receipt.json'];lint=prior['retained-lint.json'];audit=prior['retained-audit.json']
        if proof['compiler_sha256']!=cli_sha or proof['selected']['lib']!=freeze['selected']+freeze['retained']:
            raise ValueError('prior actual compiler/inventory binding changed')
        if any(row['status']!=0 for row in proof['steps'] if row['name']!='lib') or proof['steps'][-1]['status']!=101:
            raise ValueError('prior source steps differ')
        for receipt in (lint,audit):
            if receipt['state']!='passed' or receipt['sources']!=proof['sources'] or any(row['status']!=0 for row in receipt['steps']):
                raise ValueError('retained static receipt differs')
        if set(sources)!=set(proof['sources']) or [p for p in sources if sources[p]!=proof['sources'][p]]!=[freeze['changed_file']]:
            raise ValueError('change exceeds the single test owner')
        old=(ROOT/'.github/collected-old-source.rs').read_bytes()
        if hashlib.sha256(old).hexdigest()!=proof['sources'][freeze['changed_file']]:raise ValueError('prior test source differs')
        output=(ROOT/'.github/collected-retained-lib.stdout').read_text()
        if any('test '+case+' ... ok' not in output for case in freeze['retained']):raise ValueError('retained five actual passes missing')
        if re.findall(r'test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored;',output)!=[('5','1','0')]:raise ValueError('prior actual count differs')
        from rust_audit import PARSER
        def items(data):
            tree=PARSER.parse(data).root_node
            if tree.has_error:raise ValueError('test source cannot be authenticated')
            functions={node.child_by_field_name('name').text.decode():data[node.start_byte:node.end_byte] for node in tree.named_children if node.type=='function_item'}
            declarations=[(node.type,data[node.start_byte:node.end_byte]) for node in tree.named_children if node.type not in {'function_item','line_comment','block_comment'}]
            return functions,declarations
        previous,old_decls=items(old);current,new_decls=items((ROOT/freeze['changed_file']).read_bytes())
        changed={name for name in current if current[name]!=previous.get(name)}
        if old_decls!=new_decls or set(previous)-set(current) or changed!=set(freeze['allowed_functions']):raise ValueError('test AST change exceeds corrected positive/helper')
        steps.receipt.update(sources=sources,candidate=freeze['source_commit'],groups={'lib':freeze['selected']},compiler_sha256=cli_sha,
            retained={'run':freeze['retained_run'],'proof_files':freeze['proof_files'],'source':proof['candidate'],'binaries':proof['binaries'],'passed':freeze['retained'],
                'production_and_metadata':'retained actual passed steps; full production source and actual prebuilt CLI unchanged',
                'functions_changed':sorted(changed),'attribution':freeze['attribution']})
        steps.save()
        def check(name,command,overrides):
            branch=Steps(ROOT,SimpleNamespace(deadline=deadline,evidence_root=out/name,prune_under=[]));branch.receipt.update(sources=sources)
            try:
                branch.run(name,command,overrides=overrides);branch.receipt['state']='passed'
            except BaseException as error:
                branch.receipt.update(state='failed',error=str(error));raise
            finally:branch.save()
        steps.run('format-changed',['rustfmt','--edition','2024','--check',freeze['changed_file']])
        common=['--manifest-path','dev/Cargo.toml','--locked','--profile','dev-fast']
        checks=[('architecture',[ROOT/'obj/audit-venv/bin/python','scripts/rust_audit.py','--root','.','--cache','obj/fast-audit.json'],{}),
                ('clippy-private',['cargo','clippy',*common,'--lib','--','--test','-D','warnings'],{'CARGO_TARGET_DIR':str(ROOT/'obj/rust/private-lint')})]
        failures=[]
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures=[executor.submit(check,*entry) for entry in checks]
            for future in futures:
                try:future.result()
                except BaseException as error:failures.append(str(error))
        steps.receipt['independent_static_receipts']={name:sha(out/name/'receipt.json') for name in ('architecture','clippy-private')};steps.save()
        if failures:raise ValueError('; '.join(failures))
        built=steps.run('build',['cargo','test',*common,'--lib','--no-run','--message-format=json'])
        cases=freeze['selected'];binary=artifacts(built,{'lib':cases})['lib']
        listed=steps.run('lib-inventory',[binary,'--list'],cwd=ROOT/'dev');ignored=set(steps.run('lib-ignored',[binary,'--ignored','--list'],cwd=ROOT/'dev').splitlines())
        actual=inventory('\n'.join(row for row in listed.splitlines() if row not in ignored),'lib',cases)
        if actual!=cases:raise ValueError('actual exact one inventory differs')
        steps.receipt.update(selected={'lib':actual},binaries={'lib':{'sha256':sha(binary)}});steps.save()
        result=steps.run('lib',[binary,'--exact',*actual],cwd=ROOT/'dev',overrides={'RUST_TEST_NOCAPTURE':'1'})
        if re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result)!=[('1','0','0')]:raise ValueError('actual exact positive count differs')
        retained_root=ROOT/'obj/parallel/all-attribute-selected-output-v1/retained-40265276583';retained_root.mkdir(parents=True)
        captured={}
        with zipfile.ZipFile(ROOT/'.github/collected-retained-raw.zip') as archive:
            for name,expected in freeze['baseline']['files'].items():
                data=archive.read('empty-contract-baseline/'+name)
                if len(data)!=expected['bytes'] or hashlib.sha256(data).hexdigest()!=expected['sha256']:raise ValueError('retained actual first bytes differ: '+name)
                (retained_root/name).write_bytes(data);captured[name]=expected
        binding={'current_candidate':freeze['source_commit'],'current_sources':sources,'current_positive':actual,'current_cli_sha256':cli_sha,
                 'retained_source':proof['candidate'],'retained_run':freeze['retained_run'],'retained_compiler_sha256':proof['compiler_sha256'],'retained_binaries':proof['binaries'],
                 'retained_actual_passes':freeze['retained'],'retained_raw7':captured,'note':'Raw7 were actually emitted by retained source348 in run40265276583; no current rerun or reference generation.'}
        (retained_root/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
        if fingerprint(ROOT)!=sources or sha(cli)!=cli_sha:raise ValueError('actual source/CLI changed')
        for path,digest in freeze['unchanged_public_guards'].items():
            if sha(ROOT/path)!=digest:raise ValueError('public guard changed')
        steps.receipt.update(state='passed',seconds=time.monotonic()-started,retained_raw7=captured)
    except BaseException as error:
        steps.receipt.update(state='failed',error=str(error),seconds=time.monotonic()-started);raise
    finally:
        steps.save();print(json.dumps({key:value for key,value in steps.receipt.items() if key!='sources'},indent=2),flush=True)

def main():
    if len(sys.argv)==3 and sys.argv[1]=='--worker':return worker(float(sys.argv[2]))
    started=time.monotonic();process=subprocess.Popen([sys.executable,'scripts/storage.py','run','--no-prune','--',str(ROOT/'obj/audit-venv/bin/python'),__file__,'--worker',str(started)],start_new_session=True)
    status=1
    try:
        status=process.wait(timeout=max(.01,started+188-time.monotonic()));return status
    except subprocess.TimeoutExpired:
        os.killpg(process.pid,signal.SIGTERM)
        try:process.wait(timeout=max(.01,started+189-time.monotonic()))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=max(.01,started+190-time.monotonic()))
        status=124;return status
    finally:
        if status:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
        print(f'Collected positive missing-only total: {time.monotonic()-started:.2f}s status={status}',flush=True)
if __name__=='__main__':sys.exit(main())
