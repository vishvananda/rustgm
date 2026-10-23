"""Complete local trees and portable origins remain separate from Cargo path IDs."""
from collections import Counter
from copy import deepcopy
from dataclasses import replace
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cargo_plan import Job, execute
from host_process import digest
from local_package import LocalPackage, bind, capture, validate
from test_build_units import TRIPLE, decode, edge, row


def package(root, folder='sample', origin='workspace/sample'):
    path = root / folder
    (path / 'src').mkdir(parents=True)
    (path / 'Cargo.toml').write_text('[package]\nname="sample"\nversion="1.0.0"\n')
    (path / 'src/lib.rs').write_bytes(b'mod other;\r\npub fn value()->u8{other::VALUE}\r\n')
    (path / 'src/other.rs').write_text('pub const VALUE:u8=7;\n')
    (path / 'data').write_bytes(b'unchanged non-Rust input\x00')
    return capture(path, 'sample', '1.0.0', origin)


def setup(root):
    value = package(root)
    manifest, lock = root / 'Cargo.toml', root / 'Cargo.lock'
    manifest.write_text('[workspace]\nmembers=["sample"]\nresolver="2"\n')
    lock.write_text('version=4\n[[package]]\nname="sample"\nversion="1.0.0"\n')
    cargo, rustc = root / 'cargo', root / 'rustc'
    cargo.write_text('pinned cargo')
    rustc.write_text('pinned query compiler')
    job = Job(cargo, digest(cargo), rustc, digest(rustc), manifest, lock,
              tuple((p, digest(p)) for p in (manifest, lock)), 'sample', TRIPLE,
              local_packages=(value,))
    unit = row(value.cargo_id)
    unit['target'].update(name='sample', src_path=str(value.root / 'src/lib.rs'))
    return job, value, dict(version=1, roots=[0], units=[unit])


def process(data):
    def run(command, *, cwd, env, stdout, stderr, timeout):
        stdout.write(json.dumps(data).encode())
        return subprocess.CompletedProcess(command, 0)
    return run


class LocalInputs(unittest.TestCase):
    def test_relocation_preserves_content_identity_and_original_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = package(root)
            moved = root / 'other location'
            shutil.copytree(first.root, moved)
            second = capture(moved, first.name, first.version, first.origin)
            self.assertEqual(first.identity, second.identity)
            self.assertNotEqual(first.cargo_id, second.cargo_id)
            self.assertIn('%20', second.cargo_id)
            self.assertEqual((moved / 'src/lib.rs').read_bytes(),
                             b'mod other;\r\npub fn value()->u8{other::VALUE}\r\n')
            self.assertNotIn(str(first.root), json.dumps(first.identity))

    def test_complete_inventory_detects_changes_and_rejects_unresolved_identity(self):
        changes = [lambda p: p.unlink(), lambda p: p.write_text('changed'),
                   lambda p: p.chmod(0o755), lambda p: p.with_name('added').write_text('new'),
                   lambda p: p.with_name('empty').mkdir(),
                   lambda p: p.with_name('link').symlink_to(p)]
        for change in changes:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                value = package(Path(temporary))
                change(value.root / 'src/other.rs')
                with self.assertRaises(ValueError):
                    value.verify()
        with tempfile.TemporaryDirectory() as temporary:
            value = package(Path(temporary))
            for origin in ('', '/absolute', '../escape', 'a//b'):
                with self.assertRaises(ValueError):
                    capture(value.root, value.name, value.version, origin)
            (value.root / 'Cargo.toml').write_text('[package]\nname="sample"\nversion.workspace=true\n')
            with self.assertRaisesRegex(ValueError, 'identity'):
                capture(value.root, value.name, value.version, value.origin)

    def test_origins_roots_and_source_owners_must_be_unambiguous(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job, value, data = setup(root)
            other = package(root, 'different', 'workspace/different')
            for packages in ((value, value), (value, replace(other, origin=value.origin)),
                             (value, replace(other, root=value.root / 'nested'))):
                with self.assertRaises(ValueError):
                    validate(packages)
            for identity in (other.cargo_id, 'registry+test#sample@1.0.0'):
                wrong = deepcopy(data)
                wrong['units'][0]['pkg_id'] = identity
                with self.assertRaises(ValueError):
                    bind(decode(wrong), (value, other))
            with patch('cargo_plan.subprocess.run') as run, self.assertRaises(ValueError):
                execute(job, value.root / 'generated')
            run.assert_not_called()

    def test_planner_binds_full_local_package_and_revalidates_after_execution(self):
        for changed in (False, True):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                job, value, data = setup(root)
                observed = process(data)

                def run(*args, **kwargs):
                    result = observed(*args, **kwargs)
                    if changed:
                        (value.root / 'src/added.rs').write_text('new module')
                    return result

                with patch('cargo_plan.subprocess.run', side_effect=run):
                    if changed:
                        with self.assertRaisesRegex(ValueError, 'inventory changed'):
                            execute(job, root / 'output')
                    else:
                        execute(job, root / 'output')
                receipt = json.loads((root / 'output/receipt.json').read_text())
                self.assertEqual(receipt['state'], 'failed' if changed else 'planned')
                if not changed:
                    self.assertEqual(receipt['packages'], [value.identity])
                    self.assertEqual(len(receipt['local_packages'][0]['files']), 4)

    def test_shared_units_verify_each_package_twice_not_per_consumer(self):
        for width in (8, 32, 128):
            with self.subTest(width=width), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                job, value, data = setup(root)
                for index in range(1, width):
                    other = deepcopy(data['units'][0])
                    other['target']['name'] = f'consumer_{index}'
                    other['dependencies'] = []
                    data['units'].append(other)
                    data['units'][0]['dependencies'].append(edge(index, f'consumer_{index}'))
                visits, verify = Counter(), LocalPackage.verify

                def counted(package):
                    visits[package.cargo_id] += 1
                    return verify(package)

                with patch.object(LocalPackage, 'verify', counted), \
                        patch('cargo_plan.subprocess.run', side_effect=process(data)):
                    execute(job, root / 'output')
                self.assertEqual(visits, {value.cargo_id: 2})


if __name__ == '__main__':
    unittest.main()
