"""The instruction benchmark freezes inputs, refuses changed output and compares receipts."""
import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import instruction_bench as bench

BENCHMARKS = [
    {"name": "small", "argv": ["--emit-lowir", "-o", "out.lowir", "{set}/inputs/small.rs"],
     "outputs": ["out.lowir"]},
    {"name": "guest", "argv": ["--skip", "skip.tsv", "{set}/tree/lib.rs"], "env": {"ARCH": "x86_64"},
     "outputs": ["skip.tsv"], "empty": ["skip.tsv"], "status": 1, "timeout": 5},
]


class Compiler:
    """A measurer standing in for the hardware counters and the compiler."""

    def __init__(self, instructions, *, lowir=b"function @main\n", status=None):
        self.instructions = iter(instructions)
        self.lowir = lowir
        self.status = status
        self.calls = []

    def __call__(self, compiler, argv, env, folder, timeout):
        self.calls.append((argv, env, timeout, sorted(path.name for path in folder.iterdir())))
        (folder / "stdout").write_bytes(b"")
        (folder / "stderr").write_bytes(b"warning: one\n")
        guest = "--skip" in argv
        if not guest:
            content = self.lowir(len(self.calls)) if callable(self.lowir) else self.lowir
            (folder / "out.lowir").write_bytes(content)
        instructions = next(self.instructions)
        return {"status": self.status if self.status is not None else int(guest),
                "instructions": instructions, "cycles": 2 * instructions, "peak_rss_kib": 1000 + instructions,
                "wall_seconds": 0.5, "user_seconds": 0.4, "system_seconds": 0.1}


