#!/usr/bin/env python3
"""Plan and run frozen target compile units through RustGM, never Cargo or rustc.

Every unit setting is mapped to an exact RustGM argument, checked against
RustGM's fixed course profile, recorded as a lint-only input, or reported as a
typed gap. A unit with any gap fails before a process starts; nothing is skipped
or substituted. Execution stops at the first failing unit and records each
downstream dependent as blocked.
"""
from dataclasses import asdict, dataclass, field
import json
from pathlib import Path
import subprocess
import time

from build_script_output import Kind
from build_units import Execution
from host_inputs import fingerprint, identifier
from host_process import CommandLog, digest
from local_package import LocalPackage

ENVIRONMENT = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}
EDITIONS = {'2021', '2024'}
LIBRARY_TYPES = {'lib', 'rlib'}
# RustGM's fixed course target: Linux x86-64, panic abort, overflow checks on,
# debug assertions off, no optimizer and no debug information.
FIXED_PROFILE = (('opt_level', '0'), ('lto', 'false'), ('codegen_backend', None),
                 ('codegen_units', None), ('debuginfo', 0), ('split_debuginfo', None),
                 ('debug_assertions', False), ('overflow_checks', True), ('rpath', False),
                 ('incremental', False), ('panic', 'abort'))
STRIP = ({'deferred': 'None'}, {'resolved': {'Named': 'debuginfo'}})
# Flags whose only effect is a setting the fixed target already has.
FIXED_RUSTFLAGS = (('-Cpanic=abort',), ('-C', 'panic=abort'))
LINK_KINDS = {Kind.LINK_ARGUMENT, Kind.LINK_LIBRARY, Kind.LINK_SEARCH}
RECORDED_KINDS = {Kind.RERUN_FILE, Kind.RERUN_ENVIRONMENT, Kind.WARNING, Kind.METADATA}
IMPLEMENTATION = ('target_units.py', 'build_units.py', 'build_script_output.py',
                  'host_inputs.py', 'host_process.py')


@dataclass(frozen=True)
class Gap:
    """One planned setting RustGM cannot express."""
    field: str
    value: str
    reason: str


@dataclass(frozen=True)
class TargetPackage:
    """An authenticated package bound to this invocation's graph."""
    identity: str
    key: str
    root: Path
    planned_root: Path
    # None means the package manifest was unavailable: a gap, never a guess.
    environment: tuple[tuple[str, str], ...] | None
    registry: bool
    declared_features: tuple[str, ...] = ()
    lint_arguments: tuple[str, ...] = ()
    snapshot: object = field(default=None, compare=False)

    @classmethod
    def load(cls, package, planned_root=None):
        """Bind a host_inputs.Package; registry sources cap lints as Cargo does."""
        snapshot = package.snapshot
        return cls(json.dumps(snapshot.identity, sort_keys=True), package.key, snapshot.root,
                   Path(planned_root or snapshot.root), tuple(sorted(package.environment.items())),
                   not isinstance(snapshot, LocalPackage), package.features, package.lint_arguments,
                   snapshot)


@dataclass(frozen=True)
class Step:
    unit: int
    package: str
    crate_name: str
    disambiguator: str
    library: bool
    source: str
    arguments: tuple[str, ...]
    environment: tuple[tuple[str, str], ...]
    outputs: tuple[tuple[str, str], ...]
    dependencies: tuple[int, ...]
    gaps: tuple[Gap, ...]
    lint_inputs: tuple[str, ...]
    recorded_directives: tuple[str, ...]


def profile_gaps(unit):
    profile = json.loads(unit.profile_json)
    gaps = [Gap(f'profile.{key}', json.dumps(profile[key]),
                f'RustGM fixes {key} to {json.dumps(value)}')
            for key, value in FIXED_PROFILE if profile[key] != value]
    if profile['name'] not in ('dev', 'release'):
        gaps.append(Gap('profile.name', profile['name'], 'custom profile inheritance is not implemented'))
    if profile['strip'] not in STRIP:
        gaps.append(Gap('profile.strip', json.dumps(profile['strip']), 'RustGM supports only no stripping or debuginfo stripping'))
    flags = tuple(profile.get('rustflags', ()))
    while flags:
        fixed = next((form for form in FIXED_RUSTFLAGS if flags[:len(form)] == form), None)
        if fixed is None:
            gaps.append(Gap('rustflags', flags[0], 'RustGM has no equivalent for this compiler flag'))
            flags = flags[1:]
        else:
            flags = flags[len(fixed):]
    return gaps


