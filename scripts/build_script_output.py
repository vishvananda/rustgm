"""Ordered Cargo build-script output, kept outside Rust frontend phases.

This consumes the host protocol only. Configuration expectations never enable a
cfg, and link directives remain ordered until the selected target consumes them.
"""
from dataclasses import dataclass
from enum import StrEnum

MAX_BYTES = 1024 * 1024
MAX_LINE = 64 * 1024
MAX_DIRECTIVES = 4096


class Kind(StrEnum):
    CFG = 'cfg'
    CHECK_CFG = 'check-cfg'
    ENVIRONMENT = 'environment'
    RERUN_FILE = 'rerun-file'
    RERUN_ENVIRONMENT = 'rerun-environment'
    LINK_ARGUMENT = 'link-argument'
    LINK_LIBRARY = 'link-library'
    LINK_SEARCH = 'link-search'
    WARNING = 'warning'
    METADATA = 'metadata'


@dataclass(frozen=True)
class Directive:
    kind: Kind
    arguments: tuple[str, ...]
    line: int


@dataclass(frozen=True)
class Output:
    directives: tuple[Directive, ...]

    def cfg_arguments(self):
        """Only enabled cfgs become frontend options; check-cfg stays separate."""
        return tuple(part for row in self.directives if row.kind == Kind.CFG
                     for part in ('--cfg', row.arguments[0]))

    def environment(self):
        """Repeated assignments follow emission order, preserving embedded '='."""
        return dict(row.arguments for row in self.directives if row.kind == Kind.ENVIRONMENT)


SIMPLE = {
    'rustc-cfg': Kind.CFG,
    'rustc-check-cfg': Kind.CHECK_CFG,
    'rerun-if-changed': Kind.RERUN_FILE,
    'rerun-if-env-changed': Kind.RERUN_ENVIRONMENT,
    'rustc-link-lib': Kind.LINK_LIBRARY,
    'rustc-link-search': Kind.LINK_SEARCH,
    'warning': Kind.WARNING,
}
LINK_TARGETS = {
    'rustc-link-arg': 'all-supported',
    'rustc-link-arg-cdylib': 'cdylib',
    'rustc-cdylib-link-arg': 'cdylib',
    'rustc-link-arg-bins': 'bins',
    'rustc-link-arg-tests': 'tests',
    'rustc-link-arg-examples': 'examples',
    'rustc-link-arg-benches': 'benches',
}


def pair(value, label):
    key, separator, rest = value.partition('=')
    if not separator or not key:
        raise ValueError(f'{label} requires KEY=VALUE')
    return key, rest


def flags(value, line, remaining):
    """Cargo's space-separated -l/-L vocabulary is not a shell command."""
    parts = iter(value.split())
    result = []
    for part in parts:
        if part[:2] not in ('-l', '-L'):
            raise ValueError('rustc-flags permits only -l and -L')
        argument = part[2:] if len(part) > 2 else next(parts, '')
        if not argument:
            raise ValueError('rustc-flags is missing a library or search path')
        if len(result) >= remaining:
            raise ValueError('build-script directives exceed count budget')
        kind = Kind.LINK_LIBRARY if part[:2] == '-l' else Kind.LINK_SEARCH
        result.append(Directive(kind, (argument,), line))
    return result


def instruction(key, value, line, modern, remaining):
    if key in SIMPLE:
        return [Directive(SIMPLE[key], (value,), line)]
    if key in LINK_TARGETS:
        return [Directive(Kind.LINK_ARGUMENT, (LINK_TARGETS[key], value), line)]
    if key == 'rustc-link-arg-bin':
        name, argument = pair(value, key)
        if not argument:
            raise ValueError('rustc-link-arg-bin requires a linker argument')
        return [Directive(Kind.LINK_ARGUMENT, ('bin', name, argument), line)]
    if key in ('rustc-env', 'metadata'):
        values = pair(value, key)
        kind = Kind.ENVIRONMENT if key == 'rustc-env' else Kind.METADATA
        return [Directive(kind, values, line)]
    if key == 'rustc-flags':
        return flags(value, line, remaining)
    if key == 'error':
        raise ValueError(f'build script reported an error: {value}')
    if not modern:
        # Cargo's legacy unknown keys are links metadata, not ignored flags.
        return [Directive(Kind.METADATA, (key, value), line)]
    raise ValueError(f'unsupported build-script directive: {key}')


def parse(stdout, status):
    """Fail closed on command failure, malformed directives and bounded output.

    Ordinary stdout is retained by the caller and is not interpreted here. A
    signal, panic or infrastructure status cannot become language rejection.
    """
    if status != 0:
        raise ValueError(f'host build script failed with status {status}')
    if len(stdout) > MAX_BYTES:
        raise ValueError('build-script output exceeds byte budget')
    rows = []
    for line, raw in enumerate(stdout.split(b'\n'), 1):
        if raw.endswith(b'\r'):
            raw = raw[:-1]
        modern = raw.startswith(b'cargo::')
        if not modern and not raw.startswith(b'cargo:'):
            continue
        if len(raw) > MAX_LINE:
            raise ValueError(f'build-script line {line} exceeds byte budget')
        try:
            command = raw[7 if modern else 6:].decode('utf-8')
            if '\0' in command:
                raise ValueError('build-script directive contains NUL')
            key, value = pair(command, 'build-script instruction')
            if not value and key not in ('warning', 'error', 'rustc-flags'):
                raise ValueError(f'empty build-script directive: {key}')
            parsed = instruction(key, value, line, modern, MAX_DIRECTIVES - len(rows))
        except (UnicodeError, ValueError) as error:
            raise ValueError(f'build-script line {line}: {error}') from error
        if len(parsed) > MAX_DIRECTIVES - len(rows):
            raise ValueError('build-script directives exceed count budget')
        rows.extend(parsed)
    return Output(tuple(rows))
