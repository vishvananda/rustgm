"""Verify rust-src discovery, hash checks, and locked crate extraction."""
import hashlib
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import setup_rust_src


def write_crate_archive(path, folder, files):
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as archive:
        for rel, content in files.items():
            data = content.encode()
            info = tarfile.TarInfo(f"{folder}/{rel}")
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
    raw = buf.getvalue()
    path.write_bytes(raw)
    return hashlib.sha256(raw).hexdigest()


class SetupRustSrc(unittest.TestCase):
    def test_normalize_library_root_accepts_nested_layouts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lib = root / "rust-src/lib/rustlib/src/rust/library"
            (lib / "core/src").mkdir(parents=True)
            (lib / "core/src/lib.rs").write_text("// core\n")
            self.assertEqual(setup_rust_src.normalize_library_root(lib), lib.resolve())
            self.assertEqual(setup_rust_src.normalize_library_root(lib.parent), lib.resolve())
            self.assertEqual(setup_rust_src.normalize_library_root(root), lib.resolve())
            self.assertIsNone(setup_rust_src.normalize_library_root(lib / "core"))

    def test_verify_library_rejects_missing_required_roots(self):
        with tempfile.TemporaryDirectory() as directory:
            lib = Path(directory) / "library"
            (lib / "core/src").mkdir(parents=True)
            (lib / "core/src/lib.rs").write_text("// core\n")
            with self.assertRaisesRegex(ValueError, "missing alloc/src/lib.rs"):
                setup_rust_src.verify_library(lib, full_core_check=False)
            for rel in setup_rust_src.REQUIRED_LIBRARY_ROOTS:
                target = lib / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("// ok\n")
            self.assertEqual(setup_rust_src.verify_library(lib, full_core_check=False), lib.resolve())

    def test_ensure_vendor_crates_extracts_verified_archive_and_rejects_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / "cache"
            vendor = root / "vendor"
            cache.mkdir()
            archive = cache / "demo-0.1.0.crate"
            digest = write_crate_archive(archive, "demo-0.1.0", {"src/lib.rs": "pub fn f() -> u32 { 7 }\n"})
            found = setup_rust_src.find_cached_crate("demo", "0.1.0", digest, [cache])
            self.assertEqual(found, archive)
            self.assertIsNone(setup_rust_src.find_cached_crate("demo", "0.1.0", "0" * 64, [cache]))
            out = setup_rust_src.ensure_vendor_crates(
                root, crates=(("demo", "0.1.0", digest),), vendor_dir=vendor, cache_dir=cache
            )
            self.assertEqual(out, vendor.resolve())
            self.assertTrue((vendor / "demo-0.1.0/src/lib.rs").is_file())
            self.assertEqual((vendor / "demo-0.1.0/.rustgm-verified-sha256").read_text().strip(), digest)


if __name__ == "__main__":
    unittest.main()
