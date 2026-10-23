"""Execute a frozen host dependency DAG; target compilation remains RustGM's job."""
from dataclasses import asdict, dataclass
import json
from pathlib import Path
import subprocess
import time

from build_script_output import MAX_BYTES, Output, parse
from build_units import Execution
from host_build_script import generated_files
from host_inputs import compile_effects, fingerprint, identifier, metadata, prepare
from host_process import CommandLog, digest
import host_profile

MAX_GENERATED_FILES = 100_000
MAX_GENERATED_BYTES = 256 * 1024 * 1024
MAX_ARTIFACT_BYTES = 4 * 1024 * 1024 * 1024
MAX_SCRIPT_OUTPUT_BYTES = 16 * 1024 * 1024


@dataclass(frozen=True)
class Artifact:
    path: Path
    sha256: str
    key: str


@dataclass(frozen=True)
class Generated:
    output: Output
    folder: Path
    files: dict[str, str]
    metadata: dict[str, str]
    key: str


class Build:
    def __init__(self, plan, packages, contexts, folder):
        self.plan, self.packages, self.contexts, self.folder = plan, packages, contexts, folder
        self.host = contexts[None].host
        self.artifacts, self.generated = {}, {}
        self.command_groups = []
        self.generated_count = self.generated_bytes = self.artifact_bytes = 0
        self.script_output_bytes = 0
        self.library_folder = folder / 'libraries'
        self.library_folder.mkdir()
        self.receipt = {'state': 'running', 'units': [asdict(unit) for unit in plan.units],
                        'order': plan.order, 'commands': [], 'completed': [],
                        'packages': [dict(package.snapshot.identity, input_sha256=package.key)
                                     for package in packages.values()],
                        'contexts': [{'platform': platform, 'context': asdict(context)}
                                     for platform, context in contexts.items()]}
        self.save()

    def save(self):
        self.receipt['commands'] = [row for group in self.command_groups for row in group]
        (self.folder / 'receipt.json').write_text(json.dumps(self.receipt, indent=2, default=str) + '\n')

    def command_log(self, folder, package):
        # Each task writes only its own bounded command receipt. Rewriting the
        # growing graph after every process would multiply work across all nodes.
        commands = []
        self.command_groups.append(commands)

        def save():
            (folder / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')

        return CommandLog(folder, package.snapshot.root, commands, save)

    def completed(self, index, result):
        row = {'unit': index, **result}
        self.receipt['completed'].append(row)
        with (self.folder / 'progress.jsonl').open('a') as stream:
            stream.write(json.dumps(row, default=str) + '\n')

    def compile(self, index, unit, package, folder):
        script = unit.target.kind == ('custom-build',)
        dependencies, configuration = [], None
        aliases = set()
        for edge in unit.dependencies:
            dependency = self.plan.units[edge.index]
            if dependency.execution is Execution.HOST_RUN:
                if (configuration is not None or dependency.pkg_id != unit.pkg_id
                        or dependency.features != unit.features):
                    raise ValueError('host compilation lacks its exact own script output')
                configuration = self.generated[edge.index]
            else:
                alias = identifier(edge.extern_crate_name)
                if alias in aliases or dependency.target.kind != ('lib',):
                    raise ValueError('host library dependency has duplicate alias or wrong kind')
                aliases.add(alias)
                dependencies.append((alias, self.artifacts[edge.index]))
        if script and configuration is not None:
            raise ValueError('build script cannot consume its own execution output')
        key = fingerprint((package.key, asdict(unit.target), unit.profile_json, unit.features,
                           self.contexts[None].compiler_sha256,
                           [(alias, artifact.key) for alias, artifact in dependencies],
                           None if configuration is None else configuration.key))
        name = identifier(unit.target.name)
        output = folder / 'build-script' if script else self.library_folder / f'lib{name}-{key}.rlib'
        context = self.contexts[None]
        command = [context.compiler, f'--edition={unit.target.edition}', '--crate-name', name,
                   '--crate-type', 'bin' if script else 'rlib', unit.target.src_path, '-o', output,
                   '-C', f'metadata={key}', *host_profile.arguments(unit.profile_json)]
        values = ','.join(json.dumps(feature, ensure_ascii=False) for feature in package.features)
        command += ['--check-cfg', f'cfg(feature,values({values}))', '--check-cfg', 'cfg(docsrs,test)']
        for feature in unit.features:
            command += ['--cfg', 'feature=' + json.dumps(feature, ensure_ascii=False)]
        environment = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', **package.environment,
                       'CARGO_CRATE_NAME': name}
        if configuration is not None:
            flags, variables = compile_effects(configuration.output)
            command += flags
            environment.update(variables)
            environment['OUT_DIR'] = str(configuration.folder)
        for alias, artifact in dependencies:
            command += ['--extern', f'{alias}={artifact.path}']
        if dependencies:
            command += ['-L', f'dependency={self.library_folder}']
        status = self.command_log(folder, package).run('compile', command, environment)
        if status:
            raise ValueError(f'host compilation {index} failed with status {status}')
        self.artifact_bytes += output.stat().st_size
        if self.artifact_bytes > MAX_ARTIFACT_BYTES:
            raise ValueError('host artifacts exceed combined byte budget')
        self.artifacts[index] = Artifact(output, digest(output), key)

    def run(self, index, unit, package, folder):
        compiled, variables, edges = None, {}, []
        for edge in unit.dependencies:
            dependency = self.plan.units[edge.index]
            if dependency.execution is Execution.HOST_COMPILE:
                if compiled is not None or dependency.target != unit.target:
                    raise ValueError('script execution has no unique exact compiled artifact')
                compiled = self.artifacts[edge.index]
            else:
                result = self.generated[edge.index]
                if any(name in variables for name in result.metadata):
                    raise ValueError('links metadata from distinct dependencies collides')
                variables.update(result.metadata)
                edges.append(result.key)
        if compiled is None:
            raise ValueError('script execution has no compiled artifact')
        generated = folder / 'generated'
        generated.mkdir()
        context = self.contexts[unit.platform]
        wrapper = Path(__file__).with_name('cargo_query.py').absolute()
        queries = folder / 'queries.jsonl'
        environment = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', **package.environment,
                       **context.cfg_environment(), **host_profile.environment(unit.profile_json),
                       **variables, 'OUT_DIR': str(generated), 'HOST': self.host,
                       'TARGET': context.target, 'NUM_JOBS': '2', 'CARGO_ENCODED_RUSTFLAGS': '',
                       'CARGO_CFG_FEATURE': ','.join(unit.features), 'RUSTC': str(wrapper),
                       'RUSTGM_PLAN_QUERY_COMPILER': str(context.compiler),
                       'RUSTGM_PLAN_QUERY_SHA256': context.compiler_sha256,
                       'RUSTGM_PLAN_TARGET': context.target, 'RUSTGM_PLAN_QUERY_LOG': str(queries)}
        if package.links is not None:
            environment['CARGO_MANIFEST_LINKS'] = package.links
        for feature in unit.features:
            environment['CARGO_FEATURE_' + feature.upper().replace('-', '_')] = '1'
        status = self.command_log(folder, package).run('execute', [compiled.path], environment)
        with (folder / 'execute.stdout').open('rb') as stream:
            data = stream.read(MAX_BYTES + 1)
        self.script_output_bytes += len(data)
        if self.script_output_bytes > MAX_SCRIPT_OUTPUT_BYTES:
            raise ValueError('host script output exceeds combined graph budget')
        output = parse(data, status)
        query_data = b''
        if queries.exists():
            with queries.open('rb') as stream:
                query_data = stream.read(MAX_BYTES + 1)
        self.script_output_bytes += len(query_data)
        if len(query_data) > MAX_BYTES or self.script_output_bytes > MAX_SCRIPT_OUTPUT_BYTES:
            raise ValueError('host compiler queries exceed combined graph budget')
        observed = [json.loads(line) for line in query_data.splitlines()]
        if any(not isinstance(row, dict) or row.get('state') != 'query'
               or type(row.get('status')) is not int or row['status'] != 0 for row in observed):
            raise ValueError('build script attempted an unsupported or failed compiler query')
        files = generated_files(generated)
        self.generated_count += len(files)
        self.generated_bytes += sum((generated / name).stat().st_size for name in files)
        if self.generated_count > MAX_GENERATED_FILES or self.generated_bytes > MAX_GENERATED_BYTES:
            raise ValueError('generated host inputs exceed combined graph budget')
        key = fingerprint((compiled.key, unit.profile_json, asdict(context), edges,
                           [asdict(row) for row in output.directives], files))
        self.generated[index] = Generated(output, generated, files, metadata(package, output), key)

    def finish(self):
        """Validate the shared closure once before publishing any graph result."""
        for package in self.packages.values():
            package.snapshot.verify()
        for context in self.contexts.values():
            context.validate()
        for artifact in self.artifacts.values():
            if artifact.path.is_symlink() or digest(artifact.path) != artifact.sha256:
                raise ValueError('host artifact changed during graph execution')
        if set(self.library_folder.iterdir()) != {
            artifact.path for artifact in self.artifacts.values()
            if artifact.path.parent == self.library_folder
        }:
            raise ValueError('unexpected file in shared host library inventory')
        for result in self.generated.values():
            if generated_files(result.folder) != result.files:
                raise ValueError('generated script input changed during graph execution')


