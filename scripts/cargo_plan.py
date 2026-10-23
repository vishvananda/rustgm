"""Pinned, frozen path-package planning; never execute a target compilation.

Run under storage.py. This initial adapter deliberately uses a fresh Cargo home
and rejects ambient Cargo configuration. Registry acquisition and complete
package-closure authentication must precede persistent caching or upstream
acceptance; a planned graph alone cannot establish either.
"""
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess
import time

from build_units import MAX_BYTES, parse
from host_build_script import digest


@dataclass(frozen=True)
class Job:
    cargo: Path
    cargo_sha256: str
    query_compiler: Path
    query_compiler_sha256: str
    manifest: Path
    lockfile: Path
    # Absolute files in the caller's declared immutable package closure.
    inputs: tuple[tuple[Path, str], ...]
    package: str
    target: str
    features: tuple[str, ...] = ()
    default_features: bool = True

    def validate(self):
        for path, expected in ((self.cargo, self.cargo_sha256),
                               (self.query_compiler, self.query_compiler_sha256), *self.inputs):
            if not path.is_absolute() or path.is_symlink() or digest(path) != expected:
                raise ValueError('planning tool/input differs from its explicit pin')
        paths = {path for path, _ in self.inputs}
        if len(paths) != len(self.inputs) or not {self.manifest, self.lockfile} <= paths:
            raise ValueError('planning requires unique inputs including manifest and lockfile')
        if self.manifest.name != 'Cargo.toml' or self.lockfile.name != 'Cargo.lock':
            raise ValueError('planning requires explicit Cargo manifest and lockfile')
        if self.lockfile != self.manifest.with_name('Cargo.lock'):
            raise ValueError('select the workspace root manifest beside its frozen lockfile')
        for directory in (self.manifest.parent, *self.manifest.parent.parents):
            if any((directory / '.cargo' / name).exists() for name in ('config', 'config.toml')):
                raise ValueError('ambient Cargo configuration is outside the initial planner profile')
        if not re.fullmatch(r'[A-Za-z0-9_-]+', self.package):
            raise ValueError('invalid selected package name')
        if not re.fullmatch(r'[A-Za-z0-9_-]+', self.target):
            raise ValueError('target must be an explicit built-in triple')
        if len(self.features) != len(set(self.features)) or any(
                not re.fullmatch(r'[A-Za-z0-9_+./-]+', name) for name in self.features):
            raise ValueError('invalid or duplicate selected feature')
        if type(self.default_features) is not bool:
            raise ValueError('default features must be explicitly enabled or disabled')


def execute(job, folder):
    job.validate()
    folder = folder.absolute()
    if folder.exists() or folder.is_symlink():
        raise ValueError('planning evidence directory already exists')
    folder.mkdir(parents=True)
    wrapper = Path(__file__).with_name('cargo_query.py').absolute()
    implementation = {name: digest(wrapper.with_name(name)) for name in (
        'cargo_query.py', 'cargo_plan.py', 'build_units.py', 'host_build_script.py', 'build_script_output.py')}
    environment = {
        'PATH': '/usr/bin:/bin', 'CARGO_HOME': str(folder / 'cargo-home'),
        'CARGO_TARGET_DIR': str(folder / 'target'), 'CARGO_INCREMENTAL': '0',
        'CARGO_BUILD_JOBS': '2', 'RUSTC_BOOTSTRAP': '1', 'RUSTC': str(wrapper),
        'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': '', 'CARGO_ENCODED_RUSTFLAGS': '',
        'RUSTGM_PLAN_QUERY_COMPILER': str(job.query_compiler),
        'RUSTGM_PLAN_QUERY_SHA256': job.query_compiler_sha256,
        'RUSTGM_PLAN_TARGET': job.target, 'RUSTGM_PLAN_QUERY_LOG': str(folder / 'queries.jsonl'),
    }
    command = [str(job.cargo), 'build', '--manifest-path', str(job.manifest),
               '-p', job.package, '--lib', '--target', job.target,
               '--frozen', '--unit-graph', '-Z', 'unstable-options']
    if not job.default_features:
        command.append('--no-default-features')
    for feature in job.features:
        command += ['--features', feature]
    receipt = {'state': 'running', 'command': command, 'environment': environment,
               'cwd': str(job.manifest.parent), 'inputs': [(str(p), sha) for p, sha in job.inputs],
               'cargo_sha256': job.cargo_sha256, 'implementation_sha256': implementation,
               'query_compiler_sha256': job.query_compiler_sha256, 'status': None}

    def save():
        (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

    started = time.monotonic()
    save()
    try:
        with (folder / 'graph.stdout').open('xb') as stdout, (folder / 'graph.stderr').open('xb') as stderr:
            result = subprocess.run(command, cwd=job.manifest.parent, env=environment,
                                    stdout=stdout, stderr=stderr, timeout=120)
        receipt['status'] = result.returncode
        if result.returncode:
            raise ValueError(f'Cargo planning failed with status {result.returncode}')
        with (folder / 'graph.stdout').open('rb') as source:
            plan = parse(source.read(MAX_BYTES + 1), job.target)
        inputs = dict(job.inputs)
        if any(Path(unit.target.src_path) not in inputs for unit in plan.units):
            raise ValueError('planned source is absent from the declared frozen inputs')
        job.validate()
        if any(digest(wrapper.with_name(name)) != sha for name, sha in implementation.items()):
            raise ValueError('planning implementation changed during execution')
        receipt.update(state='planned', units=len(plan.units), roots=plan.roots, order=plan.order,
                       execution=[unit.execution.value for unit in plan.units])
        return plan
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        receipt['seconds'] = time.monotonic() - started
        for name in ('graph.stdout', 'graph.stderr', 'queries.jsonl'):
            path = folder / name
            if path.is_file():
                receipt[name + '_sha256'] = digest(path)
        save()
