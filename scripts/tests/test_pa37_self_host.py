"""Verify PA37 self-hosting student discovery, tree copying, and negative controls."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import build_self_hosting as whole
import pa37_self_host


class Pa37SelfHost(unittest.TestCase):
    def test_copy_tree_regular_skips_symlinks_and_scratch_dirs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            src, dst = root / "src", root / "dst"
            (src / "sub").mkdir(parents=True)
            (src / ".git").mkdir()
            (src / "target").mkdir()
            (src / "sub/a.rs").write_text("fn main() {}\n")
            (src / ".git/HEAD").write_text("ref\n")
            (src / "target/out").write_text("bin\n")
            (src / ".rustgm-verified-sha256").write_text("deadbeef\n")
            (src / "link.rs").symlink_to(src / "sub/a.rs")
            pa37_self_host.copy_tree_regular(src, dst)
            self.assertTrue((dst / "sub/a.rs").is_file())
            self.assertFalse((dst / ".git").exists())
            self.assertFalse((dst / "target").exists())
            self.assertFalse((dst / ".rustgm-verified-sha256").exists())
            self.assertFalse((dst / "link.rs").exists())

    def test_discover_student_configuration_from_cargo_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "student"
            sources = Path(directory) / "sources"
            (root / "src").mkdir(parents=True)
            (root / "Cargo.toml").write_text(
                '[package]\nname = "rustgm-student"\nversion = "0.1.0"\nedition = "2024"\n\n'
                '[lib]\nname = "rustgm"\npath = "src/lib.rs"\n\n'
                '[[bin]]\nname = "rustgm"\npath = "src/main.rs"\n'
            )
            (root / "Cargo.lock").write_text(
                'version = 4\n\n[[package]]\nname = "rustgm-student"\nversion = "0.1.0"\n'
            )
            (root / "src/lib.rs").write_text("pub fn answer() -> u32 { 42 }\n")
            (root / "src/main.rs").write_text("fn main() { let _ = rustgm::answer(); }\n")
            self.assertFalse(pa37_self_host.is_maintainer_repo(root))
            config = pa37_self_host.discover_student_configuration(root, sources)
            guest_names = [g["name"] for g in config["guests"]]
            self.assertEqual(guest_names[-2:], ["rustgm", "app"])
            self.assertTrue((sources / "src/main.rs").is_file())
            # Validated by build_self_hosting's strict configuration checker.
            whole.source_arguments(config)

    def test_negative_controls_reject_perturbed_lowir_and_bringup_stderr(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            config = {
                "core": {"path": "core/lib.rs", "disambiguator": "core_test", "cfg": []},
                "guests": [{"name": "app", "edition": "2024", "path": "app/main.rs"}],
            }
            arguments, roots = whole.source_arguments(config)
            graph = {
                "format": whole.FORMAT,
                "configuration": config,
                "source_sha256": "0" * 64,
                "stage0_sha256": "1" * 64,
            }
            for stage in ("stage1", "stage2"):
                tdir = work / stage / "target"
                ndir = work / stage / "native"
                tdir.mkdir(parents=True)
                ndir.mkdir(parents=True)
                lowir = tdir / "program.lowir"
                links = tdir / "program.native-links.json"
                exe = ndir / "rustgm"
                lowir.write_bytes(b"lowir-module-v1\n")
                links.write_text('{"schema": 1, "libraries": []}\n')
                exe.write_bytes(b"\x7fELFstage-binary")
                (tdir / "compile.stdout").write_bytes(b"")
                (tdir / "compile.stderr").write_bytes(b"")
                compiler_path = str(work / "stage1/native/rustgm") if stage == "stage2" else str(work / "stage0")
                compiler_sha = pa37_self_host.digest(work / "stage1/native/rustgm") if stage == "stage2" else "1" * 64
                receipt = {
                    "format": whole.FORMAT,
                    "state": "complete",
                    "profile": "strict",
                    "bringup_records": 0,
                    "graph": graph,
                    "implementation_sha256": {},
                    "rustgm": {"path": compiler_path, "bytes": 1, "sha256": compiler_sha},
                    "environment": whole.COMPILER_ENVIRONMENT,
                    "command": {
                        "argv": whole.command_line(compiler_path, arguments, roots[0], tdir),
                        "status": 0,
                        "timed_out": False,
                        "signal": None,
                    },
                    "outputs": {
                        "lowir": {"path": str(lowir), "bytes": lowir.stat().st_size,
                                  "sha256": pa37_self_host.digest(lowir)},
                        "native_links": {"path": str(links), "bytes": links.stat().st_size,
                                         "sha256": pa37_self_host.digest(links)},
                    },
                }
                (tdir / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
            result = pa37_self_host.do_negative_controls(work)
            self.assertTrue(result["perturbed_stage2_rejected"])
            self.assertTrue(result["bringup_record_rejected"])


if __name__ == "__main__":
    unittest.main()
