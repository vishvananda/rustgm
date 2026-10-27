#!/usr/bin/env python3
"""Compare actual host DAG execution with independent pinned, offline Cargo."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time
import tomllib

from build_environment import Context
from build_units import Execution
from cargo_plan import Job, execute as plan_cargo
from crate_vendor import Vendor
from host_graph import execute
from host_process import CommandLog, digest
from package_snapshot import Snapshot

sys.path.insert(0, str(Path(__file__).resolve().parent / 'tests'))
from host_graph_fixtures import local_packages, packages

ROOT = Path(__file__).resolve().parent.parent
TRIPLE = 'x86_64-unknown-linux-gnu'
ENVIRONMENT = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}
ANSWER = b'pub const ANSWER:u8=17;\n'


def save(path, value):
    path.write_text(json.dumps(value, indent=2, default=str) + '\n')


def toolchain():
    channel = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    result = {}
    for name in ('cargo', 'rustc'):
        query = ['rustup', 'which', '--toolchain', channel, name]
        path = Path(subprocess.check_output(query, cwd=ROOT, text=True, timeout=30).strip())
        version = subprocess.check_output([path, '--version'], env=ENVIRONMENT, text=True, timeout=30)
        if not version.startswith(f'{name} {channel} '):
            raise ValueError(f'host graph qualification requires pinned {name} {channel}')
        result[name] = {'path': str(path), 'sha256': digest(path), 'version': version.strip()}
    return result


def independent(root, snapshots, driver, tools, local=False):
    vendor = None if local else Vendor(root / 'vendor', snapshots)
    cargo, compiler = (Path(tools[name]['path']) for name in ('cargo', 'rustc'))
    declaration = {'expected': {'cargo_status': 101, 'failure': 'intentional target graph stop',
                               'app_generated': ANSWER.decode(), 'host_compilations': 6,
                               'script_runs': 5, 'candidate_target_compilations': 0},
                   'packages': [dict(p.identity, files=p.files, directories=p.directories) for p in snapshots],
                   'tools': tools, 'fixture_sha256': digest(Path(__file__).parent / 'tests/host_graph_fixtures.py')}
    save(root / 'declaration.json', declaration)
    commands = []
    log = CommandLog(root, driver, commands, lambda: save(root / 'commands.json', commands))
    environment = {**ENVIRONMENT, 'CARGO_HOME': str(root / 'cargo-home'),
                   'CARGO_TARGET_DIR': str(root / 'target'), 'CARGO_INCREMENTAL': '0',
                   'CARGO_BUILD_JOBS': '2', 'RUSTC': str(compiler), 'CARGO_ENCODED_RUSTFLAGS': ''}
    command = [cargo, 'build', '--offline', '-vv', '--manifest-path', driver / 'Cargo.toml',
               '--target', TRIPLE, *(vendor.arguments() if vendor is not None else [])]
    status = log.run('independent', command, environment)
    if status != 101 or declaration['expected']['failure'] not in (root / 'independent.stderr').read_text():
        raise ValueError('independent Cargo did not reach its deliberate target stop')
    answers = list((root / 'target').rglob('answer.rs'))
    if len(answers) != 1 or answers[0].read_bytes() != ANSWER:
        raise ValueError('independent host scripts differ from the declared generated answer')
    inputs = tuple((path, digest(path)) for path in sorted(driver.iterdir()) if path.is_file())
    job = Job(cargo, tools['cargo']['sha256'], compiler, tools['rustc']['sha256'],
              driver / 'Cargo.toml', driver / 'Cargo.lock', inputs, 'app' if local else 'driver',
              TRIPLE, vendor=vendor, local_packages=snapshots if local else ())
    plan = plan_cargo(job, root / 'plan')
    native = [u for u in plan.units if u.pkg_id == snapshots[2].cargo_id and u.target.kind == ('custom-build',)]
    if (sum(u.execution is Execution.HOST_COMPILE for u in native) != 1
            or sum(u.execution is Execution.HOST_RUN for u in native) != 2):
        raise ValueError('independent graph lacks the declared shared host script')
    save(root / 'independent.json', {'state': 'passed', 'answer_sha256': digest(answers[0]),
                                    'declaration_sha256': digest(root / 'declaration.json')})
    return plan


def candidate(root, plan, snapshots, tools):
    compiler = Path(tools['rustc']['path'])
    commands = []
    log = CommandLog(root, ROOT, commands, lambda: save(root / 'cfg-commands.json', commands))
    if log.run('cfg', [compiler, '--print=cfg', '--target', TRIPLE], ENVIRONMENT):
        raise ValueError('explicit cfg query failed')
    cfg = {}
    for line in (root / 'cfg.stdout').read_text().splitlines():
        key, equals, value = line.partition('=')
        cfg.setdefault(key, [])
        if equals:
            cfg[key].append(json.loads(value))
    # This fixture's frozen host and target profiles both use unwind.
    context = Context(TRIPLE, TRIPLE, compiler, tools['rustc']['sha256'],
                      tuple((key, tuple(values)) for key, values in cfg.items()),
                      ((root / 'cfg.stdout', digest(root / 'cfg.stdout')),))
    # Pinned Cargo applies warn caps to this directory-source fixture, while
    # workspace members have no cap. Keep this independently observed input
    # explicit; source kind alone is not a general Cargo cap inference rule.
    caps = {p.cargo_id: 'warn' for p in snapshots if isinstance(p, Snapshot)}
    result = execute(plan, snapshots, {None: context, TRIPLE: context}, root / 'candidate',
                     lint_caps=caps)
    receipt = json.loads((root / 'candidate/receipt.json').read_text())
    calls = receipt['commands']
    if (sum(row['name'] == 'compile' for row in calls) != 6
            or sum(row['name'] == 'execute' for row in calls) != 5
            or any('--target' in row['command'] for row in calls)):
        raise ValueError('candidate did not build each required host unit exactly once')
    if sum('--forbid=unsafe_code' in row['command'] for row in calls) != 3:
        raise ValueError('helper host library and script lost their actual manifest lint policy')
    if sum('--cap-lints' in row['command'] for row in calls) != (6 if caps else 0):
        raise ValueError('host compilation lost its explicitly observed dependency lint caps')
    app = [value for index, value in result.items() if plan.units[index].pkg_id == snapshots[0].cargo_id]
    expected = json.loads((root / 'independent.json').read_text())['answer_sha256']
    if len(app) != 1 or (app[0].folder / 'answer.rs').read_bytes() != ANSWER or app[0].files['answer.rs'] != expected:
        raise ValueError('candidate host result differs from independent Cargo')
    save(root / 'complete.json', {'state': 'passed', 'host_compilations': 6, 'script_runs': 5,
                                 'candidate_target_compilations': 0, 'answer_sha256': expected,
                                 'candidate_receipt_sha256': digest(root / 'candidate/receipt.json')})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, default=ROOT / 'obj/qualification/host-graph' / str(time.time_ns()))
    parser.add_argument('--local', action='store_true', help='qualify complete local path package inputs')
    args = parser.parse_args()
    root = args.out.absolute()
    root.mkdir(parents=True, exist_ok=False)
    tools = toolchain()
    snapshots, driver = (local_packages if args.local else packages)(root)
    plan = independent(root, snapshots, driver, tools, args.local)
    candidate(root, plan, snapshots, tools)
    print(f'PASS host graph: six compilations, five runs; generated answer matches independent Cargo ({root})')


if __name__ == '__main__':
    main()
