"""Hosted native3/CLI follow-up; requires exact successful private25 evidence."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / 'scripts'))
from backend import emit_object, load_tools
from fastcheck_run import fingerprint


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    plan_path, proof_path, launcher_path, cli_log = map(Path, sys.argv[1:5])
    plan = json.loads(plan_path.read_text())
    proof = json.loads(proof_path.read_text())
    launcher = json.loads(launcher_path.read_text())
    assert plan['sources'] == fingerprint(ROOT) == proof['sources']
    assert proof['state'] == launcher['state'] == 'passed'
    assert set(proof['groups']['lib']) == set(plan['private_cases'])
    assert len(proof['groups']['lib']) == plan['private_count'] == 25
    assert all(digest(ROOT / path) == value for path, value in plan['fixtures'].items())
    harness = Path(proof['binaries']['lib']['path'])
    assert digest(harness) == proof['binaries']['lib']['sha256']
    rows = [json.loads(line) for line in cli_log.read_text().splitlines() if line.strip()]
    bins = [row for row in rows if row.get('reason') == 'compiler-artifact'
            and row.get('target', {}).get('name') == 'rustgm' and row.get('executable')]
    assert len(bins) == 1
    assert Path(bins[0]['target']['src_path']).resolve() == (ROOT / 'dev/bin/rustgm.rs').resolve()
    cli = Path(bins[0]['executable']).resolve()
    assert cli.is_relative_to(ROOT / 'obj/rust')
    tools = load_tools(('cppgm++',))
    rustc = Path(subprocess.check_output(['rustup', 'which', 'rustc'], text=True).strip())
    cc = Path(shutil.which('cc')).resolve()
    observer = plan_path.parent / plan['observer']['packet_file']
    assert digest(observer) == plan['observer']['sha256']
    assert digest(ROOT / 'backend.lock.json') == plan['backend_lock_sha256']
    folder = ROOT / 'obj/qualification/reservation-native'
    generated = folder / 'generated'
    generated.mkdir(parents=True, exist_ok=False)
    inputs = {str(path): digest(path) for path in
              [plan_path, proof_path, launcher_path, cli_log, harness, cli, rustc, cc,
               Path(tools['cppgm++']), observer, Path(__file__)]}
    for case in plan['native_cases']:
        source = ROOT / case['source']
        assert digest(source) == case['source_sha256']
        for path in [source, ROOT / case['candidate_lowir']]:
            inputs[str(path)] = digest(path)
    receipt = {'state': 'running', 'scope': 'native3 + actual CLI/fence + retained controls',
               'inputs': inputs, 'source_fingerprint': plan['sources'], 'commands': []}
    environment = {**os.environ, 'RUSTC_BOOTSTRAP': '1', 'OMP_NUM_THREADS': '2'}
    started = time.monotonic()

    def run(name, command, status=0, stdout=None, diagnostic=None):
        command = list(map(str, command))
        remaining = started + 270 - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('hosted native/CLI budget exhausted')
        try:
            result = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                    timeout=min(40, remaining))
        except subprocess.TimeoutExpired as error:
            result = subprocess.CompletedProcess(command, 124, error.stdout or b'', error.stderr or b'')
        (folder / (name + '.stdout')).write_bytes(result.stdout)
        (folder / (name + '.stderr')).write_bytes(result.stderr)
        matches = result.returncode == status
        if stdout is not None:
            matches = matches and result.stdout == stdout.encode() and not result.stderr
        if diagnostic is not None:
            matches = matches and not result.stdout and diagnostic.encode() in result.stderr
        receipt['commands'].append({'name': name, 'command': command,
                                    'status': result.returncode, 'expected_status': status,
                                    'matches': matches,
                                    'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
                                    'stderr_sha256': hashlib.sha256(result.stderr).hexdigest()})
        if not matches:
            raise ValueError(name + ': outcome differs from declared contract')
        return result

    try:
        version = run('rustc-version', [rustc, '-vV'])
        assert version.stdout.startswith(b'rustc 1.98.0 ')
        for index, case in enumerate(plan['related_existing_controls']):
            result = run('existing-control-' + str(index), [harness, '--exact', case, '--nocapture'])
            assert b'test result: ok. 1 passed; 0 failed;' in result.stdout
        for case in plan['native_cases']:
            name = case['name']
            source = ROOT / case['source']
            run(name + '-rust-object', [rustc, '--edition=2024', '-Zthreads=2',
                '--target=x86_64-unknown-linux-gnu', '--cap-lints=allow', '--crate-type=lib',
                '--crate-name', name, '--emit=obj', '-Cpanic=abort', source,
                '-o', generated / (name + '-rust.o')], stdout='')
            run(name + '-rust-link', [cc, '-std=c11', observer,
                generated / (name + '-rust.o'), '-o', generated / (name + '-rust')], stdout='')
            run(name + '-rust-native', [generated / (name + '-rust')], stdout='7\n')
            emit_object(ROOT / case['candidate_lowir'], generated / (name + '.o'), tools,
                        run=lambda args: run(name + '-candidate-object', args))
            run(name + '-candidate-link', [cc, '-std=c11', observer,
                generated / (name + '.o'), '-lm', '-o', generated / name], stdout='')
            run(name + '-candidate-native', [generated / name], stdout='7\n')
            result = run(name + '-cli-inspection', [cli, '--emit-trait-semantics',
                         '--edition=2024', '--cap-lints=allow', source])
            assert result.stdout and not result.stderr
            contract = plan['cli_library_transport']
            run(name + '-cli-transport-fence', [cli, contract['mode'], '--edition=2024',
                '--cap-lints=allow', *contract['args'], source],
                status=contract['expected_status'], diagnostic=contract['expected_diagnostic'])
        for index, case in enumerate(plan['cli_semantic_cases'] + plan['cli_associated_cases']):
            result = run('pa20-cli-' + str(index), [cli, case['mode'], '--edition=2024',
                         '--cap-lints=allow', ROOT / case['source']], status=case['expected_status'])
            assert (result.stdout and not result.stderr) if case['expected_status'] == 0 else (
                not result.stdout and result.stderr.startswith(b'ERROR: '))
        assert plan['sources'] == fingerprint(ROOT)
        assert all(digest(path) == value for path, value in inputs.items())
        receipt['state'] = 'passed'
    except BaseException as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        receipt['seconds'] = time.monotonic() - started
        receipt['artifacts'] = {str(path): digest(path) for path in generated.iterdir() if path.is_file()}
        (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
