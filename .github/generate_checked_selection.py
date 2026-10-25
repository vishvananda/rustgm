#!/usr/bin/env python3
"""Proposed selected generator: validate all declared contracts before publishing refs."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import time


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(case, actual):
    output, errors, status = actual
    expected = case['contract']
    label = case['path']
    if status != f"{case['exit_status']}\n".encode():
        raise ValueError(f'{label}: independent exit contract differs')
    text = errors.decode('utf-8')
    warnings = [line for line in text.splitlines(keepends=True) if line.startswith('warning[')]
    codes = [re.match(r'warning\[([^]]+)\]:', line)[1] for line in warnings]
    if codes != expected['warning_codes_in_order']:
        raise ValueError(f'{label}: warning codes/count/order differ: {codes}')
    if 'warning_stderr' in expected and ''.join(warnings) != expected['warning_stderr']:
        raise ValueError(f'{label}: exact warning stderr differs')
    if 'warning_messages_in_order' in expected:
        messages = [re.fullmatch(r'warning\[[^]]+\]: file 0: \d+: (.*)\n', line) for line in warnings]
        if any(match is None for match in messages) or [match[1] for match in messages] != expected['warning_messages_in_order']:
            raise ValueError(f'{label}: warning messages/order differ')
    if case['exit_status'] == 1:
        if output or not text or 'ERROR:' not in text:
            raise ValueError(f'{label}: error requires empty stdout and original diagnostic')
        for fragment in expected['error_fragments']:
            if fragment not in text:
                raise ValueError(f'{label}: required error fragment absent: {fragment}')
        if 'exact_error_stderr' in expected and text != expected['exact_error_stderr']:
            raise ValueError(f'{label}: mixed warning/error bytes differ')
        if label.endswith('diagnostic-condition-not-selected.rs') and 'note: EMPTY' in text:
            raise ValueError(f'{label}: unselected diagnostic condition became active')
    else:
        if ''.join(warnings) != text:
            raise ValueError(f'{label}: undeclared successful diagnostic bytes')
        prefixes = {'--emit-body-ir': b'body-crate 1\n', '--emit-lowir': b'function @',
                    '--emit-patterns': b'selected-patterns 1\n', '--emit-semantics': b'semantic-crate 1\n',
                    '--emit-trait-semantics': b'trait-semantics 1\n', '--emit-generic-semantics': b'generic-semantics 1\n',
                    '--emit-expanded': b'macro-matching 1\n'}
        if not output.startswith(prefixes[case['effective_mode']]):
            raise ValueError(f'{label}: successful typed-view prefix absent')
        if 'pattern_decisions' in expected:
            decisions = [line for line in output.splitlines() if line.startswith(b'match ')]
            if len(decisions) != expected['pattern_decisions'] or any(b'exhaustive=true uninhabited=true ' not in line for line in decisions):
                raise ValueError(f'{label}: canonical coverage decision contract differs')
        if 'integer_transcription' in expected:
            wanted = f'token definition integer "{expected["integer_transcription"]}"\n'.encode()
            opposite = f'token definition integer "{expected["opposite_integer_absent"]}"\n'.encode()
            if output.count(wanted) != 1 or opposite in output:
                raise ValueError(f'{label}: configured branch differs')
            if sum(line.startswith(b'definition ') for line in output.splitlines()) != 1 or sum(line.startswith(b'invocation ') for line in output.splitlines()) != 1 or not output.endswith(b'end-macro-matching definitions 1 invocations 1\n'):
                raise ValueError(f'{label}: configured definition/invocation count differs')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--review', required=True, type=Path)
    parser.add_argument('--app', required=True, type=Path)
    parser.add_argument('--seconds', type=float, default=240)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 240:
        raise ValueError('selected-generation deadline must be at most 240 seconds')
    deadline = time.monotonic() + args.seconds
    root = Path.cwd().resolve()
    sys.path.insert(0, str(root / 'scripts'))
    import course
    from storage import exclusive
    report_bytes = args.review.read_bytes()
    report = json.loads(report_bytes)
    app = args.app.resolve()
    evidence = root / 'obj/parallel/pa31/all-attribute-selected-generation-v1' / str(time.time_ns())
    selected = []
    for stage, declared in report['stages'].items():
        if digest((root / stage / 'tests/manifest.json').read_bytes()) != declared['manifest_sha256']:
            raise ValueError(f'{stage}: source manifest differs from reviewed candidate')
        if digest((root / stage / 'reference.json').read_bytes()) != declared['base_reference_sha256']:
            raise ValueError(f'{stage}: full reference receipt differs')
        for batch in declared['preserve_scoped_receipts']:
            if digest((root / batch['path']).read_bytes()) != batch['sha256']:
                raise ValueError(f'{stage}: preexisting scoped receipt differs')
        manifest, cases = course.load_cases(root, stage)
        if declared['already_generated']:
            # Import the separately reviewed PA7 artifact before invoking this driver.
            course.reference_batches.validate(root, stage, manifest, cases)
        by_path = {case['path']: case for case in cases}
        for case in declared['selected_cases']:
            if digest((root / stage / case['path']).read_bytes()) != case['source_sha256']:
                raise ValueError(f'{stage}/{case["path"]}: fixture differs')
            if course.reference_batches.case_digest(by_path[case['path']], manifest['mode']) != case['case_sha256']:
                raise ValueError(f'{stage}/{case["path"]}: effective case contract differs')
            if not declared['already_generated']:
                selected.append((stage, manifest, by_path[case['path']], case))
    selected_paths = {f'{stage}/{case["path"]}' for stage, _, case, _ in selected}
    excluded = {Path(str(root / source) + suffix) for source in selected_paths for suffix in course.SIDECARS}
    preserved = {path: digest(path.read_bytes()) for stage in report['stages']
                 for path in (root / stage / 'tests').rglob('*.ref*')
                 if path not in excluded}
    preserved.update({root / stage / 'reference.json': declared['base_reference_sha256'] for stage, declared in report['stages'].items()})
    preserved.update({root / batch['path']: batch['sha256'] for declared in report['stages'].values() for batch in declared['preserve_scoped_receipts']})
    old_batches = {path for stage in report['stages'] for path in (root / stage / 'reference-batches').glob('*.json')}
    def remaining_timeout():
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ValueError('selected-generation global deadline exhausted')
        return min(10, remaining)
    original = course.run_case
    def preflight(row):
        stage, manifest, case, contract = row
        actual = None
        try:
            _, actual = original(root, stage, manifest['mode'], case, [str(app)], remaining_timeout())
            verify(contract, actual)
            error = None
        except (ValueError, OSError) as problem:
            error = str(problem)
        return (stage, case['path']), actual, error
    with exclusive(root):
        # No sidecar is changed before every selected case passes its independent contract.
        with ThreadPoolExecutor(max_workers=2) as pool:
            observations = list(pool.map(preflight, selected))
        evidence.mkdir(parents=True, exist_ok=True)
        for (stage, path), actual, _ in observations:
            if actual is not None:
                destination = evidence / 'preflight' / stage / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                for suffix, data in zip(('.stdout', '.stderr', '.exit_status'), actual):
                    Path(str(destination) + suffix).write_bytes(data)
        failures = [{'stage': stage, 'path': path, 'error': error} for (stage, path), _, error in observations if error is not None]
        (evidence / 'preflight-results.json').write_text(json.dumps({'count': len(observations), 'failures': failures}, indent=2) + '\n')
        if failures:
            for failure in failures:
                print(json.dumps(failure), file=sys.stderr)
            raise ValueError(f'{len(failures)}/91 preflight contracts failed; NO references written; evidence {evidence}')
        frozen = {key: actual for key, actual, _ in observations}
        receipt = {'candidate': report['candidate'], 'actual_ci_source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
                   'review_sha256': digest(report_bytes),
                   'app': str(app), 'app_sha256': digest(app.read_bytes()),
                   'compiler_source_sha256': {str(path.relative_to(root)): digest(path.read_bytes()) for path in sorted((root / 'dev').rglob('*.rs'))},
                   'manifest_sha256': {stage: declared['manifest_sha256'] for stage, declared in report['stages'].items()},
                   'selected_source_sha256': {f'{stage}/{case["path"]}': case['source_sha256'] for stage, declared in report['stages'].items() for case in declared['selected_cases']},
                   'selected': [{'stage': stage, 'path': path, 'outputs_sha256': [digest(part) for part in actual]}
                                for (stage, path), actual in frozen.items()]}
        (evidence / 'checked-preflight.json').write_text(json.dumps(receipt, indent=2) + '\n')
        def checked(root_arg, stage, mode, case, app_arg, timeout):
            fixture, actual = original(root_arg, stage, mode, case, app_arg, remaining_timeout())
            if actual != frozen[(stage, case['path'])]:
                raise ValueError(f'{stage}/{case["path"]}: changed between checked preflight and generation')
            return fixture, actual
        course.run_case = checked
        for stage, declared in report['stages'].items():
            if declared['already_generated']:
                continue
            if not course.run_stage(root, stage, [str(app)], bless=True, jobs=2, timeout=10,
                                    evidence_root=evidence,
                                    case_paths=[case['path'] for case in declared['selected_cases']]):
                raise ValueError(f'{stage}: selected reference generation failed')
        for path, expected in preserved.items():
            if digest(path.read_bytes()) != expected:
                raise ValueError(f'unselected reference changed: {path}')
        for stage, declared in report['stages'].items():
            manifest, cases = course.load_cases(root, stage)
            course.reference_batches.validate(root, stage, manifest, cases)
        new_batches = {path for stage in report['stages'] for path in (root / stage / 'reference-batches').glob('*.json')} - old_batches
        if len(new_batches) != 8:
            raise ValueError('expected exactly eight new scoped receipts')
        generated = sorted(excluded | new_batches)
        receipt['generated_sha256'] = {str(path.relative_to(root)): digest(path.read_bytes()) for path in generated}
        receipt['state'] = 'passed'
        (evidence / 'binding.json').write_text(json.dumps(receipt, indent=2) + '\n')
        bundle = evidence.with_suffix('.tar.gz')
        with tarfile.open(bundle, 'w:gz') as archive:
            for path in [*generated, *sorted(path for path in evidence.rglob('*') if path.is_file())]:
                archive.add(path, arcname=str(path.relative_to(root)), recursive=False)
        print(f'Upload selected-only bundle: {bundle.relative_to(root)} sha256={digest(bundle.read_bytes())}')
        print('All 91 remaining selected contracts/references passed; all unselected references and full receipts unchanged.')


if __name__ == '__main__':
    main()
