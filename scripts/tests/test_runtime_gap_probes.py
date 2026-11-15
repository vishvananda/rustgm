"""The runtime gap probes stay well-formed without running any tool."""
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import runtime_gap_probes as probes


class RuntimeGapProbes(unittest.TestCase):
    def test_every_probe_is_named_once_and_declares_an_observation(self):
        names = [probe[0] for probe in probes.FRONTEND + probes.BACKEND]
        self.assertEqual(len(names), len(set(names)))
        for probe in probes.FRONTEND + probes.BACKEND:
            self.assertTrue(all(isinstance(part, str) and part.strip() for part in probe), probe[0])

    def test_backend_probes_are_single_entry_programs(self):
        for name, text, _, _ in probes.BACKEND:
            self.assertEqual(text.count("[role=entry]"), 1, name)
            self.assertTrue(text.endswith("}\n"), name)

    def test_a_failed_build_is_the_observation(self):
        def refuse(_source, _output):
            raise ValueError("backend command failed (1): ERROR: undefined symbol:\n write")

        with tempfile.TemporaryDirectory() as folder:
            observed = probes.built(refuse, Path(folder) / "unit.lowir", Path(folder) / "unit")
        self.assertEqual(observed, "backend command failed (1): ERROR: undefined symbol: write")

    def test_clearing_removes_only_the_scratch_files(self):
        with tempfile.TemporaryDirectory() as folder:
            scratch = Path(folder) / "scratch"
            scratch.mkdir()
            (scratch / "unit.o").write_bytes(b"")
            (Path(folder) / "observations.tsv").write_text("kept\n")
            probes.clear(scratch)
            self.assertEqual([path.name for path in Path(folder).iterdir() if path.is_file()], ["observations.tsv"])
            self.assertEqual(list(scratch.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
