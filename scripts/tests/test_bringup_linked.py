"""The linked bring-up driver adds a consumer to core's session without changing either."""
import argparse
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import bringup_linked as linked

PROGRAM = """#![no_std]
#![no_main]

unsafe extern "C" {
    fn abort() -> !;
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
"""


class Linked(unittest.TestCase):
    def test_the_guest_keeps_its_text_and_gains_a_rust_entry(self):
        guest = linked.guest(PROGRAM)
        self.assertNotIn("#![no_std]", guest)
        self.assertNotIn("#![no_main]", guest)
        self.assertNotIn("no_mangle", guest)
        self.assertIn('pub(crate) extern "C" fn guest_main(_argc: i32', guest)
        self.assertTrue(guest.endswith(linked.ADAPTER))
        # Everything else is the program's own text, in order.
        body = PROGRAM.replace("#![no_std]\n", "").replace("#![no_main]\n", "")
        self.assertEqual(guest[:-len(linked.ADAPTER)],
                         body.replace(linked.ENTRY, linked.GUEST_ENTRY))

    def test_a_program_without_one_exported_main_or_abort_is_refused(self):
        with self.assertRaisesRegex(ValueError, "exactly one exported C main"):
            linked.guest(PROGRAM.replace("#[unsafe(no_mangle)]\n", ""))
        with self.assertRaisesRegex(ValueError, "abort declaration"):
            linked.guest(PROGRAM.replace("fn abort() -> !;", ""))

    def test_the_host_root_is_cores_bytes_followed_by_the_mount(self):
        root = "//! core\n#![no_core]\npub mod num;\n"
        self.assertEqual(linked.host(root), root + linked.MOUNT)
        self.assertIn("use __guest_app::main;", linked.MOUNT)

    def test_preparation_copies_the_library_once_and_remounts_each_program(self):
        with tempfile.TemporaryDirectory() as folder:
            folder = Path(folder)
            source = folder / "frozen" / "library" / "core" / "src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text("//! core\n")
            (source / "num.rs").write_text("// num\n")
            (source / "lib.rs").chmod(0o444)
            program = folder / "one.rs"
            program.write_text(PROGRAM)
            work = folder / "work"
            work.mkdir()
            root = linked.prepare(work, folder / "frozen" / "library", program)
            self.assertEqual(root.read_text(), "//! core\n" + linked.MOUNT)
            self.assertEqual((work / "library/core/src/num.rs").read_text(), "// num\n")
            self.assertEqual((work / "app/main.rs").read_text(), linked.guest(PROGRAM))
            # The frozen input is untouched, and a second program replaces the first.
            self.assertEqual((source / "lib.rs").read_text(), "//! core\n")
            program.write_text(PROGRAM.replace("    0\n", "    1\n"))
            linked.prepare(work, folder / "frozen" / "library", program)
            self.assertIn("    1\n", (work / "app/main.rs").read_text())
            self.assertEqual(root.read_text(), "//! core\n" + linked.MOUNT)

    def test_guest_mode_passes_the_unchanged_program_beside_frozen_core(self):
        with tempfile.TemporaryDirectory() as folder:
            folder = Path(folder)
            source = folder / "frozen" / "library" / "core" / "src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text("//! core\n")
            program = folder / "one.rs"
            program.write_text(PROGRAM)
            work = folder / "work"
            work.mkdir()
            options = argparse.Namespace(guest=True, work=work,
                                         library=folder / "frozen" / "library")
            root, extra = linked.session(options, program)
            self.assertEqual(root, source / "lib.rs")
            self.assertEqual(extra, ["--bringup-guest", f"app={program}"])
            # Nothing is copied, mounted or rewritten.
            self.assertEqual(list(work.iterdir()), [])
            self.assertEqual((source / "lib.rs").read_text(), "//! core\n")
            self.assertEqual(program.read_text(), PROGRAM)
            # Without the mode the same call mounts the program in a copy.
            options.guest = False
            root, extra = linked.session(options, program)
            self.assertEqual(root, work / "library" / "core" / "src" / "lib.rs")
            self.assertEqual(extra, [])
            self.assertEqual(root.read_text(), "//! core\n" + linked.MOUNT)

    def test_skip_rows_are_keyed_by_path_and_offset(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "skip.tsv"
            path.write_text("# note\n\na.rs\t3\tf\tcause\nb.rs\t9\n")
            self.assertEqual(linked.rows(path),
                             {("a.rs", "3"): "a.rs\t3\tf\tcause", ("b.rs", "9"): "b.rs\t9"})


if __name__ == "__main__":
    unittest.main()
