"""Execute one explicitly configured host task and retain its ordered outputs.

The crate planner supplies this task; manifest resolution and target compilation
are separate responsibilities. There is no compiler discovery or target fallback.
"""
from dataclasses import asdict, dataclass
import json
import os
from pathlib import Path
import subprocess

from build_script_output import MAX_BYTES, parse
import host_profile
from host_process import CommandLog, digest

MAX_FILES = 100_000
MAX_GENERATED_BYTES = 256 * 1024 * 1024


@dataclass(frozen=True)
class Job:
    package: Path
    source: Path
    edition: str
    compiler: Path
    compiler_sha256: str
    inputs: dict[str, str]
    compile_environment: dict[str, str]
    environment: dict[str, str]
    features: tuple[str, ...] = ()
    # Explicit already built host libraries. Target artifacts never enter here.
    dependencies: tuple[tuple[str, Path, str], ...] = ()
    profile_json: str | None = None

    def validate(self):
        if self.profile_json is not None:
            host_profile.decode(self.profile_json)
        if not self.package.is_absolute() or not self.source.is_absolute():
            raise ValueError('host task requires absolute package/source paths')
        if self.edition not in ('2021', '2024'):
            raise ValueError('host task edition is outside the bootstrap profile')
        if not self.compiler.is_absolute() or digest(self.compiler) != self.compiler_sha256:
            raise ValueError('host compiler differs from its explicit pin')
        relative = self.source.relative_to(self.package).as_posix()
        if relative not in self.inputs:
            raise ValueError('host source is absent from the frozen inputs')
        for name, expected in self.inputs.items():
            path = self.package / name
            if path.is_symlink() or not path.resolve().is_relative_to(self.package.resolve()):
                raise ValueError('host input escapes its package')
            if digest(path) != expected:
                raise ValueError(f'host input changed: {name}')
        aliases = set()
        for alias, path, expected in self.dependencies:
            if not alias.isascii() or not alias.isidentifier() or alias in aliases:
                raise ValueError('invalid or duplicate host dependency alias')
            aliases.add(alias)
            if not path.is_absolute() or digest(path) != expected:
                raise ValueError('host dependency differs from its explicit pin')
        if len(self.features) != len(set(self.features)):
            raise ValueError('duplicate resolved host feature')
        if any(not value or any(ord(ch) < 32 for ch in value) for value in self.features):
            raise ValueError('invalid resolved host feature')
        for environment in (self.compile_environment, self.environment):
            if any(not key or '=' in key or '\0' in key or '\0' in value
                   for key, value in environment.items()):
                raise ValueError('invalid explicit host environment')
            if 'OUT_DIR' in environment:
                raise ValueError('host task owns its generated output directory')


def generated_files(folder):
    """Hash generated inputs once, rejecting symlinks and bounded growth."""
    if folder.is_symlink() or not folder.is_dir():
        raise ValueError('generated build-script root is not a real directory')
    result, total, directories = {}, 0, 0
    for directory, dirs, files in os.walk(folder, followlinks=False):
        dirs.sort()
        directories += 1
        if directories > MAX_FILES:
            raise ValueError('generated build-script directories exceed budget')
        parent = Path(directory)
        if any((parent / name).is_symlink() for name in dirs):
            raise ValueError('generated build-script directory is a symlink')
        for name in sorted(files):
            path = parent / name
            if path.is_symlink() or not path.is_file():
                raise ValueError('generated build-script input is not a regular file')
            total += path.stat().st_size
            if len(result) >= MAX_FILES or total > MAX_GENERATED_BYTES:
                raise ValueError('generated build-script inputs exceed budget')
            result[path.relative_to(folder).as_posix()] = digest(path)
    return dict(sorted(result.items()))


def execute(job, folder):
    """Run under the repository resource/storage guard, using a fresh output root.

    This primitive does not cache: its caller must close the complete dependency,
    environment and rerun-input graph before enabling persistent reuse.
    """
    job.validate()
    folder = folder.absolute()
    if folder.exists() or folder.is_symlink():
        raise ValueError('host build-script evidence directory already exists')
    folder.mkdir(parents=True)
    generated = folder / 'generated'
    generated.mkdir()
    commands = []
    receipt = {
        'package': str(job.package), 'source': str(job.source), 'edition': job.edition,
        'compiler': str(job.compiler), 'compiler_sha256': job.compiler_sha256,
        'inputs': job.inputs, 'features': list(job.features),
        'profile': None if job.profile_json is None else json.loads(job.profile_json),
        'dependencies': [(alias, str(path), sha) for alias, path, sha in job.dependencies],
        'commands': commands, 'state': 'running',
    }

    def save():
        (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

    run = CommandLog(folder, job.package, commands, save).run

    save()
    try:
        binary = folder / 'build-script'
        command = [job.compiler, f'--edition={job.edition}', '--crate-name',
                   'build_script_build', job.source, '-o', binary]
        if job.profile_json is not None:
            command += host_profile.arguments(job.profile_json)
        for feature in sorted(job.features):
            command += ['--cfg', 'feature=' + json.dumps(feature, ensure_ascii=False)]
        for alias, path, _ in sorted(job.dependencies):
            command += ['--extern', f'{alias}={path}']
        for path in sorted({path.parent for _, path, _ in job.dependencies}):
            command += ['-L', f'dependency={path}']
        status = run('compile', command, job.compile_environment)
        if status:
            raise ValueError(f'host build-script compilation failed with status {status}')
        receipt['executable_sha256'] = digest(binary)
        environment = {**job.environment, 'OUT_DIR': str(generated)}
        for feature in job.features:
            environment['CARGO_FEATURE_' + feature.upper().replace('-', '_')] = '1'
        status = run('execute', [binary], environment)
        with (folder / 'execute.stdout').open('rb') as stream:
            parsed = parse(stream.read(MAX_BYTES + 1), status)
        job.validate()
        receipt['generated'] = generated_files(generated)
        receipt['directives'] = [asdict(row) for row in parsed.directives]
        receipt['state'] = 'complete'
        save()
        return parsed
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        receipt.update(state='failed', error=str(error))
        save()
        raise
