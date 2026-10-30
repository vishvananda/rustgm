"""Generate only three owning PA20 references after actual private/native success.

Authored for hosted CI; no local execution or qualification is implied.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / 'scripts'))
from fastcheck_run import fingerprint

CASES = (
    ('closed-local.rs', '6aa761f203981c09e672f87c93f42dcf7cbdcd69d8478c98ec468e7696ab3ab4', 0),
    ('real-bound-overlap.rs', '7c81ecf37a33210f8979b9dab266282e94ff3e481322e2b3a82d851a7308d117', 1),
    ('open-bound-overlap.rs', '3a548cb538b8a4414fec32b2492a8573931c74f10020321b19f0ee033e726a90', 1),
)
PREFIX = 'tests/spec/local-bound-coherence/'
SUFFIXES = ('.ref', '.ref.stderr', '.ref.exit_status')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    folder = ROOT / 'obj/qualification/reservation-references'
    folder.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    receipt = {'state': 'running', 'commands': [], 'scope': 'fresh Rust metadata3 then owning PA20 partial reference generation ONLY'}
    environment = {**os.environ, 'RUSTUP_TOOLCHAIN': '1.98.0', 'RUSTC_BOOTSTRAP': '1',
                   'OMP_NUM_THREADS': '2', 'RUST_TEST_THREADS': '2', 'RUSTGM_TEST_JOBS': '2'}

    def run(name, command, expected, limit=40):
        command = list(map(str, command))
        remaining = started + 270 - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('reference observation/generation budget exhausted')
        out, err = folder / (name + '.stdout'), folder / (name + '.stderr')
        begin = time.monotonic()
        with out.open('wb') as stdout, err.open('wb') as stderr:
            process = subprocess.Popen(command, cwd=ROOT, env=environment,
                                       stdout=stdout, stderr=stderr, start_new_session=True)
            try:
                status = process.wait(timeout=min(limit, remaining))
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=1)
                status = 124
        row = {'name': name, 'command': command, 'status': status,
               'expected_status': expected, 'matches': status == expected,
               'seconds': time.monotonic() - begin,
               'stdout_sha256': digest(out), 'stderr_sha256': digest(err)}
        receipt['commands'].append(row)
        write_json(folder / 'receipt.json', receipt)
        return status == expected

    try:
        packet = ROOT / '.github/qualification/reservation/reservation.json'
        plan = json.loads(packet.read_text())
        assert digest(packet) == '70046c8b489755240feb135011ce334cb14d33a1b5f25ad43762c2560583066c'
        assert fingerprint(ROOT) == plan['sources']
        assert all(digest(ROOT / name) == value for name, value in plan['fixtures'].items())
        selected_path = ROOT / 'obj/qualification/ci-build/reservation-private.json'
        selected = json.loads(selected_path.read_text())
        private_path, launcher_path = Path(selected['receipt']), Path(selected['launcher'])
        private, launcher = json.loads(private_path.read_text()), json.loads(launcher_path.read_text())
        assert selected['state'] == 'passed-private25-only'
        assert digest(private_path) == selected['receipt_sha256']
        assert digest(launcher_path) == selected['launcher_sha256']
        assert private['state'] == launcher['state'] == 'passed'
        assert private['sources'] == plan['sources']
        assert set(private['groups']['lib']) == set(plan['private_cases'])
        assert len(private['groups']['lib']) == len(plan['private_cases']) == 25
        cli_log = ROOT / 'obj/qualification/ci-build/reservation-cli.jsonl'
        rows = [json.loads(line) for line in cli_log.read_text().splitlines() if line.strip()]
        bins = [row for row in rows if row.get('reason') == 'compiler-artifact'
                and row.get('target', {}).get('name') == 'rustgm' and row.get('executable')]
        assert len(bins) == 1
        assert Path(bins[0]['target']['src_path']).resolve() == (ROOT / 'dev/bin/rustgm.rs').resolve()
        cli = Path(bins[0]['executable']).resolve()
        assert cli.is_relative_to(ROOT / 'obj/rust') and cli.is_file()
        native_path = ROOT / 'obj/qualification/reservation-native/receipt.json'
        native_launcher_path = ROOT / 'obj/qualification/ci-build/reservation-native-launcher.json'
        native = json.loads(native_path.read_text())
        assert native['state'] == json.loads(native_launcher_path.read_text())['state'] == 'passed'
        assert native['source_fingerprint'] == plan['sources']
        native_keys = ((cli, str(cli)), (packet, '.github/qualification/reservation/reservation.json'),
                       (private_path, str(private_path)), (launcher_path, str(launcher_path)),
                       (cli_log, 'obj/qualification/ci-build/reservation-cli.jsonl'))
        for path, key in native_keys:
            assert native['inputs'][key] == digest(path)
        stage = ROOT / 'pa20'
        manifest_path = stage / 'tests/manifest.json'
        manifest = json.loads(manifest_path.read_text())
        declared = {case['path']: case for case in manifest['cases']}
        cases = []
        for name, source_hash, status in CASES:
            selector = PREFIX + name
            case = declared[selector]
            source = stage / selector
            assert digest(source) == source_hash == case['source_sha256']
            assert case['exit_status'] == status
            assert case['rustc'] == {'kind': 'library', 'status': status, 'emit': 'metadata', 'default_lints': True}
            assert case.get('mode', manifest['mode']) == '--emit-trait-semantics'
            assert not case.get('args') and not case.get('environment') and not case.get('auxiliary')
            assert not any(Path(str(source) + suffix).exists() for suffix in SUFFIXES)
            cases.append({'selector': selector, 'source_sha256': source_hash, 'expected_rust_status': status,
                          'expected_candidate_status': status})
        base = stage / 'reference.json'
        assert isinstance(json.loads(base.read_text()).get('case_contracts'), dict)
        before_sidecars = {str(path.relative_to(ROOT)): digest(path)
                           for path in (stage / 'tests').rglob('*.ref*') if path.is_file()}
        before_batches = {str(path.relative_to(ROOT)): digest(path)
                          for path in (stage / 'reference-batches').glob('*.json')}
        inputs = {str(path): digest(path) for path in (packet, selected_path, private_path, launcher_path,
                  cli_log, cli, native_path, native_launcher_path, manifest_path, base, Path(__file__).resolve())}
        rust_flags = ['--edition=2024', '-Zthreads=2', '--target=x86_64-unknown-linux-gnu',
                      '--crate-type=lib', '--emit=metadata']
        write_json(folder / 'declared-plan.json', {'state': 'declared-before-observation', 'cases': cases,
                   'rust_toolchain': '1.98.0', 'rust_flags': rust_flags,
                   'rust_lints': 'default; no --cap-lints override', 'environment': {
                       key: environment[key] for key in ('RUSTUP_TOOLCHAIN', 'RUSTC_BOOTSTRAP', 'OMP_NUM_THREADS')},
                   'sources': plan['sources'], 'inputs': inputs,
                   'generator': 'scripts/course.py pa20 --app ACTUAL_CARGO_CLI --bless --jobs 2 --case EXACT_THREE'})
        assert run('rustc-path', ['rustup', 'which', '--toolchain', '1.98.0', 'rustc'], 0)
        rustc = Path((folder / 'rustc-path.stdout').read_text().strip())
        receipt['rustc_sha256'] = digest(rustc)
        assert run('rustc-version', [rustc, '-vV'], 0)
        assert (folder / 'rustc-version.stdout').read_bytes().startswith(b'rustc 1.98.0 ')
        matches = []
        for index, case in enumerate(cases):
            output = folder / ('case-' + str(index) + '.rmeta')
            matches.append(run('rust-metadata-' + str(index), [rustc, *rust_flags,
                '--crate-name', 'reservation_reference_' + str(index), stage / case['selector'], '-o', output],
                case['expected_rust_status']))
        assert all(matches), 'fresh Rust metadata3 differed; reference generation is forbidden'
        assert fingerprint(ROOT) == plan['sources']
        command = [sys.executable, 'scripts/course.py', 'pa20', '--app', cli, '--bless',
                   '--jobs', '2', '--timeout', '20', '--evidence-root', folder / 'course']
        for case in cases:
            command += ['--case', case['selector']]
        assert run('owning-course-generation', command, 0, limit=90)
        assert fingerprint(ROOT) == plan['sources']
        assert all(digest(ROOT / name) == value for name, value in plan['fixtures'].items())
        assert all(digest(path) == value for path, value in inputs.items())
        assert all(digest(ROOT / path) == value for path, value in before_sidecars.items())
        assert all(digest(ROOT / path) == value for path, value in before_batches.items())
        after_batches = {str(path.relative_to(ROOT)): digest(path)
                         for path in (stage / 'reference-batches').glob('*.json')}
        new_batches = set(after_batches) - set(before_batches)
        assert len(new_batches) == 1
        batch_path = ROOT / next(iter(new_batches))
        batch = json.loads(batch_path.read_text())
        assert batch['selected_paths'] == [case['selector'] for case in cases]
        assert batch['base_reference_sha256'] == inputs[str(base)]
        assert batch['manifest_sha256'] == inputs[str(manifest_path)]
        assert batch['compiler_source_sha256'] == {name: value for name, value in plan['sources'].items()
                                                    if name.startswith('dev/') and name.endswith('.rs')}
        generated = [batch_path]
        for case in cases:
            source = stage / case['selector']
            for suffix in SUFFIXES:
                path = Path(str(source) + suffix)
                assert digest(path) == batch['cases'][case['selector']]['outputs'][suffix]
                generated.append(path)
            assert Path(str(source) + '.ref.exit_status').read_text() == str(case['expected_candidate_status']) + '\n'
        new_sidecars = {str(path.relative_to(ROOT)) for path in (stage / 'tests').rglob('*.ref*') if path.is_file()} - set(before_sidecars)
        assert new_sidecars == {str(path.relative_to(ROOT)) for path in generated if path != batch_path}
        # Copy authentic generated bytes and unchanged base/manifest, never write reference contents.
        for path in [*generated, base, manifest_path]:
            target = folder / 'generated' / path.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
        receipt.update(state='passed-reference-generation-only', inputs=inputs,
                       generated={str(path.relative_to(ROOT)): digest(path) for path in generated},
                       sources_after=fingerprint(ROOT), final_full_harness='REQUIRED subsequent workflow step')
    except BaseException as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        receipt['seconds'] = time.monotonic() - started
        receipt['evidence'] = {str(path.relative_to(folder)): digest(path)
                               for path in folder.rglob('*') if path.is_file() and path != folder / 'receipt.json'}
        write_json(folder / 'receipt.json', receipt)


if __name__ == '__main__':
    main()
