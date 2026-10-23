"""Explicit package and target-context environment for host script execution."""
from dataclasses import dataclass
from pathlib import Path
import re
import tomllib

from host_build_script import digest


def package_environment(package):
    manifest = package.root / 'Cargo.toml'
    document = tomllib.loads(manifest.read_text())
    if 'lints' in document or 'cargo-features' in document:
        raise ValueError('build-script manifest lint/unstable policy is not implemented')
    data = document['package']
    version = re.fullmatch(r'(\d+)\.(\d+)\.(\d+)(?:-([^+]+))?(?:\+.+)?', package.version)
    if version is None or data.get('name') != package.name or data.get('version') != package.version:
        raise ValueError('build-script package version/manifest identity mismatch')
    authors = data.get('authors', [])
    if not isinstance(authors, list) or any(not isinstance(author, str) for author in authors):
        raise ValueError('package authors must be strings')
    environment = {'CARGO_MANIFEST_DIR': str(package.root), 'CARGO_MANIFEST_PATH': str(manifest),
                   'CARGO_PKG_NAME': package.name, 'CARGO_PKG_VERSION': package.version,
                   'CARGO_PKG_AUTHORS': ':'.join(authors)}
    for key, value in zip(('MAJOR', 'MINOR', 'PATCH', 'PRE'), version.groups()):
        environment['CARGO_PKG_VERSION_' + key] = value or ''
    for key in ('description', 'homepage', 'license', 'license-file', 'readme', 'repository', 'rust-version'):
        value = data.get(key, '')
        if value is False and key == 'readme':
            value = ''
        if not isinstance(value, str):
            raise ValueError('unresolved package environment value')
        environment['CARGO_PKG_' + key.upper().replace('-', '_')] = value
    return environment


@dataclass(frozen=True)
class Context:
    host: str
    target: str
    compiler: Path
    compiler_sha256: str
    # These are explicitly frozen build-script facts, not implicit rustc target
    # capability authorization for the RustGM frontend.
    cfg: tuple[tuple[str, tuple[str, ...]], ...]
    inputs: tuple[tuple[Path, str], ...]

    def validate(self):
        for triple in (self.host, self.target):
            if not re.fullmatch(r'[A-Za-z0-9_-]+', triple):
                raise ValueError('build-script context requires explicit triples')
        pins = ((self.compiler, self.compiler_sha256), *self.inputs)
        if not self.inputs or len({path for path, _ in pins}) != len(pins):
            raise ValueError('build-script configuration requires unique explicit input pins')
        for path, sha in pins:
            if not path.is_absolute() or path.is_symlink() or digest(path) != sha:
                raise ValueError('build-script context input differs from its pin')
        self.cfg_environment()

    def cfg_environment(self):
        if len(self.cfg) > 1024 or sum(len(values) for _, values in self.cfg) > 4096:
            raise ValueError('build-script cfg facts exceed count budget')
        result = {}
        for key, values in self.cfg:
            name = 'CARGO_CFG_' + key.upper()
            if (not re.fullmatch(r'[a-z_][a-z_0-9]*', key) or name in result or key == 'feature'
                    or len(values) != len(set(values))
                    or any(',' in value or any(ord(char) < 32 for char in value) for value in values)):
                raise ValueError('invalid, ambiguous or duplicate build-script cfg fact')
            result[name] = ','.join(values)
        if sum(len(key) + len(value) for key, value in result.items()) > 256 * 1024:
            raise ValueError('build-script cfg facts exceed byte budget')
        return result
