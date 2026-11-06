"""Independent module checks consume frozen providers rather than bare modules."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_rustc import program_case


class RustcChecks(unittest.TestCase):
    def test_provider_binding_preserves_outcome_and_rejects_changed_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = root / "original.rs"
            original.write_text("pub fn f(){}\n")
            path = root / "provider.rs"
            path.write_text("#![no_std]\npub fn f(){}\n")
            item = {"oracle": "source-fragment", "destination": original.name,
                    "oracle_companion": {
                "path": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "expected_status": 0}}
            self.assertEqual(program_case(root, item), (path, 0, ["--crate-type=lib"]))
            item["oracle_companion"]["expected_status"] = 1
            self.assertEqual(program_case(root, item), (path, 1, ["--crate-type=lib"]))
            path.write_text("pub fn changed(){}\n")
            with self.assertRaisesRegex(ValueError, "companion bytes changed"):
                program_case(root, item)
            item["oracle_companion"]["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
            with self.assertRaisesRegex(ValueError, "does not embed its original source"):
                program_case(root, item)

    def test_unknown_outcomes_do_not_become_expected_rejections(self):
        with self.assertRaisesRegex(ValueError, "unknown upstream oracle"):
            program_case(Path("."), {"oracle": "required but unimplemented"})
        self.assertIsNone(program_case(Path("."), {"oracle": "source-fragment"}))
        with self.assertRaisesRegex(ValueError, "repository fixture"):
            program_case(Path("."), {"oracle": "source-fragment", "oracle_companion": {
                "path": "../outside.rs"}})
