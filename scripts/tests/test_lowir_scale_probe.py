"""The scale probe renames exactly a unit's own symbols and keeps one entry."""
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import lowir_scale_probe as probe

UNIT = b"""global @rg.data.0 : obj<8x8> [storage=readonly] = {
  ptr addr @rg.1
}
function @rg.1(%p0 : i64) -> void {
  block ^b0:
    call void @abort()
    unreachable
}
declare function @abort() -> void [return=noreturn]
function @main() -> i64 [role=entry] {
  block ^b0:
    call void @rg.1(0)
    return i64 0
}
"""


class ScaleProbe(unittest.TestCase):
    def test_a_unit_is_split_into_what_it_defines_and_declares(self):
        unit = probe.parse(UNIT)
        self.assertEqual(unit["entry"], b"main")
        self.assertEqual(unit["counts"][:2], (2, 1))
        self.assertEqual(unit["declarations"], [b"declare function @abort() -> void [return=noreturn]\n"])
        functions = b".r7".join(unit["functions"])
        self.assertIn(b"function @rg.1.r7(%p0 : i64) -> void {", functions)
        self.assertIn(b"function @main.r7() -> i64 {", functions)
        self.assertIn(b"call void @rg.1.r7(0)", functions)
        self.assertIn(b"call void @abort()", functions)
        self.assertNotIn(b"role=entry", functions)
        self.assertEqual(b".r7".join(unit["globals"]),
                         b"global @rg.data.0.r7 : obj<8x8> [storage=readonly] = {\n  ptr addr @rg.1.r7\n}\n")

    def test_units_the_probe_cannot_replicate_are_refused(self):
        for text in (b"function @f() -> void {\n  block ^b0:\n    return\n}\n",
                     b"assembly @asm0 intel readwrite 0 0 0 0 {\n}\n" + UNIT,
                     UNIT.replace(b"-> i64 [role=entry]", b"-> i32 [role=entry]"),
                     UNIT.replace(b"obj<8x8>", b"obj<16000000x8>"),
                     UNIT[:-2]):
            with self.assertRaises(probe.Unsuitable):
                probe.parse(text)

    def test_the_driver_calls_every_entry_once_and_has_one_root(self):
        entries = [b"main.r%d" % index for index in range(probe.FAN_OUT * 2 + 3)]
        tree = b"".join(probe.driver(entries))
        for entry in entries:
            self.assertEqual(tree.count(b"@" + entry + b"()"), 1)
        self.assertEqual(tree.count(b"[role=entry]"), 1)
        self.assertEqual(tree.count(b"function @probe.g0."), 3)
        self.assertIn(b"function @main() -> i64 [role=entry] {", tree)
        self.assertEqual(b"".join(probe.driver([b"main.r0"])),
                         b"function @main() -> i64 [role=entry] {\n  block ^b0:\n"
                         b"    %t0 = call i64 @main.r0()\n    return i64 %t0\n}\n")

    def test_a_composed_program_declares_once_and_counts_what_it_wrote(self):
        units = [probe.parse(UNIT), probe.parse(UNIT)]
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "program.lowir"
            shape = probe.compose(units, 3, path)
            text = path.read_bytes()
        self.assertEqual(shape["replicas"], 6)
        self.assertEqual(text.count(b"declare function @abort()"), 1)
        self.assertEqual(text.count(b"\nfunction @") + text.startswith(b"function @"), shape["functions"])
        self.assertEqual(text.count(b"global @"), shape["globals"])
        self.assertEqual((len(text), text.count(b"\n")), (shape["bytes"], shape["lines"]))
        self.assertLess(text.rindex(b"global @"), text.index(b"declare function"))
        conflicting = probe.parse(UNIT.replace(b"@abort() -> void [return=noreturn]", b"@abort() -> void"))
        with self.assertRaises(probe.Unsuitable), tempfile.TemporaryDirectory() as folder:
            probe.compose([units[0], conflicting], 1, Path(folder) / "program.lowir")


if __name__ == "__main__":
    unittest.main()
