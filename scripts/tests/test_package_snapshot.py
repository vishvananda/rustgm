"""Frozen package bytes, lockfile closure and unit ownership must agree."""
from dataclasses import replace
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import package_snapshot
from crate_vendor import Vendor
from host_build_script import digest
from package_snapshot import CHECKSUM, REGISTRY, extract
import test_build_units as unit_tests
from test_build_units import decode, row
from cargo_plan import execute


def archive(root, extra=(), *, manifest=None):
    path = root / 'sample.crate'
    with tarfile.open(path, 'w:gz') as output:
        for name, data in [('sample-1.0.0/Cargo.toml', manifest or
                           b'[package]\nname="sample"\nversion="1.0.0"\nedition="2021"\n'),
                           ('sample-1.0.0/src/lib.rs', b'// original\r\npub fn answer() {}\r\n'),
                           ('sample-1.0.0/Cargo.lock', b'original packaged lockfile\n'), *extra]:
            if isinstance(data, tarfile.TarInfo):
                output.addfile(data)
            else:
                entry = tarfile.TarInfo(name)
                entry.size = len(data)
                entry.mode = 0o644
                output.addfile(entry, io.BytesIO(data))
    return path


def snapshot(root, extra=()):
    source = archive(root, extra)
    vendor = root / 'vendor'
    vendor.mkdir()
    return extract(source, digest(source), 'sample', '1.0.0', vendor)


def lockfile(path, package):
    path.write_text(f'version = 4\n[[package]]\nname = "sample"\nversion = "1.0.0"\n'
                    f'source = "{REGISTRY}"\nchecksum = "{package.archive_sha256}"\n')


