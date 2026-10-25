"""Retain bounded multi-crate command observations for independent qualification."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Check:
    def __init__(self, app, folder):
        self.app = app.resolve()
        self.folder = folder.resolve()
        self.folder.mkdir(parents=True, exist_ok=False)
        self.rows = []

    def run(self, name, command, expected=0, codes=(), *, timeout=180, environment=None):
        started = time.monotonic()
        command = [str(arg) for arg in command]
        environment = {} if environment is None else environment
        options = {'capture_output': True, 'timeout': timeout}
        if environment:
            options['env'] = {**os.environ, **environment}
        result = subprocess.run(command, **options)
        stdout = self.folder / (name + '.stdout')
        stderr = self.folder / (name + '.stderr')
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        self.rows.append({'name': name, 'command': command, 'status': result.returncode,
                          'expected': expected, 'seconds': time.monotonic() - started,
                          'timeout': timeout, 'environment': environment,
                          'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr)})
        (self.folder / 'runs.json').write_text(json.dumps(self.rows, indent=2) + '\n')
        if result.returncode != expected or any(code.encode() not in result.stderr for code in codes):
            raise ValueError(f'{name}: expected {expected}, got {result.returncode}\n'
                             + result.stderr.decode(errors='replace'))
        return result