def kind_gaps(unit):
    target = unit.target
    if target.kind == ('bin',) and target.crate_types == ('bin',):
        return False, []
    if target.kind == ('lib',) or set(target.kind) <= LIBRARY_TYPES | {'dylib', 'cdylib', 'staticlib'}:
        unsupported = [kind for kind in target.crate_types if kind not in LIBRARY_TYPES]
        return True, [Gap('crate_types', kind, f'RustGM cannot produce a {kind} artifact')
                      for kind in unsupported]
    return False, [Gap('target.kind', ','.join(target.kind), 'RustGM compiles only libraries and binaries')]


class Planner:
    def __init__(self, plan, packages, source_core, scripts):
        self.plan, self.packages, self.scripts = plan, packages, scripts
        self.position = {index: order for order, index in enumerate(plan.order)}
        self.steps = {}
        self.source_core = source_core
        if source_core is not None and sum(
                unit.pkg_id == source_core and unit.execution is Execution.TARGET_COMPILE
                for unit in plan.units) != 1:
            raise ValueError('source core package must own exactly one target unit')

    def closure(self, index):
        """Every target compile unit reachable through ordinary edges."""
        seen, stack = set(), [index]
        while stack:
            for edge in self.plan.units[stack.pop()].dependencies:
                child = self.plan.units[edge.index]
                if child.execution is Execution.TARGET_COMPILE and edge.index not in seen:
                    seen.add(edge.index)
                    stack.append(edge.index)
        return sorted(seen, key=self.position.__getitem__)

    def source(self, unit, package):
        planned = Path(unit.target.src_path)
        if not planned.is_relative_to(package.planned_root):
            raise ValueError('target source lies outside its bound package')
        return package.root / planned.relative_to(package.planned_root)

    def script(self, unit, gaps, lint_inputs, recorded):
        """Consume the unit's own build-script output; return arguments, environment, key."""
        runs = [edge.index for edge in unit.dependencies
                if self.plan.units[edge.index].execution is Execution.HOST_RUN]
        if not runs:
            return [], {}, None
        script_unit = self.plan.units[runs[0]]
        if len(runs) != 1 or script_unit.pkg_id != unit.pkg_id or script_unit.features != unit.features:
            raise ValueError('target compilation lacks its exact own build-script execution')
        generated = self.scripts.get(runs[0])
        if generated is None:
            gaps.append(Gap('build-script.output', str(runs[0]),
                            'build-script output was not supplied; execute the host graph first'))
            return [], {}, None
        arguments, environment = [], {'OUT_DIR': str(generated.folder)}
        for row in generated.output.directives:
            if row.kind == Kind.CFG:
                arguments += ['--cfg', row.arguments[0]]
            elif row.kind == Kind.ENVIRONMENT:
                environment[row.arguments[0]] = row.arguments[1]
            elif row.kind == Kind.CHECK_CFG:
                lint_inputs.append(f'check-cfg {row.arguments[0]}')
            elif row.kind in LINK_KINDS:
                gaps.append(Gap(f'build-script.{row.kind}', ' '.join(row.arguments),
                                'RustGM has no native link input for build-script directives'))
            elif row.kind in RECORDED_KINDS:
                recorded.append(f'{row.kind} {" ".join(row.arguments)}')
            else:
                gaps.append(Gap(f'build-script.{row.kind}', ' '.join(row.arguments),
                                'unknown build-script directive kind'))
        return arguments, environment, generated.key

    def dependencies(self, index, unit, gaps, lint_inputs):
        """Direct library edges are aliases; the rest of the closure is inventory only."""
        direct, arguments = set(), []
        for edge in unit.dependencies:
            child = self.plan.units[edge.index]
            if child.execution is Execution.HOST_RUN:
                continue
            alias = identifier(edge.extern_crate_name)
            if child.execution is not Execution.TARGET_COMPILE:
                gaps.append(Gap('dependency.proc-macro', alias, 'procedural macro execution is not implemented'))
                continue
            if edge.noprelude:
                gaps.append(Gap('dependency.noprelude', alias, 'RustGM has no extern prelude modifier'))
            if edge.public:
                lint_inputs.append(f'public dependency {alias}')
            if edge.nounused:
                lint_inputs.append(f'nounused dependency {alias}')
            dependency = self.steps[edge.index]
            if not dependency.library:
                gaps.append(Gap('dependency.kind', alias, 'dependency has no library artifact'))
                continue
            direct.add(edge.index)
            arguments += ['--extern', f'{alias}={dict(dependency.outputs)["metadata"]}']
        closure = self.closure(index)
        for child in closure:
            if child not in direct and self.steps[child].library:
                arguments += ['--dependency-metadata', dict(self.steps[child].outputs)['metadata']]
        return arguments, closure

    def mode(self, unit, name, disambiguator, closure):
        """The selected sysroot core provides itself; its dependents are its clients."""
        if self.source_core is None:
            return []
        if unit.pkg_id == self.source_core:
            return ['--source-core', f'{name}@{disambiguator}']
        core = next((child for child in closure if self.plan.units[child].pkg_id == self.source_core), None)
        if core is None:
            return []
        step = self.steps[core]
        return ['--source-client', f'{step.crate_name}@{step.disambiguator}']

    def unit(self, index, folder):
        unit = self.plan.units[index]
        package = self.packages.get(unit.pkg_id)
        if package is None:
            raise ValueError('target unit lacks its bound package')
        gaps, lint_inputs, recorded = profile_gaps(unit), [], []
        library, kinds = kind_gaps(unit)
        gaps += kinds
        if unit.target.edition not in EDITIONS:
            gaps.append(Gap('edition', unit.target.edition, 'RustGM supports only Rust 2021 and 2024'))
        if package.environment is None:
            gaps.append(Gap('package.environment', package.identity,
                            'package manifest unavailable; its Cargo environment cannot be reproduced'))
        name = identifier(unit.target.name)
        source = self.source(unit, package)
        script_arguments, script_environment, script_key = self.script(unit, gaps, lint_inputs, recorded)
        dependency_arguments, closure = self.dependencies(index, unit, gaps, lint_inputs)
        # Cargo's own cfg expectations and manifest lint levels only select
        # diagnostics. RustGM implements neither, so they are recorded per unit.
        if package.lint_arguments:
            lint_inputs.append('manifest lints ' + ' '.join(package.lint_arguments))
        values = ','.join(json.dumps(feature) for feature in package.declared_features)
        lint_inputs += [f'check-cfg cfg(feature,values({values}))', 'check-cfg cfg(docsrs,test)']
        key = fingerprint((package.identity, package.key, name, unit.target.kind, unit.target.crate_types,
                           unit.target.edition, sorted(unit.features), unit.profile_json,
                           [(self.steps[child].crate_name, self.steps[child].disambiguator) for child in closure],
                           script_key))
        disambiguator = 'h' + key[:20]
        unit_folder = folder / f'unit-{index}'
        outputs = [('lowir', str(unit_folder / f'{name}.lowir'))]
        arguments = ['--emit-lowir', f'--edition={unit.target.edition}',
                     '--crate-type=lib' if library else '--crate-type=bin',
                     '--crate-name', name, '--crate-disambiguator', disambiguator,
                     '--ub-checks=no', *self.mode(unit, name, disambiguator, closure)]
        if package.registry:
            arguments.append('--cap-lints=allow')
        for feature in sorted(unit.features):
            arguments += ['--cfg', 'feature=' + json.dumps(feature)]
        arguments += script_arguments + dependency_arguments
        if library:
            outputs.append(('metadata', str(unit_folder / f'{name}.rgcrate')))
            arguments += ['--metadata-output', outputs[1][1]]
        arguments += ['-o', outputs[0][1], str(source)]
        environment = {**ENVIRONMENT, **dict(package.environment or ()), 'CARGO_CRATE_NAME': name,
                       **script_environment}
        return Step(index, package.identity, name, disambiguator, library, str(source), tuple(arguments),
                    tuple(sorted(environment.items())), tuple(outputs), tuple(closure), tuple(gaps),
                    tuple(lint_inputs), tuple(recorded))


