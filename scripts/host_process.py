"""Recorded host processes share explicit environments and bounded execution."""
from dataclasses import dataclass
import hashlib
from pathlib import Path
import subprocess
import time
from typing import Callable

# Shared default; a longer limit is an explicit per-call argument, recorded per row.
DEFAULT_TIMEOUT = 120


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()


@dataclass
class CommandLog:
    folder: Path
    cwd: Path
    commands: list
    save: Callable[[], None]

    def run(self, name, command, environment, timeout=DEFAULT_TIMEOUT):
        if type(timeout) is not int or timeout <= 0:
            raise ValueError('process timeout must be a positive whole number of seconds')
        command = [str(part) for part in command]
        started = time.monotonic()
        row = {'name': name, 'command': command, 'cwd': str(self.cwd),
               'environment': environment, 'timeout_seconds': timeout, 'status': None}
        self.commands.append(row)
        self.save()
        with (self.folder / f'{name}.stdout').open('xb') as stdout, \
                (self.folder / f'{name}.stderr').open('xb') as stderr:
            try:
                result = subprocess.run(command, cwd=self.cwd, env=environment,
                                        stdout=stdout, stderr=stderr, timeout=timeout)
            except subprocess.TimeoutExpired:
                row['error'] = 'timeout'
                raise
            finally:
                row['seconds'] = time.monotonic() - started
                self.save()
        row['status'] = result.returncode
        row['stdout_sha256'] = digest(self.folder / f'{name}.stdout')
        row['stderr_sha256'] = digest(self.folder / f'{name}.stderr')
        self.save()
        return result.returncode
