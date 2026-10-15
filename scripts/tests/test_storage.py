"""Artifact pruning must preserve evidence, pins, caches and paths outside obj."""
from contextlib import redirect_stdout
import io
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from storage import check_space, exclusive, main, minimum_reserve, prune


class StorageTests(unittest.TestCase):
    def test_reserve_defaults_and_explicit_override(self):
        for environment, expected in (({}, 40), ({"GITHUB_ACTIONS": "true"}, 8),
                                      ({"GITHUB_ACTIONS": "false"}, 40),
                                      ({"GITHUB_ACTIONS": "true",
                                        "RUSTGM_MIN_FREE_GIB": "12"}, 12)):
            with patch.dict(os.environ, environment, clear=True):
                self.assertEqual(minimum_reserve(), expected)

    @staticmethod
    def binary(path):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"\x7fELF" + b"x" * 4096)
        path.chmod(0o755)

    def test_apply_only_removes_generated_files_and_preserves_pins_and_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "obj/run/program"
            self.binary(output)
            metadata = output.with_suffix(".rmeta")
            metadata.write_bytes(b"metadata")
            evidence = output.with_suffix(".json")
            evidence.write_text('{"passed":true}')
            source = output.with_suffix(".rs")
            source.write_text("fn main() {}")
            pinned = root / "obj/baseline/candidate"
            self.binary(pinned)
            (pinned.parent / ".keep-artifacts").touch()
            cache = root / "obj/rust/release/rustgm"
            self.binary(cache)
            external = root / "external/program"
            self.binary(external)
            (root / "obj/escape").symlink_to(external.parent, target_is_directory=True)
            (root / "obj/run/link.o").symlink_to(external)
            with redirect_stdout(io.StringIO()):
                self.assertEqual(prune(root), 2)
                self.assertTrue(output.exists())
                self.assertEqual(prune(root, apply=True), 2)
            for path in (evidence, source, pinned, cache, external):
                self.assertTrue(path.exists(), path)
            self.assertFalse(output.exists())
            self.assertFalse(metadata.exists())

    def test_age_filter_and_explicit_build_cache(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old = root / "obj/run/old"
            recent = root / "obj/run/recent"
            cache = root / "obj/rust/release/rustgm"
            for path in (old, recent, cache):
                self.binary(path)
            os.utime(old, (time.time() - 7200,) * 2)
            with redirect_stdout(io.StringIO()):
                self.assertEqual(prune(root, apply=True, hours=1, build_cache=True), 1)
            self.assertTrue(recent.exists())
            self.assertFalse(cache.exists())

    def test_active_runs_and_symlinked_obj_are_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch("storage.active_artifacts", return_value=[123]):
                with self.assertRaisesRegex(ValueError, "still running"):
                    prune(root, apply=True)
            external = root / "outside"
            external.mkdir()
            (root / "obj").symlink_to(external, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "symlinked obj"):
                prune(root, apply=True)

    def test_low_space_and_overlapping_commands_fail_before_work(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch("storage.shutil.disk_usage") as usage:
                usage.return_value.free = 1
                with self.assertRaisesRegex(ValueError, "required"):
                    check_space(root, 40)
            with exclusive(root):
                with self.assertRaisesRegex(ValueError, "running"):
                    with exclusive(root):
                        self.fail("overlapping command acquired lock")

    def test_command_success_prunes_and_failure_preserves_artifacts(self):
        script = ("from pathlib import Path; "
                  "p=Path('obj/run/program'); p.parent.mkdir(parents=True,exist_ok=True); "
                  "p.write_bytes(b'\\x7fELFxxxx'); p.chmod(0o755)")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            previous = root / "obj/failed-earlier/program"
            self.binary(previous)
            os.utime(previous, (time.time() - 3600,) * 2)
            for code, no_prune in ((7, False), (0, True), (0, False)):
                argv = ["storage.py", "run", "--minimum-gib", "0", "--", sys.executable,
                        "-c", script + f"; raise SystemExit({code})"]
                if no_prune:
                    argv.insert(2, "--no-prune")
                with patch("storage.ROOT", root), patch.object(sys, "argv", argv):
                    with redirect_stdout(io.StringIO()):
                        self.assertEqual(main(), code)
                self.assertEqual((root / "obj/run/program").exists(), code != 0 or no_prune)
                self.assertTrue(previous.exists())