def plan_units(plan, packages, folder, *, source_core=None, scripts=None):
    """Plan every target compile unit in dependency order; gaps are data."""
    planner = Planner(plan, packages, source_core, scripts or {})
    for index in plan.order:
        if plan.units[index].execution is Execution.TARGET_COMPILE:
            planner.steps[index] = planner.unit(index, Path(folder).absolute())
    if not planner.steps:
        raise ValueError('graph selects no target compile unit; a target build cannot pass empty')
    return tuple(planner.steps.values())


class Run:
    def __init__(self, steps, rustgm, rustgm_sha256, folder, graph):
        self.steps, self.rustgm, self.rustgm_sha256, self.folder = steps, Path(rustgm), rustgm_sha256, folder
        self.implementation = {name: digest(Path(__file__).with_name(name)) for name in IMPLEMENTATION}
        self.receipt = {'state': 'running', 'analysis': 'RustGM target units; no Cargo or rustc target compile',
                        'graph': graph, 'rustgm': {'path': str(rustgm), 'sha256': rustgm_sha256},
                        'implementation_sha256': self.implementation,
                        'steps': [asdict(step) for step in steps], 'completed': [], 'failure': None,
                        'blocked': [], 'not_run': [], 'commands': []}

    def save(self):
        (self.folder / 'receipt.json').write_text(json.dumps(self.receipt, indent=2, default=str) + '\n')

    def fail(self, step, kind, detail):
        self.receipt['failure'] = {'unit': step.unit, 'crate': step.crate_name, 'kind': kind, 'detail': detail}
        later = self.steps[self.steps.index(step) + 1:]
        failed = {step.unit}
        for other in later:
            if failed & set(other.dependencies):
                failed.add(other.unit)
                self.receipt['blocked'].append({'unit': other.unit, 'crate': other.crate_name,
                                                'blocked_by': step.unit})
            else:
                self.receipt['not_run'].append({'unit': other.unit, 'crate': other.crate_name})

    def compile(self, step):
        unit_folder = self.folder / f'unit-{step.unit}'
        unit_folder.mkdir()
        commands = self.receipt['commands']
        # Every planned path is absolute; the unit's own folder is a stable cwd.
        log = CommandLog(unit_folder, unit_folder, commands, self.save)
        status = log.run('compile', [self.rustgm, *step.arguments], dict(step.environment))
        outputs = {}
        for name, path in step.outputs:
            path = Path(path)
            if status == 0 and path.is_file() and not path.is_symlink() and path.stat().st_size:
                outputs[name] = {'path': str(path), 'sha256': digest(path), 'bytes': path.stat().st_size}
            elif path.exists():
                # A failed or incomplete output is never consumed or published.
                path.rename(path.with_name(path.name + '.rejected'))
        return status, outputs

    def execute(self, packages):
        started = time.monotonic()
        try:
            if digest(self.rustgm) != self.rustgm_sha256:
                raise ValueError('RustGM binary differs from its declared pin')
            for step in self.steps:
                if step.gaps:
                    self.fail(step, 'unsupported', [asdict(gap) for gap in step.gaps])
                    break
                status, outputs = self.compile(step)
                if status != 0:
                    self.fail(step, 'compile', {'status': status})
                    break
                if set(outputs) != {name for name, _ in step.outputs}:
                    self.fail(step, 'output', {'missing': sorted({n for n, _ in step.outputs} - set(outputs))})
                    break
                self.receipt['completed'].append({'unit': step.unit, 'crate': step.crate_name, 'outputs': outputs})
                self.save()
            for package in packages.values():
                if package.snapshot is not None:
                    package.snapshot.verify()
            if digest(self.rustgm) != self.rustgm_sha256 or any(
                    digest(Path(__file__).with_name(name)) != sha for name, sha in self.implementation.items()):
                raise ValueError('RustGM or executor implementation changed during execution')
            self.receipt['state'] = 'failed' if self.receipt['failure'] else 'complete'
        except (OSError, ValueError, subprocess.TimeoutExpired) as error:
            self.receipt.update(state='infrastructure failure', error=str(error))
        finally:
            self.receipt['seconds'] = time.monotonic() - started
            self.save()
        return self.receipt


def execute(steps, packages, rustgm, rustgm_sha256, folder, graph):
    """Run planned steps in order; a receipt is complete only if every unit built."""
    folder = Path(folder).absolute()
    if folder.exists() or folder.is_symlink():
        raise ValueError('target unit output already exists')
    if any(folder.is_relative_to(package.root) for package in packages.values()):
        raise ValueError('target unit output must be outside package sources')
    if any(Path(path).parent != folder / f'unit-{step.unit}' for step in steps for _, path in step.outputs):
        raise ValueError('target unit outputs were planned for another folder')
    folder.mkdir(parents=True)
    return Run(steps, rustgm, rustgm_sha256, folder, graph).execute(packages)
