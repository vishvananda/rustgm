"""Run the five predeclared nested Into outcomes, without blessing candidate output."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
FOLDER = ROOT / 'obj/qualification/source-from-rust'
GENERATED = FOLDER / 'generated'
CASES = ('nested-into-dot', 'nested-into-ufcs', 'nested-into-explicit',
         'nested-into-lookalike', 'nested-into-wrapper')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    GENERATED.mkdir(parents=True, exist_ok=False)
    contract_path = ROOT / 'pa31/tests/spec/source-from/contract.json'
    contract = json.loads(contract_path.read_text())
    rustc = subprocess.check_output(['rustup', 'which', 'rustc'], text=True).strip()
    receipt = {'state': 'running', 'purpose': 'independent predeclared Rust outcomes',
               'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
               'contract_sha256': digest(contract_path.read_bytes()),
               'rustc': rustc, 'rustc_sha256': digest(Path(rustc).read_bytes()),
               'rustc_version': subprocess.check_output([rustc, '-vV'], text=True),
               'cases': [], 'commands': []}
    environment = {**os.environ, 'RUSTC_BOOTSTRAP': '1'}

    def run(name, command):
        started = time.monotonic()
        output = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, timeout=60)
        for channel in ('stdout', 'stderr'):
            (FOLDER / f'{name}.{channel}').write_bytes(getattr(output, channel))
        receipt['commands'].append({'name': name, 'command': command, 'status': output.returncode,
                                    'seconds': time.monotonic() - started,
                                    **{f'{channel}_sha256': digest(getattr(output, channel))
                                       for channel in ('stdout', 'stderr')}})
        return output

    try:
        failures = []
        for name in CASES:
            source = ROOT / f'pa31/tests/spec/source-from/{name}.rs'
            declared = contract['cases'][source.name]
            if digest(source.read_bytes()) != declared['sha256']:
                raise ValueError(f'{name}: source changed after declaration')
            row = {'name': name, 'source_sha256': declared['sha256'], 'state': 'failed'}
            receipt['cases'].append(row)
            object_path = GENERATED / f'{name}.o'
            executable = GENERATED / f'{name}.native'
            output = run(name + '-object', [rustc, '--edition=2024', '-Zthreads=2',
                         '-Cpanic=abort', '--cap-lints=allow', '--crate-type=lib',
                         '--crate-name', 'minicore', '--emit=obj', str(source), '-o', str(object_path)])
            if output.returncode != declared['rust_object_expected'] or output.stdout or output.stderr:
                failures.append(name + ': object outcome differs from predeclared result')
                continue
            output = run(name + '-link', ['cc', str(ROOT / 'pa31/tests/spec/source-from/observer.c'),
                                         str(object_path), '-lm', '-o', str(executable)])
            if output.returncode or output.stdout or output.stderr:
                failures.append(name + ': native link failed')
                continue
            output = run(name + '-native', [str(executable)])
            expected = declared['native']
            if (output.returncode != expected['exit_status'] or output.stdout != expected['stdout'].encode()
                    or output.stderr != expected['stderr'].encode()):
                failures.append(name + ': native outcome differs from predeclared result')
                continue
            row['state'] = 'passed'
        if failures:
            raise ValueError('; '.join(failures))
        receipt['state'] = 'passed'
        print('All five declared nested Into Rust object/native outcomes passed')
    except BaseException as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        (FOLDER / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
