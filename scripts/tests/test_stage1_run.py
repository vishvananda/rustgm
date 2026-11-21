"""The stage-1 driver links through the object bridge and records bounded runs."""
import argparse
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import stage1_run  # noqa: E402


def options(lowir, out, **changes):
    values = {"lowir": str(lowir), "native_links": None, "out": str(out), "runs": 1, "arg": [],
              "stdin": None, "env": [], "time_limit": 30, "memory_limit": 1024,
              "expect_status": None, "expect_stdout": None, "backtrace": False}
    values.update(changes)
    return argparse.Namespace(**values)


class Measure(unittest.TestCase):
    def test_a_run_records_status_output_input_and_memory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "in").write_bytes(b"given input\n")
            row = stage1_run.measure(["/bin/sh", "-c", "echo out; cat; echo err >&2; exit 3"],
                                     cwd=root, stdout=root / "o", stderr=root / "e", limit=30,
                                     memory=512 * 1024 * 1024, stdin=root / "in", environment={})
            self.assertEqual((row["status"], row["signal"], row["timed_out"]), (3, None, False))
            self.assertEqual((root / "o").read_bytes(), b"out\ngiven input\n")
            self.assertEqual((root / "e").read_bytes(), b"err\n")
            self.assertGreater(row["peak_mib"], 0)

    def test_a_peak_above_the_drivers_own_size_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            grow = "x = bytearray(256 * 1024 * 1024); x[::4096] = b'1' * len(x[::4096])"
            row = stage1_run.measure([sys.executable, "-c", grow], cwd=root, stdout=root / "o",
                                     stderr=root / "e", limit=60)
            self.assertEqual(row["status"], 0)
            self.assertTrue(row["peak_exact"])
            self.assertGreater(row["peak_mib"], 256)

    def test_a_run_past_its_limit_is_killed_and_marked(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            row = stage1_run.measure(["/bin/sh", "-c", "sleep 30"], cwd=root, stdout=root / "o",
                                     stderr=root / "e", limit=1)
            self.assertTrue(row["timed_out"])
            self.assertEqual(row["signal"], "SIGKILL")
            self.assertLess(row["seconds"], 10)

    def test_a_reached_stub_is_named_from_standard_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "e"
            path.write_bytes(b"noise\n" + stage1_run.STUB_MARKER + b"std:io/mod.rs 1234 read\n")
            self.assertEqual(stage1_run.stub(path), "std:io/mod.rs 1234 read")
            path.write_bytes(b"thread 'main' panicked\n")
            self.assertIsNone(stage1_run.stub(path))


    def test_backtrace_frames_are_read_innermost_first(self):
        text = ("Program received signal SIGABRT, Aborted.\n"
                "#0  __pthread_kill_implementation (threadid=<optimized out>) at x.c:44\n"
                "#4  0x00007ffff7c288ec in __GI_abort () at ./stdlib/abort.c:77\n"
                "#5  0x000055555559cf96 in rgc$4$internal$rg.trap.floorf64.t18446 ()\n")
        self.assertEqual(stage1_run.frames(text), [
            "__pthread_kill_implementation", "__GI_abort", "rgc$4$internal$rg.trap.floorf64.t18446"])


class Drive(unittest.TestCase):
    def test_the_object_bridge_builds_then_each_run_starts_fresh(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lowir = root / "graph.lowir"
            lowir.write_text("serialized unit")
            links = root / "graph.native-links.json"
            links.write_text(json.dumps({"schema": 1, "target": "x86_64-linux", "libraries": [
                {"kind": "dylib", "name": "c"}, {"kind": "dylib", "name": "pthread"}]}))
            cppgm = root / "cppgm"
            cppgm.write_text("pinned")
            out = root / "out"
            commands = []

            def build(sources, output, *, libraries, run):
                commands.append((sources, output, libraries))
                output.write_text("#!/bin/sh\necho \"$1\"; pwd >&2\n")
                output.chmod(0o755)

            with mock.patch.object(stage1_run.backend, "load_tools", return_value={"cppgm": str(cppgm)}), \
                    mock.patch.object(stage1_run.backend, "build_objects", side_effect=build):
                (root / "expected").write_bytes(b"first\n")
                record = stage1_run.drive(options(lowir, out, native_links=str(links), runs=2,
                                                  arg=["first"], expect_status=0,
                                                  expect_stdout=str(root / "expected")))
            self.assertEqual(record["state"], "complete", record.get("error"))
            self.assertEqual(commands, [([lowir], out / "program", [
                {"kind": "dylib", "name": "c"}, {"kind": "dylib", "name": "pthread"}])])
            self.assertEqual([row["problems"] for row in record["runs"]], [[], []])
            self.assertEqual([Path(row["cwd"]).name for row in record["runs"]], ["run-0", "run-1"])
            self.assertEqual(json.loads((out / "record.json").read_text())["state"], "complete")

    def test_an_existing_output_or_another_input_kind_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "graph.rs"
            source.write_text("fn main() {}")
            with self.assertRaisesRegex(ValueError, "serialized .lowir"):
                stage1_run.drive(options(source, root / "out"))
            lowir = root / "graph.lowir"
            lowir.write_text("serialized unit")
            (root / "out").mkdir()
            with self.assertRaisesRegex(ValueError, "already exists"):
                stage1_run.drive(options(lowir, root / "out"))


if __name__ == "__main__":
    unittest.main()
