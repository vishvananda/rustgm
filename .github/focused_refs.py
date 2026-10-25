"""Observe a frozen new Assert set; hypotheses and infrastructure stay distinct."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time


def digest(value):
    return hashlib.sha256(value).hexdigest()


def worker():
    started = time.monotonic()
    plan_path = Path('.github/assert-oracles-plan.json')
    plan_raw = plan_path.read_bytes()
    assert digest(plan_raw) == '39ae5c72dbae101f73c6ae45ce66ad873a8bfe71a225b99a0b264017b7509293'
    plan = json.loads(plan_raw)
    assert len(plan['cases']) == len({case['name'] for case in plan['cases']}) == 16
    out = Path('obj/parallel/all-attribute-selected-output-v1')
    out.mkdir(parents=True, exist_ok=False)
    (out / 'plan.json').write_bytes(plan_raw)
    rustc = Path(subprocess.check_output(['rustup', 'which', '--toolchain', '1.98.0', 'rustc'], text=True, timeout=5).strip())
    assert digest(rustc.read_bytes()) == plan['rustc_sha256']
    version = subprocess.check_output([str(rustc), '-vV'], timeout=5)
    (out / 'rustc-vV.txt').write_bytes(version)
    rows, infra, cleanup = [], [], []
    try:
        for case in plan['cases']:
            if time.monotonic() - started >= 110:
                raise TimeoutError('frozen oracle budget exhausted')
            name = case['name']
            source = Path('.github/assert-oracles') / (name + '.rs')
            assert digest(source.read_bytes()) == case['sha256']
            folder = out / name
            folder.mkdir()
            shutil.copy2(source, folder / 'source.rs')
            target = folder / ('program' if case['mode'] == 'native' else 'output.rmeta')
            command = [str(rustc), '--edition=' + plan['edition'], '--target=' + plan['target'],
                       '-Cpanic=' + case['panic'], '--error-format=json', '--crate-name', 'assert_probe']
            if case['mode'] == 'metadata':
                command += ['--emit=metadata']
            command += [str(source), '-o', str(target)]
            began = time.monotonic()
            result = subprocess.run(command, capture_output=True, timeout=12)
            (folder / 'compile.stdout').write_bytes(result.stdout)
            (folder / 'compile.stderr').write_bytes(result.stderr)
            diagnostics = [json.loads(line) for line in result.stderr.splitlines() if line.startswith(b'{')]
            errors = [d for d in diagnostics if d.get('level') == 'error']
            codes = [d['code']['code'] for d in errors if d.get('code')]
            warnings = [d['code']['code'] if d.get('code') else None for d in diagnostics if d.get('level') == 'warning']
            row = dict(case=name, mode=case['mode'], panic=case['panic'], command=command,
                       source_sha256=case['sha256'], compile_status=result.returncode,
                       error_codes=codes, warning_codes=warnings,
                       errors=[d.get('message') for d in errors],
                       compile_stdout_sha256=digest(result.stdout), compile_stderr_sha256=digest(result.stderr))
            bad = result.returncode not in (0, 1) or b'internal compiler error' in result.stderr or b'panicked at' in result.stderr
            bad |= result.returncode == 1 and not errors
            if bad:
                infra.append(dict(case=name, phase='compile', status=result.returncode))
            row['prediction_matches'] = result.returncode == case['expected_compile_status'] and codes == case['expected_error_codes'] and warnings == case['expected_warning_codes']
            if case['mode'] == 'native' and result.returncode == 0:
                execution = subprocess.run([str(target.resolve())], capture_output=True, timeout=5)
                (folder / 'run.stdout').write_bytes(execution.stdout)
                (folder / 'run.stderr').write_bytes(execution.stderr)
                row.update(run_status=execution.returncode, stdout=execution.stdout.decode(),
                           run_stdout_sha256=digest(execution.stdout), run_stderr_sha256=digest(execution.stderr))
                row['prediction_matches'] &= execution.returncode == case['expected_run_status'] and execution.stdout == case['expected_stdout'].encode() and execution.stderr == b''
                if execution.returncode < 0 or execution.returncode == 86:
                    infra.append(dict(case=name, phase='run', status=execution.returncode))
            if target.exists() and not bad:
                cleanup.append(dict(case=name, path=str(target), bytes=target.stat().st_size, sha256=digest(target.read_bytes())))
                target.unlink()
            row['seconds'] = time.monotonic() - began
            rows.append(row)
    except BaseException as error:
        infra.append(dict(phase='observer', error=repr(error)))
        raise
    finally:
        binding = dict(state='observed' if not infra and len(rows)==16 else 'infrastructure_failed',
                       ci_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                       plan_sha256=digest(plan_raw), rustc_sha256=plan['rustc_sha256'],
                       version_sha256=digest(version), seconds=time.monotonic()-started,
                       observations=rows, disagreements=[r['case'] for r in rows if not r['prediction_matches']],
                       infra_failures=infra, cleanup=cleanup,
                       scope='independent observations only; no RustGM candidate implementation or native-unwind claim')
        (out / 'binding.json').write_text(json.dumps(binding, indent=2)+'\n')
        print(json.dumps({k:binding[k] for k in ['state','plan_sha256','seconds','disagreements','infra_failures']},indent=2))
    assert len(rows)==16 and not infra


if __name__ == '__main__':
    if '--worker' in sys.argv:
        worker()
    else:
        raise SystemExit(subprocess.run(['timeout','--signal=TERM','--kill-after=3s','130s',sys.executable,
            'scripts/storage.py','run','--no-prune','--',sys.executable,__file__,'--worker'],check=False).returncode)
