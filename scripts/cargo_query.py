#!/usr/bin/python3
"""Allow only the pinned Cargo planner's observed empty-input rustc queries."""
import json
import os
from pathlib import Path
import subprocess
import sys

from host_build_script import digest

QUERY_ENVIRONMENT = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}


def validate(arguments, target):
    if arguments in (['-vV'], ['--version']):
        return False
    prefix = ['-', '--crate-name', '___', '--print=file-names']
    types = []
    for kind in ('bin', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'):
        types += ['--crate-type', kind]
    suffix = ['--print=sysroot', '--print=split-debuginfo', '--print=crate-name',
              '--print=cfg', '-Wwarnings']
    if arguments not in (prefix + types + suffix, prefix + ['--target', target] + types + suffix):
        raise ValueError('Cargo planning attempted an unsupported query or compilation')
    return True


def main():
    arguments = sys.argv[1:]
    compiler = Path(os.environ['RUSTGM_PLAN_QUERY_COMPILER'])
    expected = os.environ['RUSTGM_PLAN_QUERY_SHA256']
    row = {'arguments': arguments, 'state': 'rejected'}
    log = Path(os.environ['RUSTGM_PLAN_QUERY_LOG'])
    try:
        empty_input = validate(arguments, os.environ['RUSTGM_PLAN_TARGET'])
        if empty_input and sys.stdin.buffer.read(1):
            raise ValueError('Cargo planning supplied source input to rustc')
        if not compiler.is_absolute() or digest(compiler) != expected:
            raise ValueError('planning query compiler differs from its pin')
        row.update(state='query', empty_input=empty_input, compiler_sha256=expected,
                   environment=QUERY_ENVIRONMENT)
        # Cargo's unstable introspection switch is not a target capability.
        # In particular, RUSTC_BOOTSTRAP changes --print=cfg even with no source.
        result = subprocess.run([str(compiler), *arguments], input=b'', capture_output=True,
                                timeout=30, env=QUERY_ENVIRONMENT)
        row['status'] = result.returncode
        for name, data in (('stdout', result.stdout), ('stderr', result.stderr)):
            path = log.with_name(f'query-{os.getpid()}.{name}')
            path.write_bytes(data)
            row[name] = {'path': path.name, 'sha256': digest(path)}
        sys.stdout.buffer.write(result.stdout)
        sys.stderr.buffer.write(result.stderr)
        return result.returncode
    finally:
        with log.open('a') as output:
            output.write(json.dumps(row) + '\n')


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, KeyError, ValueError, subprocess.TimeoutExpired) as error:
        raise SystemExit(str(error)) from error