class InstructionBench(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        sources = self.root / "sources"
        (sources / "tree/nested").mkdir(parents=True)
        (sources / "tree/lib.rs").write_text("pub mod nested;\n")
        (sources / "tree/nested/mod.rs").write_text("pub fn f() {}\n")
        (sources / "small.rs").write_text("fn main() {}\n")
        self.plan = {"trees": {"tree": str(sources / "tree")},
                     "files": {"inputs/small.rs": str(sources / "small.rs")},
                     "pins": {"inputs/small.rs": bench.sha256(sources / "small.rs")},
                     "benchmarks": BENCHMARKS}
        self.compiler = self.root / "rustgm"
        self.compiler.write_bytes(b"compiler")

    def frozen(self):
        bench.freeze(self.plan, self.root / "set")
        return self.root / "set"

    def test_freeze_copies_and_hashes_every_input_and_checks_pins(self):
        files = bench.freeze(self.plan, self.root / "set")
        self.assertEqual(sorted(files), ["inputs/small.rs", "tree/lib.rs", "tree/nested/mod.rs"])
        manifest = json.loads((self.root / "set/set.json").read_text())
        self.assertEqual(manifest["files"], files)
        self.assertEqual(manifest["benchmarks"], BENCHMARKS)
        self.assertEqual((self.root / "set/tree/nested/mod.rs").read_text(), "pub fn f() {}\n")
        self.assertFalse(os.access(self.root / "set/tree/lib.rs", os.W_OK) and os.geteuid() != 0)
        self.plan["pins"]["tree/lib.rs"] = "0" * 64
        with self.assertRaisesRegex(bench.BenchError, "pinned"):
            bench.freeze(self.plan, self.root / "other")

    def test_freeze_refuses_links_escaping_paths_and_unsafe_benchmarks(self):
        (self.root / "sources/tree/link.rs").symlink_to(self.root / "sources/small.rs")
        with self.assertRaisesRegex(bench.BenchError, "regular file"):
            bench.freeze(self.plan, self.root / "linked")
        (self.root / "sources/tree/link.rs").unlink()
        for target in ("../escape.rs", "/absolute.rs", "set.json"):
            plan = dict(self.plan, files={target: self.plan["files"]["inputs/small.rs"]}, pins={})
            with self.assertRaisesRegex(bench.BenchError, "inside the set"):
                bench.freeze(plan, self.root / "escape")
        for row in ({"name": "../up", "argv": ["x"]}, {"name": "a", "argv": []},
                    {"name": "a", "argv": ["x"], "outputs": ["../out"]},
                    {"name": "a", "argv": ["x"], "outputs": ["stderr"]},
                    {"name": "a", "argv": ["x"], "env": {"A": 1}}):
            with self.assertRaises(bench.BenchError):
                bench.checked_benchmarks([row])
        with self.assertRaisesRegex(bench.BenchError, "repeated"):
            bench.checked_benchmarks([BENCHMARKS[0], BENCHMARKS[0]])

    def test_a_changed_or_missing_input_stops_the_run(self):
        directory = self.frozen()
        changed = directory / "tree/lib.rs"
        changed.chmod(0o644)
        changed.write_text("pub mod other;\n")
        with self.assertRaisesRegex(bench.BenchError, "tree/lib.rs is missing or changed"):
            bench.run_set(directory, self.compiler, self.root / "run", measurer=Compiler([1]))
        changed.unlink()
        with self.assertRaisesRegex(bench.BenchError, "tree/lib.rs is missing or changed"):
            bench.load_set(directory)

    def test_a_run_records_counters_spread_and_output_hashes(self):
        directory = self.frozen()
        compiler = Compiler([1000, 1010, 1004, 50, 50, 50])
        with contextlib.redirect_stdout(io.StringIO()) as printed:
            receipt = bench.run_set(directory, self.compiler, self.root / "run", measurer=compiler)
        self.assertEqual(receipt, json.loads((self.root / "run/receipt.json").read_text()))
        self.assertTrue(receipt["complete"])
        self.assertEqual(receipt["compiler"]["sha256"], bench.sha256(self.compiler))
        small = receipt["benchmarks"]["small"]
        self.assertEqual((small["instructions"], small["instructions_min"], small["instructions_max"]),
                         (1004, 1000, 1010))
        self.assertAlmostEqual(small["instructions_spread"], 0.01)
        self.assertEqual((small["cycles"], small["peak_rss_kib"]), (2008, 2010))
        self.assertEqual(len(small["samples"]), 3)
        self.assertNotIn("status", small["samples"][0])
        self.assertEqual(small["argv"][-1], f"{directory.resolve()}/inputs/small.rs")
        self.assertEqual(small["produced"]["status"], 0)
        self.assertEqual(small["produced"]["outputs"]["out.lowir"]["bytes"], len(b"function @main\n"))
        self.assertEqual(set(small["produced"]["outputs"]), {"out.lowir", "stdout", "stderr"})
        guest = receipt["benchmarks"]["guest"]
        self.assertEqual((guest["produced"]["status"], guest["instructions_spread"]), (1, 0.0))
        self.assertEqual(guest["produced"]["outputs"]["skip.tsv"]["bytes"], 0)
        # Each compilation starts in a directory holding only its declared empty files.
        self.assertEqual([call[3] for call in compiler.calls], [[], [], [], ["skip.tsv"]] + [["skip.tsv"]] * 2)
        self.assertEqual(compiler.calls[3][1:3], ({"ARCH": "x86_64"}, 5))
        # The first repetition keeps its outputs; equal later ones are removed.
        self.assertTrue((self.root / "run/small/0/out.lowir").is_file())
        self.assertFalse((self.root / "run/small/2/out.lowir").exists())
        self.assertIn("spread 1.0000%", printed.getvalue())

    def test_a_run_selects_named_benchmarks_and_refuses_unknown_ones(self):
        directory = self.frozen()
        with contextlib.redirect_stdout(io.StringIO()):
            receipt = bench.run_set(directory, self.compiler, self.root / "one", repeat=1, only=["guest"],
                                    measurer=Compiler([7]))
        self.assertEqual(list(receipt["benchmarks"]), ["guest"])
        with self.assertRaisesRegex(bench.BenchError, "unknown benchmarks"):
            bench.run_set(directory, self.compiler, self.root / "none", only=["absent"], measurer=Compiler([]))

    def test_output_that_differs_between_repetitions_or_a_wrong_status_fails(self):
        directory = self.frozen()
        unstable = Compiler([1, 1, 1], lowir=lambda call: b"run %d\n" % call)
        with self.assertRaisesRegex(bench.BenchError, "small: repetition 1 produced different output"):
            bench.run_set(directory, self.compiler, self.root / "unstable", measurer=unstable)
        self.assertTrue((self.root / "unstable/small/1/out.lowir").is_file())
        with self.assertRaisesRegex(bench.BenchError, "small: exit status 101, expected 0"):
            bench.run_set(directory, self.compiler, self.root / "failed", measurer=Compiler([1], status=101))

    def receipts(self, *, lowir=b"other\n", instructions=(900, 900, 900, 50, 50, 50)):
        directory = self.frozen()
        with contextlib.redirect_stdout(io.StringIO()):
            base = bench.run_set(directory, self.compiler, self.root / "base",
                                 measurer=Compiler([1000, 1000, 1000, 50, 50, 50]))
            candidate = bench.run_set(directory, self.compiler, self.root / "candidate",
                                      measurer=Compiler(instructions, lowir=lowir))
        return base, candidate

    def test_compare_reports_deltas_for_identical_output(self):
        base, candidate = self.receipts(lowir=b"function @main\n")
        rows, problems, unshared = bench.compare(base, candidate)
        self.assertEqual((problems, unshared), ([], []))
        self.assertEqual([row["name"] for row in rows], ["small", "guest"])
        self.assertAlmostEqual(rows[0]["instructions"], -0.1)
        self.assertAlmostEqual(rows[0]["cycles"], -0.1)
        self.assertEqual((rows[1]["instructions"], rows[1]["same_output"]), (0.0, True))
        table = bench.render_comparison(rows, unshared)
        self.assertIn("-10.000%", table)
        self.assertNotIn("DIFFERENT", table)
        arguments = ["compare", str(self.root / "base/receipt.json"), str(self.root / "candidate/receipt.json")]
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(bench.main(arguments), 0)

    def test_compare_fails_on_any_output_difference(self):
        base, candidate = self.receipts()
        rows, problems, _ = bench.compare(base, candidate)
        self.assertEqual(problems, ["small: out.lowir differ"])
        self.assertEqual([row["same_output"] for row in rows], [False, True])
        arguments = ["compare", str(self.root / "base/receipt.json"), str(self.root / "candidate/receipt.json")]
        with contextlib.redirect_stdout(io.StringIO()) as table, contextlib.redirect_stderr(io.StringIO()) as failed:
            self.assertEqual(bench.main(arguments), 1)
        self.assertIn("DIFFERENT", table.getvalue())
        self.assertIn("FAILED: small: out.lowir differ", failed.getvalue())
        candidate["benchmarks"]["guest"]["produced"]["status"] = 0
        candidate["benchmarks"]["guest"]["produced"]["outputs"]["stderr"]["sha256"] = "0" * 64
        self.assertIn("guest: status, stderr differ", bench.compare(base, candidate)[1])

    def test_compare_refuses_other_sets_and_incomplete_receipts_and_names_one_sided_rows(self):
        base, candidate = self.receipts(lowir=b"function @main\n")
        del candidate["benchmarks"]["guest"]
        self.assertEqual(bench.compare(base, candidate)[1:], ([], ["guest"]))
        candidate["set"]["sha256"] = "0" * 64
        del candidate["complete"]
        candidate["benchmarks"]["small"]["argv"] = ["--other"]
        self.assertEqual(bench.compare(base, candidate)[1],
                         ["the candidate receipt is incomplete or has another schema",
                          "the receipts measured different benchmark sets", "small: arguments differ"])
        candidate["benchmarks"] = {}
        self.assertIn("the receipts share no benchmark", bench.compare(base, candidate)[1])

    def test_hardware_counters_measure_only_the_started_program(self):
        try:
            os.close(bench.counter(os.getpid(), bench.INSTRUCTIONS))
        except bench.BenchError as error:
            self.skipTest(str(error))
        script = self.root / "compiler.sh"
        script.write_text("#!/bin/sh\necho \"$ARCH $2\" > out.lowir\necho diagnostic >&2\nexit 0\n")
        script.chmod(0o755)
        plan = dict(self.plan, benchmarks=[{"name": "live", "argv": ["-o", "{set}/inputs/small.rs"],
                                            "env": {"ARCH": "x86_64"}, "outputs": ["out.lowir"]}])
        bench.freeze(plan, self.root / "live-set")
        with contextlib.redirect_stdout(io.StringIO()):
            receipt = bench.run_set(self.root / "live-set", script, self.root / "live", repeat=2)
        live = receipt["benchmarks"]["live"]
        self.assertGreater(live["instructions_min"], 10_000)
        self.assertLess(live["instructions_spread"], 0.05)
        self.assertGreater(live["cycles"], 0)
        self.assertTrue(100 < live["peak_rss_kib"] < 100_000)
        expected = f"x86_64 {(self.root / 'live-set').resolve()}/inputs/small.rs\n"
        self.assertEqual((self.root / "live/live/0/out.lowir").read_text(), expected)
        self.assertEqual((self.root / "live/live/0/stderr").read_text(), "diagnostic\n")
        (self.root / "unstarted").mkdir()
        with self.assertRaisesRegex(bench.BenchError, "cannot start"):
            bench.measure(self.root / "absent", [], {}, self.root / "unstarted", 5)
        self.assertIn("cannot start", (self.root / "unstarted/stderr").read_text())
        (self.root / "slow").mkdir()
        with self.assertRaisesRegex(bench.BenchError, "exceeded 0.2 seconds"):
            bench.measure("/bin/sleep", ["5"], {}, self.root / "slow", 0.2)


if __name__ == "__main__":
    unittest.main()
