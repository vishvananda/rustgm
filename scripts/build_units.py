"""Decode the pinned Cargo unit protocol without compiling Rust source.

Graph rows are invocation-local identities, never crate disambiguators or cache
keys. A valid graph establishes ordering, not successful compilation or proof
that its caller froze the complete package/configuration closure.
"""
from dataclasses import dataclass
from enum import Enum
import json
from pathlib import Path

MAX_BYTES = 16 * 1024 * 1024
MAX_UNITS = 10_000
MAX_EDGES = 100_000


class Execution(Enum):
    TARGET_COMPILE = 'target-compile'
    HOST_COMPILE = 'host-compile'
    HOST_RUN = 'host-run'


@dataclass(frozen=True)
class Dependency:
    index: int
    extern_crate_name: str
    public: bool
    noprelude: bool
    nounused: bool


@dataclass(frozen=True)
class Target:
    kind: tuple[str, ...]
    crate_types: tuple[str, ...]
    name: str
    src_path: str
    edition: str
    doc: bool
    doctest: bool
    test: bool


@dataclass(frozen=True)
class Unit:
    pkg_id: str
    target: Target
    # Preserve the entire profile; a future executor must support every setting.
    profile_json: str
    platform: str | None
    mode: str
    features: tuple[str, ...]
    dependencies: tuple[Dependency, ...]

    @property
    def execution(self):
        if self.mode == 'run-custom-build':
            return Execution.HOST_RUN
        return Execution.HOST_COMPILE if self.platform is None else Execution.TARGET_COMPILE


@dataclass(frozen=True)
class Plan:
    units: tuple[Unit, ...]
    roots: tuple[int, ...]
    order: tuple[int, ...]


def fields(value, required):
    if not isinstance(value, dict) or set(value) != set(required.split()):
        raise ValueError('unexpected fields in pinned Cargo unit protocol')
    return value


def string(value):
    if not isinstance(value, str) or not value or any(ord(ch) < 32 for ch in value):
        raise ValueError('invalid unit graph string')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('invalid unit graph boolean')
    return value


def strings(value, *, empty=True):
    if not isinstance(value, list) or (not empty and not value):
        raise ValueError('invalid unit graph string list')
    result = tuple(string(item) for item in value)
    if len(result) != len(set(result)):
        raise ValueError('duplicate unit graph string')
    return result


def index(value, size):
    if type(value) is not int or not 0 <= value < size:
        raise ValueError('unit graph index outside its declared inventory')
    return value


