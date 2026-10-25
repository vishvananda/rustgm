"""Cached syntax results must preserve uncached and cross-file audit findings."""
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rust_audit


class AuditCache(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.source = self.root / "dev/src/support/value.rs"
        self.source.parent.mkdir(parents=True)
        self.source.write_text("fn value() {}\n")
        self.cache = self.root / "cache.json"

    def audit(self):
        output = io.StringIO()
        with redirect_stdout(output):
            result = rust_audit.audit(self.root, self.cache)
        return result, output.getvalue()

    def test_reuses_unchanged_files_but_rechecks_changed_content_and_roles(self):
        self.assertTrue(self.audit()[0])
        with patch.object(rust_audit, "inspect_file", wraps=rust_audit.inspect_file) as inspect:
            self.assertTrue(self.audit()[0])
            inspect.assert_not_called()
            self.source.write_text("unsafe fn value() {}\n")
            self.assertFalse(self.audit()[0])
            self.assertEqual(inspect.call_count, 1)
            roles = self.root / "scripts/rust-module-roles.json"
            roles.parent.mkdir()
            roles.write_text(json.dumps({"dev/src/support/value.rs": "unknown"}))
            self.assertIn("unrecognized module role", self.audit()[1])
            self.assertEqual(inspect.call_count, 2)

    def test_global_duplicates_and_removed_files_are_not_hidden(self):
        block = "\n".join(f"fn value_{number}() {{}}" for number in range(32))
        self.source.write_text(block)
        self.assertTrue(self.audit()[0])
        other = self.source.with_name("other.rs")
        other.write_text(block)
        self.assertIn("duplicate 28-line code block", self.audit()[1])
        self.assertIn("duplicate 28-line code block", self.audit()[1])
        other.unlink()
        self.assertTrue(self.audit()[0])
        self.assertNotIn("dev/src/support/other.rs", json.loads(self.cache.read_text())["files"])

    def test_global_stems_and_malformed_cache_still_checked(self):
        for number in range(7):
            self.source.with_name(f"repeated_{number}.rs").write_text(f"fn name_{number}() {{}}")
        self.assertIn("repeated filename stem", self.audit()[1])
        self.assertIn("repeated filename stem", self.audit()[1])
        self.cache.write_text('{"signature": null')
        self.assertIn("repeated filename stem", self.audit()[1])
