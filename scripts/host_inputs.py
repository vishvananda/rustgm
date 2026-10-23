"""Authenticated host graph inputs and ordered script effects, without execution."""
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import tomllib

from build_environment import package_environment
from build_script_output import Kind
from build_units import Execution, MAX_BYTES, parse
import host_profile
from local_package import LocalPackage, validate_layout


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False, default=str).encode()).hexdigest()


def identifier(value):
    value = value.replace('-', '_')
    if not value.isascii() or not value.isidentifier():
        raise ValueError('host crate or dependency name is not an ASCII identifier')
    return value


@dataclass(frozen=True)
class Package:
    snapshot: object
    environment: dict
    links: str | None
    features: tuple[str, ...]
    key: str
    sources: frozenset[str]

    @classmethod
    def load(cls, snapshot):
        snapshot.verify()
        document = tomllib.loads((snapshot.root / 'Cargo.toml').read_text())
        environment = package_environment(snapshot, document)
        links = document['package'].get('links')
        if links is not None:
            identifier(links)
        declared = document.get('features', {})
        suppressed = {value[4:] for values in declared.values() for value in values
                      if isinstance(value, str) and value.startswith('dep:')}
        features = set(declared)
        for table in [document, *document.get('target', {}).values()]:
            for kind in ('dependencies', 'build-dependencies', 'dev-dependencies'):
                features.update(name for name, dependency in table.get(kind, {}).items()
                                if isinstance(dependency, dict) and dependency.get('optional')
                                and name not in suppressed)
        key = fingerprint((snapshot.identity, snapshot.files, snapshot.directories))
        return cls(snapshot, environment, links, tuple(sorted(features)), key,
                   frozenset(str(snapshot.root / name) for name, _, _ in snapshot.files))


def prepare(plan, snapshots, contexts):
    """Verify each package/context once, then bind each unit in constant time."""
    if None not in contexts:
        raise ValueError('host graph requires an explicit host context')
    host_context = contexts[None]
    host = host_context.host
    if host_context.target != host or any(
        c.host != host or c.compiler != host_context.compiler
        or c.compiler_sha256 != host_context.compiler_sha256
        or (platform is not None and c.target != platform)
        for platform, c in contexts.items()
    ):
        raise ValueError('host graph contexts disagree on the pinned host compiler')
    for context in contexts.values():
        context.validate()
    targets = {u.platform for u in plan.units if u.platform is not None}
    if len(targets) != 1 or set(contexts) != {None, *targets}:
        raise ValueError('host graph requires its selected target context')
    wire = json.dumps({'version': 1, 'roots': plan.roots, 'units': [
        {**asdict(unit), 'profile': json.loads(unit.profile_json)} for unit in plan.units]})
    document = json.loads(wire)
    for unit in document['units']:
        del unit['profile_json']
    encoded = json.dumps(document).encode()
    if len(encoded) > MAX_BYTES or parse(encoded, next(iter(targets))) != plan:
        raise ValueError('host graph differs from its validated dependency order')
    packages = {}
    if len(snapshots) > len(plan.units) or sum(len(p.files) for p in snapshots) > 1_000_000:
        raise ValueError('host package inventory exceeds combined count budget')
    validate_layout(tuple(p for p in snapshots if isinstance(p, LocalPackage)))
    roots = sorted(p.root for p in snapshots)
    if any(right.is_relative_to(left) for left, right in zip(roots, roots[1:])):
        raise ValueError('host package roots must not overlap')
    for snapshot in snapshots:
        if snapshot.cargo_id in packages:
            raise ValueError('duplicate host package identity')
        packages[snapshot.cargo_id] = Package.load(snapshot)
    linked = {}
    for package in packages.values():
        if package.links is not None:
            if package.links in linked:
                raise ValueError('duplicate package links identity')
            linked[package.links] = package.snapshot.cargo_id
    for unit in plan.units:
        if unit.execution is Execution.TARGET_COMPILE:
            continue
        package = packages.get(unit.pkg_id)
        if package is None or unit.target.src_path not in package.sources:
            raise ValueError('host unit source lacks its authenticated package')
        host_profile.decode(unit.profile_json)
        if not set(unit.features) <= set(package.features):
            raise ValueError('host unit feature is absent from its manifest')
        if any(',' in feature for feature in unit.features):
            raise ValueError('host feature cannot be represented in CARGO_CFG_FEATURE')
        if any(edge.public or edge.noprelude or edge.nounused for edge in unit.dependencies):
            raise ValueError('host dependency modifiers are not implemented')
        if unit.execution is Execution.HOST_COMPILE and not (
            unit.target.kind == ('custom-build',) and unit.target.crate_types == ('bin',)
            or unit.target.kind == ('lib',) and unit.target.crate_types in (('lib',), ('rlib',))
        ):
            raise ValueError('host graph requires ordinary rlibs or build scripts')
    return packages, contexts


def compile_effects(output):
    arguments = []
    for row in output.directives:
        if row.kind in (Kind.CFG, Kind.CHECK_CFG, Kind.LINK_LIBRARY, Kind.LINK_SEARCH):
            flag = {Kind.CFG: '--cfg', Kind.CHECK_CFG: '--check-cfg',
                    Kind.LINK_LIBRARY: '-l', Kind.LINK_SEARCH: '-L'}[row.kind]
            arguments.extend((flag, row.arguments[0]))
        # Link-arg instructions do not apply to ordinary library/rlib targets.
        # Their ordered original rows remain in the script result for targets.
    return arguments, output.environment()


def metadata(package, output):
    """Only immediate links edges receive metadata; enabled cfgs stay separate."""
    if package.links is None:
        return {}
    prefix = 'DEP_' + identifier(package.links).upper() + '_'
    result, originals = {}, {}
    for row in output.directives:
        if row.kind != Kind.METADATA:
            continue
        key, value = row.arguments
        name = prefix + identifier(key).upper()
        if name in originals and originals[name] != key:
            raise ValueError('distinct metadata keys collide after environment normalization')
        originals[name] = key
        result[name] = value
    return result