def execute(plan, snapshots, contexts, folder):
    """Build each selected host node once, retaining per-run target context.

    This is invocation-local sharing. No persistent cache or successful target
    compilation is inferred from the graph or its completed host artifacts.
    """
    packages, contexts = prepare(plan, snapshots, contexts)
    folder = folder.absolute()
    if folder.exists() or folder.is_symlink():
        raise ValueError('host graph output already exists')
    folder.mkdir(parents=True)
    implementation = {name: digest(Path(__file__).with_name(name)) for name in (
        'host_graph.py', 'host_inputs.py', 'host_process.py', 'host_build_script.py',
        'build_environment.py', 'build_script_output.py', 'host_profile.py',
        'build_units.py', 'cargo_query.py', 'package_snapshot.py')}
    build = Build(plan, packages, contexts, folder)
    build.receipt['implementation_sha256'] = implementation
    build.save()
    started = time.monotonic()
    try:
        for index in plan.order:
            unit = plan.units[index]
            if unit.execution is Execution.TARGET_COMPILE:
                continue
            task = folder / f'unit-{index}'
            task.mkdir()
            package = packages[unit.pkg_id]
            if unit.execution is Execution.HOST_COMPILE:
                build.compile(index, unit, package, task)
                result = {'artifact': asdict(build.artifacts[index])}
            else:
                build.run(index, unit, package, task)
                result = {'generated': asdict(build.generated[index])}
            build.completed(index, result)
        build.finish()
        if any(digest(Path(__file__).with_name(name)) != sha for name, sha in implementation.items()):
            raise ValueError('host graph implementation changed during execution')
        build.receipt['state'] = 'complete'
        return build.generated
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        build.receipt.update(state='failed', error=str(error))
        raise
    finally:
        build.receipt['seconds'] = time.monotonic() - started
        build.receipt['retained_output_bytes'] = build.generated_bytes + build.artifact_bytes
        build.save()