class Snapshots(unittest.TestCase):
    def test_original_bytes_and_packaged_lock_are_unchanged(self):
        with tempfile.TemporaryDirectory() as temporary:
            package = snapshot(Path(temporary))
            self.assertEqual((package.root / 'src/lib.rs').read_bytes(),
                             b'// original\r\npub fn answer() {}\r\n')
            self.assertEqual((package.root / 'Cargo.lock').read_bytes(), b'original packaged lockfile\n')
            checksum = json.loads((package.root / CHECKSUM).read_text())
            self.assertEqual(checksum['package'], package.archive_sha256)
            self.assertNotIn(CHECKSUM, checksum['files'])
            self.assertEqual(checksum['files']['src/lib.rs'], digest(package.root / 'src/lib.rs'))
            package.verify()

    def test_identity_is_portable_but_snapshot_paths_are_explicit(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = snapshot(root)
            other = root / 'other'
            other.mkdir()
            second = extract(first.archive, first.archive_sha256, 'sample', '1.0.0', other)
            self.assertEqual(first.identity, second.identity)
            self.assertNotEqual(first.root, second.root)
            self.assertEqual(first.files, second.files)

    def test_missing_added_changed_symlink_and_mode_inputs_fail(self):
        def mode(path):
            path.chmod(0o755)

        changes = [lambda path: path.unlink(), lambda path: path.write_bytes(b'changed'), mode,
                   lambda path: path.with_name('extra.rs').write_bytes(b'extra'),
                   lambda path: path.with_name('empty').mkdir(),
                   lambda path: path.with_name('link').symlink_to(path)]
        for change in changes:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                package = snapshot(Path(temporary))
                change(package.root / 'src/lib.rs')
                with self.assertRaises(ValueError):
                    package.verify()

    def test_archive_pin_and_manifest_identity_are_required(self):
        for manifest in (None, b'[package]\nname="wrong"\nversion="1.0.0"\n'):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = archive(root, manifest=manifest)
                vendor = root / 'vendor'
                vendor.mkdir()
                expected = '0' * 64 if manifest is None else digest(source)
                with self.assertRaises(ValueError):
                    extract(source, expected, 'sample', '1.0.0', vendor)

    def test_archive_paths_links_duplicates_and_reserved_metadata_fail(self):
        link = tarfile.TarInfo('sample-1.0.0/link')
        link.type = tarfile.SYMTYPE
        link.linkname = '../outside'
        hardlink = tarfile.TarInfo('sample-1.0.0/hardlink')
        hardlink.type = tarfile.LNKTYPE
        hardlink.linkname = 'sample-1.0.0/src/lib.rs'
        extras = [('sample-1.0.0/../escape', b'bad'), ('/sample-1.0.0/absolute', b'bad'),
                  ('sample-1.0.0//alias', b'bad'), ('sample-1.0.0/src/lib.rs', b'duplicate'),
                  ('sample-1.0.0/' + CHECKSUM, b'forged'), ('link', link), ('hardlink', hardlink)]
        for extra in extras:
            with self.subTest(extra=extra[0]), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaises(ValueError):
                    snapshot(Path(temporary), [extra])

    def test_extraction_and_inventory_budgets_are_enforced(self):
        for field, value in (('MAX_FILES', 1), ('MAX_BYTES', 1)):
            with tempfile.TemporaryDirectory() as temporary:
                with patch.object(package_snapshot, field, value), self.assertRaises(ValueError):
                    snapshot(Path(temporary))
            with tempfile.TemporaryDirectory() as temporary:
                package = snapshot(Path(temporary))
                with patch.object(package_snapshot, field, value), self.assertRaises(ValueError):
                    package.verify()


class Vendors(unittest.TestCase):
    def test_lockfile_requires_complete_registry_set_even_if_unit_is_inactive(self):
        for mutation in ('none', 'missing', 'checksum', 'extra', 'source'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                package = snapshot(root)
                lock = root / 'Cargo.lock'
                lockfile(lock, package)
                text = lock.read_text()
                if mutation == 'missing':
                    text = 'version=4\npackage=[]\n'
                elif mutation == 'checksum':
                    text = text.replace(package.archive_sha256, '0' * 64)
                elif mutation == 'extra':
                    text += '\n[[package]]\nname="missing"\nversion="2.0.0"\n' + f'source="{REGISTRY}"\n'
                elif mutation == 'source':
                    text = text.replace(REGISTRY, 'git+https://example.invalid/repository')
                lock.write_text(text)
                vendor = Vendor(package.root.parent, (package,))
                if mutation == 'none':
                    vendor.verify(lock)
                else:
                    with self.assertRaisesRegex(ValueError, 'closure'):
                        vendor.verify(lock)

    def test_unpinned_vendor_entries_and_duplicate_packages_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = snapshot(root)
            lock = root / 'Cargo.lock'
            lockfile(lock, package)
            with self.assertRaisesRegex(ValueError, 'inventory'):
                Vendor(package.root.parent, (package, package)).verify(lock)
            (package.root.parent / 'unrecorded').mkdir()
            with self.assertRaisesRegex(ValueError, 'inventory'):
                Vendor(package.root.parent, (package,)).verify(lock)

    def test_unit_binding_checks_both_source_ownership_and_registry_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            package = snapshot(Path(temporary))
            vendor = Vendor(package.root.parent, (package,))
            unit = row(package.cargo_id)
            unit['target']['src_path'] = str(package.root / 'src/lib.rs')
            data = dict(version=1, roots=[0], units=[unit])
            self.assertEqual(vendor.bind(decode(data), {}), [package.identity])
            unit['pkg_id'] = REGISTRY + '#other@1.0.0'
            with self.assertRaisesRegex(ValueError, 'different registry package'):
                vendor.bind(decode(data), {})
            unit['target']['src_path'] = '/undeclared/lib.rs'
            with self.assertRaisesRegex(ValueError, 'no declared package source'):
                vendor.bind(decode(data), {})

    def test_planner_authenticates_all_packages_before_and_after_execution(self):
        for mutate in (False, True):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                package = snapshot(root)
                job, source = unit_tests.Planning().job(root)
                lockfile(job.lockfile, package)
                job = replace(job, inputs=tuple((path, digest(path)) for path, _ in job.inputs),
                              vendor=Vendor(package.root.parent, (package,)))
                source = package.root / 'src/lib.rs'
                process = unit_tests.Planning.process(source)

                def run(*args, **kwargs):
                    result = process(*args, **kwargs)
                    stream = kwargs['stdout']
                    stream.seek(0)
                    data = dict(version=1, roots=[0], units=[row(package.cargo_id)])
                    data['units'][0]['target']['src_path'] = str(source)
                    stream.write(json.dumps(data).encode())
                    stream.truncate()
                    if mutate:
                        (package.root / 'Cargo.lock').write_text('changed inactive input')
                    return result

                with patch('cargo_plan.subprocess.run', side_effect=run):
                    if mutate:
                        with self.assertRaisesRegex(ValueError, 'inventory changed'):
                            execute(job, root / 'output')
                    else:
                        execute(job, root / 'output')
                receipt = json.loads((root / 'output/receipt.json').read_text())
                self.assertEqual(receipt['state'], 'failed' if mutate else 'planned')
                if not mutate:
                    self.assertEqual(receipt['packages'], [package.identity])
                    self.assertIn('source.crates-io.replace-with="rustgm-frozen"', receipt['command'])


if __name__ == '__main__':
    unittest.main()