def target(value):
    fields(value, 'kind crate_types name src_path edition doc doctest test')
    kind = strings(value['kind'], empty=False)
    kinds = {'lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'bin', 'proc-macro', 'custom-build'}
    if not set(kind) <= kinds:
        raise ValueError('target kind outside the bootstrap build profile')
    crate_types = strings(value['crate_types'], empty=False)
    if not set(crate_types) <= kinds - {'custom-build'}:
        raise ValueError('crate type outside the bootstrap build profile')
    source = string(value['src_path'])
    if not Path(source).is_absolute() or value['edition'] not in ('2015', '2018', '2021', '2024'):
        raise ValueError('source path or edition outside the bootstrap build profile')
    return Target(kind, crate_types, string(value['name']), source, value['edition'],
                  *(boolean(value[key]) for key in ('doc', 'doctest', 'test')))


def unit(value, size, selected_target):
    fields(value, 'pkg_id target profile platform mode features dependencies')
    owner = target(value['target'])
    platform = value['platform']
    if platform is not None and platform != selected_target:
        raise ValueError('unit uses an unselected target platform')
    mode = value['mode']
    if mode not in ('build', 'run-custom-build'):
        raise ValueError('unit mode outside the bootstrap build profile')
    if mode == 'run-custom-build' and owner.kind != ('custom-build',):
        raise ValueError('only a build script may be a host execution unit')
    if mode == 'build' and ('custom-build' in owner.kind or 'proc-macro' in owner.kind):
        if platform is not None:
            raise ValueError('build scripts and procedural macros must compile for the host')
    profile = value['profile']
    fields(profile, 'name opt_level lto codegen_backend codegen_units debuginfo '
           'split_debuginfo debug_assertions overflow_checks rpath incremental panic strip'
           + (' rustflags' if isinstance(profile, dict) and 'rustflags' in profile else ''))
    if 'rustflags' in profile:
        flags = profile['rustflags']
        if not isinstance(flags, list) or len(flags) > MAX_EDGES:
            raise ValueError('profile rustflags exceed budget or have invalid shape')
        for flag in flags:
            string(flag)
    dependencies = value['dependencies']
    if not isinstance(dependencies, list) or len(dependencies) > MAX_EDGES:
        raise ValueError('unit dependencies exceed budget or have invalid shape')
    rows = []
    for dependency in dependencies:
        fields(dependency, 'index extern_crate_name public noprelude nounused')
        rows.append(Dependency(index(dependency['index'], size), string(dependency['extern_crate_name']),
                               *(boolean(dependency[key]) for key in ('public', 'noprelude', 'nounused'))))
    if len(rows) != len({(row.index, row.extern_crate_name) for row in rows}):
        raise ValueError('duplicate unit dependency')
    return Unit(string(value['pkg_id']), owner, json.dumps(profile, sort_keys=True, allow_nan=False),
                platform, mode, strings(value['features']), tuple(rows))


def order(units, roots):
    """Dependency-first iterative DFS: each node/edge is visited once."""
    state, result = bytearray(len(units)), []
    for root in roots:
        if state[root]:
            continue
        state[root] = 1
        stack = [(root, iter(units[root].dependencies))]
        while stack:
            current, children = stack[-1]
            edge = next(children, None)
            if edge is None:
                stack.pop()
                state[current] = 2
                result.append(current)
            elif state[edge.index] == 1:
                raise ValueError('cycle in unit dependencies')
            elif state[edge.index] == 0:
                state[edge.index] = 1
                stack.append((edge.index, iter(units[edge.index].dependencies)))
    if len(result) != len(units):
        raise ValueError('unreachable units in the selected graph')
    return tuple(result)


def validate_domains(units):
    for owner in units:
        children = [units[edge.index] for edge in owner.dependencies]
        if owner.execution is Execution.HOST_RUN:
            scripts = [child for child in children if child.execution is Execution.HOST_COMPILE
                       and child.target.kind == ('custom-build',) and child.pkg_id == owner.pkg_id
                       and child.target == owner.target and child.features == owner.features]
            if len(scripts) != 1:
                raise ValueError('build-script execution lacks its exact host compiler unit')
            if any(child is not scripts[0] and child.execution is not Execution.HOST_RUN
                   for child in children):
                raise ValueError('build-script execution has an unexpected compilation dependency')
            continue
        for child in children:
            if child.execution is Execution.HOST_RUN:
                if child.platform != owner.platform:
                    raise ValueError('build-script configuration crosses host/target domains')
            elif child.platform != owner.platform and child.target.kind != ('proc-macro',):
                raise ValueError('ordinary dependency crosses host/target domains')


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON field in unit graph')
        result[key] = value
    return result


def parse(data, selected_target):
    if len(data) > MAX_BYTES:
        raise ValueError('unit graph exceeds byte budget')
    try:
        document = json.loads(data, object_pairs_hook=unique_object)
    except (RecursionError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError('invalid unit graph JSON') from error
    fields(document, 'version units roots')
    if type(document['version']) is not int or document['version'] != 1:
        raise ValueError('unsupported unit graph version')
    raw = document['units']
    if not isinstance(raw, list) or not 0 < len(raw) <= MAX_UNITS:
        raise ValueError('unit inventory is empty, malformed or exceeds budget')
    units = tuple(unit(row, len(raw), selected_target) for row in raw)
    if sum(len(row.dependencies) for row in units) > MAX_EDGES:
        raise ValueError('unit graph exceeds edge budget')
    identities = {(row.pkg_id, row.target, row.profile_json, row.platform, row.mode,
                   frozenset(row.features)) for row in units}
    if len(identities) != len(units):
        raise ValueError('duplicate build unit identity')
    if not isinstance(document['roots'], list) or not document['roots']:
        raise ValueError('unit graph has no selected roots')
    roots = tuple(index(value, len(units)) for value in document['roots'])
    if len(roots) != len(set(roots)):
        raise ValueError('duplicate selected root')
    if any(units[root].execution is not Execution.TARGET_COMPILE for root in roots):
        raise ValueError('selected roots must compile through the target frontend')
    result = order(units, roots)
    validate_domains(units)
    return Plan(units, roots, result)
