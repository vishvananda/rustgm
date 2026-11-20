"""The bring-up census grows a skip list from the compiler's own attribution."""
from pathlib import Path
import contextlib
import io
import json
import sys
import tempfile
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import bringup_census as census

# Fails until the list names `broken`, then compiles while naming one waived
# fence and one function that kept its body without a proof.
COMPILER = """#!/usr/bin/env python3
import sys
listed = open(sys.argv[sys.argv.index("--bringup-skip-list") + 1]).read()
if "a.rs\\t7\\t" not in listed:
    sys.stderr.write("bringup-blocker\\ta.rs\\t7\\tbroken\\town cause\\tborrows\\n")
    sys.stderr.write("ERROR: file 3: 9: last cause\\n")
    sys.exit(1)
sys.stderr.write("bringup-waived\\tsome fence\\n")
sys.stderr.write("bringup-unproven\\tb.rs\\t4\\tkept\\tsome proof failed\\n")
"""
# Always fails in a function that the list already names.
AGAIN = """#!/usr/bin/env python3
import sys
sys.stderr.write("bringup-blocker\\ta.rs\\t8\\tbroken\\town cause\\n")
sys.stderr.write("ERROR: file 3: 9: last cause\\n")
sys.exit(1)
"""


class Census(unittest.TestCase):
    def test_marked_lines_are_read_and_other_text_is_ignored(self):
        stderr = ("note\nbringup-waived\tfence one\n"
                  "bringup-unproven\tb.rs\t4\tkept\tsome proof failed\n"
                  "bringup-blocker\ta.rs\t7\tbroken\town cause\tlowering\n"
                  "bringup-blocker\ta.rs\t9\tolder\tno stage\n"
                  "bringup-blocker\ta.rs\tx\tmalformed\n"
                  "bringup-unattributed\tno owner\nERROR: file 3: 9: last cause\n")
        self.assertEqual(census.waived(stderr), ["fence one"])
        self.assertEqual(census.unproven(stderr), ["b.rs\t4\tkept\tsome proof failed"])
        self.assertEqual(census.diagnose(stderr),
                         ([("a.rs", 7, "broken", "own cause", "lowering"),
                           ("a.rs", 9, "older", "no stage", None)], ["no owner"],
                          "file 3: 9: last cause"))
        self.assertEqual(census.cause("file 3: 9: last cause"), "last cause")
        self.assertEqual(census.cause(None), "no error line")

    def run_census(self, folder, compiler):
        program = folder / "compiler.py"
        program.write_text(compiler)
        program.chmod(0o755)
        arguments = ["census", "--compiler", str(program), "--skip-list",
                     str(folder / "skip.tsv"), "--out", str(folder / "out"),
                     "--", "crate.rs"]
        printed = io.StringIO()
        with mock.patch.object(sys, "argv", arguments), contextlib.redirect_stdout(printed):
            status = census.main()
        return status, printed.getvalue(), folder / "out"

    def test_the_list_grows_until_the_crate_compiles_and_records_what_was_passed_over(self):
        with tempfile.TemporaryDirectory() as folder:
            folder = Path(folder)
            status, printed, out = self.run_census(folder, COMPILER)
            self.assertEqual(status, 0)
            self.assertEqual((folder / "skip.tsv").read_text(),
                             "a.rs\t7\tbroken\town cause\tborrows\n")
            self.assertEqual((out / "waived.tsv").read_text(), "some fence\n")
            self.assertEqual((out / "unproven.tsv").read_text(),
                             "b.rs\t4\tkept\tsome proof failed\n")
            self.assertEqual((out / "summary.tsv").read_text(), "1\town cause\n")
            records = json.loads((out / "records.json").read_text())
            self.assertEqual(records["outcome"], "compiled")
            self.assertEqual(records["unproven"], ["b.rs\t4\tkept\tsome proof failed"])
            self.assertIn("listed 1; distinct causes 1; waived fences 1; unproven bodies 1",
                          printed)

    def test_a_failure_that_names_no_new_function_stops_the_census(self):
        with tempfile.TemporaryDirectory() as folder:
            folder = Path(folder)
            (folder / "skip.tsv").write_text("# a different function\na.rs\t8\tother\n")
            status, printed, out = self.run_census(folder, AGAIN)
            self.assertEqual(status, 1)
            self.assertIn("outcome: listed function failed again", printed)
            self.assertIn("repeated: a.rs:8 broken: own cause", printed)
            self.assertEqual(json.loads((out / "records.json").read_text())["added"], 0)


if __name__ == "__main__":
    unittest.main()
