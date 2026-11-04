"""External dependency manifests reject malformed inputs before invoking a backend."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import native_links


class NativeLinks(unittest.TestCase):
    def test_roundtrip_unicode_escaped_names_and_order(self):
        libraries = [{"name": 'unicode-é-"-\\-\n', "kind": "dylib"},
                     {"name": "ordinary", "kind": "static"}]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "links.json"
            path.write_text(json.dumps({"schema": 1, "target": "x86_64-linux",
                                        "libraries": libraries}))
            self.assertEqual(native_links.load(path), libraries)

    def test_manifest_rejects_unknown_shapes_and_combined_budget_exhaustion(self):
        valid = {"schema": 1, "target": "x86_64-linux", "libraries": []}
        invalid = [[], {**valid, "schema": True}, {**valid, "schema": 2},
                   {**valid, "target": "other"}, {**valid, "extra": 1},
                   {**valid, "libraries": {}},
                   {**valid, "libraries": [{"name": "a", "kind": "static"}] * 1025},
                   {**valid, "libraries": [{"name": "x" * 4096, "kind": "dylib"}] * 17}]
        for name, kind in [("", "dylib"), ("a\0b", "static"), ("a", "framework"),
                           ("x" * 4097, "dylib"), ([], "static")]:
            invalid.append({**valid, "libraries": [{"name": name, "kind": kind}]})
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "links.json"
            for plan in invalid:
                path.write_text(json.dumps(plan))
                with self.subTest(plan=str(plan)[:100]), self.assertRaises(ValueError):
                    native_links.load(path)
            path.write_text('{"schema":1,"schema":1,"target":"x86_64-linux","libraries":[]}')
            with self.assertRaises(ValueError):
                native_links.load(path)
            path.write_bytes(b" " * 1_000_001)
            with self.assertRaises(ValueError):
                native_links.load(path)

    def test_library_names_never_become_driver_switches(self):
        libraries = [{"name": "-Wl,arbitrary", "kind": "dylib"},
                     {"name": "$(touch unwanted)", "kind": "static"}]
        arguments = native_links.arguments(libraries, [])
        self.assertIn("-l-Wl,arbitrary", arguments)
        self.assertIn("-l:lib$(touch unwanted).a", arguments)
        self.assertNotIn("-Wl,arbitrary", arguments)
