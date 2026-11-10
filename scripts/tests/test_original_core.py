"""Reusing pinned sources must not weaken the whole-crate observer's integrity."""
import hashlib
import argparse
import io
import json
from pathlib import Path
from contextlib import redirect_stdout
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from observe_original_core import (
    CENSUS_HEADER, PROGRESS_HEADER, SOURCE_CONTRACT_DEFAULTS, census_summary, progress_summary, lint_profile_arguments, observe, observer_seconds,
    source_contract_profile_arguments, verify_sources, worker,
)


class OriginalCoreInputs(unittest.TestCase):
    def test_partial_progress_retains_only_complete_lines_and_is_never_a_census(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "progress.tsv"
            item = "item\tfunction\t9\tfile 0:17\tnode 2"
            path.write_text(PROGRESS_HEADER + "\n" + item + "\n"
                            "record\t1\tbodies\tfailed\tfn\tfile 0:17\t-\tincompatible types\n"
                            "record\t2\tbodies")
            self.assertEqual(progress_summary(path), {"analysis_only": True,
                "complete_census": False, "records": 1, "last_item": item})
            with self.assertRaises(ValueError):
                census_summary(path)

    def test_observer_window_is_bounded_by_the_worker_deadline(self):
        for value in ("0", "4", "171", "1000000"):
            with self.subTest(value=value), self.assertRaises(argparse.ArgumentTypeError):
                observer_seconds(value)
        for value in (5, 120, 170):
            self.assertEqual(observer_seconds(str(value)), value)
        for deadline, expected in ((190, 170), (150, 142)):
            with self.subTest(deadline=deadline), tempfile.TemporaryDirectory() as temp:
                process = Mock()
                process.wait.return_value = 1
                with patch("observe_original_core.subprocess.Popen", return_value=process), \
                        patch("observe_original_core.time.monotonic", return_value=0):
                    self.assertEqual(observe(["immutable-cli"], Path(temp), deadline, seconds=170), 1)
                process.wait.assert_called_once_with(timeout=expected)

    def test_explicit_resources_keep_uncapped_defaults_and_real_zero(self):
        args, limits = source_contract_profile_arguments({"profile": {}}, {})
        self.assertEqual(args, [])
        self.assertEqual(limits, SOURCE_CONTRACT_DEFAULTS)
        self.assertEqual(set(SOURCE_CONTRACT_DEFAULTS.values()), {None})
        limits["work"] = 7
        self.assertIsNone(SOURCE_CONTRACT_DEFAULTS["work"])
        requested = {"work": 14_500_624, "attempted_bytes": 0, "retained_bytes": 0xffff_ffff}
        args, effective = source_contract_profile_arguments(
            {"profile": {"source_contract_limits": requested}},
            {"source_contract_limits": dict(requested)})
        self.assertEqual(args, ["--source-contract-work-limit=14500624",
                                "--source-contract-attempted-bytes=0",
                                "--source-contract-retained-bytes=4294967295"])
        self.assertEqual(effective, requested)
        effective["work"] = 0
        self.assertEqual(requested["work"], 14_500_624)

    def test_resources_require_complete_matching_finite_owners(self):
        legacy = {"work": 1_000_000, "attempted_bytes": 67_108_864, "retained_bytes": 67_108_864}
        explicit = {"source_contract_limits": dict(legacy)}
        for profile, producer in (({}, explicit), (explicit, {})):
            with self.subTest(profile=profile), self.assertRaisesRegex(ValueError, "explicit"):
                source_contract_profile_arguments({"profile": profile}, producer)
        changed = {"source_contract_limits": {**legacy, "work": 2_000_000}}
        with self.assertRaisesRegex(ValueError, "differ"):
            source_contract_profile_arguments({"profile": explicit}, changed)
        invalid = [None, [], {}, {**legacy, "extra": 1}, dict(SOURCE_CONTRACT_DEFAULTS)]
        invalid.extend({**legacy, "work": value}
                       for value in (True, False, -1, 0x1_0000_0000, 1.0, "1000000", None))
        for value in invalid:
            owner = {"source_contract_limits": value}
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "finite"):
                source_contract_profile_arguments({"profile": owner}, owner)

    def test_worker_records_and_executes_the_declared_resource_profile(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / "source"
            path = source / "library/core/src/lib.rs"
            path.parent.mkdir(parents=True)
            path.write_bytes(b"pub fn f() {}")
            cli = root / "cli"
            cli.write_bytes(b"immutable test compiler")
            digest = hashlib.sha256(cli.read_bytes()).hexdigest()
            limits = {"work": 14_500_624, "attempted_bytes": 67_108_864, "retained_bytes": 67_108_864}
            binding = root / "producer.json"
            binding.write_text(json.dumps({"candidate_sha256": digest,
                                          "compiler_source_sha256": {},
                                          "source_contract_limits": limits}))
            manifest = root / "manifest.json"
            manifest.write_text(json.dumps({"revision": "fixture", "expected_cli_sha256": digest,
                "config_source_sha256": {}, "profile": {"source_contract_limits": limits},
                "files": [{"path": "library/core/src/lib.rs", "bytes": path.stat().st_size,
                           "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}]}))
            args = argparse.Namespace(cli=cli, producer_binding=binding, out=root / "out",
                                      manifest=manifest, source_root=source, cap_lints=None,
                                      expected_cli_sha256=None, census=False, observer_seconds=170)
            commands = []

            def observed(command, out, _deadline, *, seconds=120):
                self.assertEqual(seconds, 170)
                commands.append(command)
                (out / "stdout").write_bytes(b"")
                (out / "stderr").write_text("ERROR: fixture compilation boundary\n")
                (out / "whole-core.openat").write_bytes(b"immutable fake trace")
                return 1

            with patch("observe_original_core.shutil.which", return_value=sys.executable), \
                    patch("observe_original_core.observe", side_effect=observed), \
                    patch("observe_original_core.opened_sources", return_value=[{"path": str(path)}]), \
                    redirect_stdout(io.StringIO()):
                self.assertEqual(worker(args), 0)
            self.assertEqual(json.loads((args.out / "binding.json").read_text())["observer_deadline_seconds"], 170)
            self.assertEqual(json.loads((args.out / "result.json").read_text())["observer_deadline_seconds"], 170)
            self.assertEqual(len(commands), 1)
            for flag in ("--source-contract-work-limit=14500624",
                         "--source-contract-attempted-bytes=67108864",
                         "--source-contract-retained-bytes=67108864"):
                self.assertIn(flag, commands[0])
            for name in ("binding.json", "result.json"):
                self.assertEqual(json.loads((args.out / name).read_text())["source_contract_limits"], limits)
            self.assertEqual(json.loads((args.out / "result.json").read_text())["status"], 1)

    def test_census_is_analysis_only_and_requires_one_complete_failing_file(self):
        complete = "\n".join([
            CENSUS_HEADER,
            "record\t1\tdeclarations\tfailed\tstruct `A`\tfile 0:4\t-\tfixture failure",
            "record\t2\tdeclarations\tblocked\tfunction `b`\tfile 0:9\t1\tblocked",
            *(f"summary\t{phase}\tchecked\t1\tfailed\t0\tblocked\t0"
              for phase in ("frontend", "declarations", "bodies", "completion")),
            "terminal\t2\tdeclarations\tstopped", "end\t2", ""])
        for name, status, text, accepted in (
                ("complete", 1, complete, True), ("passing-status", 0, complete, False),
                ("truncated", 1, complete.rsplit("end", 1)[0], False), ("missing", 1, None, False),
                ("timeout", None, None, False), ("invalid-progress", 1, complete, False)):
            with self.subTest(name), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                source = root / "source"
                path = source / "library/core/src/lib.rs"
                path.parent.mkdir(parents=True)
                path.write_bytes(b"pub fn f() {}")
                cli = root / "cli"
                cli.write_bytes(b"immutable test compiler")
                digest = hashlib.sha256(cli.read_bytes()).hexdigest()
                binding = root / "producer.json"
                binding.write_text(json.dumps({"candidate_sha256": digest, "compiler_source_sha256": {}}))
                manifest = root / "manifest.json"
                manifest.write_text(json.dumps({"revision": "fixture", "expected_cli_sha256": digest,
                    "config_source_sha256": {}, "profile": {},
                    "files": [{"path": "library/core/src/lib.rs", "bytes": path.stat().st_size,
                               "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}]}))
                args = argparse.Namespace(cli=cli, producer_binding=binding, out=root / "out",
                                          manifest=manifest, source_root=source, cap_lints=None,
                                          expected_cli_sha256=None, census=True, progress=True)
                commands = []

                def observed(command, out, _deadline, *, seconds=120, text=text, status=status):
                    commands.append(command)
                    census = out / "census.tsv"
                    if text is not None:
                        census.write_text(text)
                    (out / "stdout").write_bytes(b"")
                    (out / "stderr").write_text(
                        "census declarations: checked 1 failed 1 blocked 1\n"
                        f"ERROR: analysis-only census written to {census}; a census is never a passing build\n")
                    (out / "whole-core.openat").write_bytes(b"immutable fake trace")
                    header = CENSUS_HEADER if name == "invalid-progress" else PROGRESS_HEADER
                    (out / "progress.tsv").write_text(header + "\n"
                        "item\tfunction\t9\tfile 0:17\tnode 2\n")
                    if status is None:
                        raise TimeoutError("fixture observer deadline")
                    return status

                with patch("observe_original_core.shutil.which", return_value=sys.executable), \
                        patch("observe_original_core.observe", side_effect=observed), \
                        patch("observe_original_core.opened_sources", return_value=[{"path": str(path)}]), \
                        redirect_stdout(io.StringIO()):
                    self.assertEqual(worker(args), 0 if accepted else 1)
                index = commands[0].index("--census-output")
                self.assertEqual(commands[0][index + 1], str(args.out.resolve() / "census.tsv"))
                result = json.loads((args.out / "result.json").read_text())
                self.assertIn("--census-progress", commands[0])
                if name == "invalid-progress":
                    self.assertIn("progress_error", result)
                    self.assertEqual(result["classification"], "infrastructure/progress failure")
                else:
                    self.assertFalse(result["partial_progress"]["complete_census"])
                self.assertIs(result["infrastructure_failure"], not accepted)
                if accepted:
                    self.assertEqual(result["classification"], "analysis-only census; not a qualification")
                    self.assertIs(result["qualification"], False)
                    self.assertEqual(result["census"]["categories"], {"failed": 1, "blocked": 1})
                    self.assertEqual(result["census"]["terminal"],
                                     {"record": "2", "phase": "declarations", "category": "stopped"})

    def test_strict_observation_cannot_silently_reuse_a_capped_profile_or_producer(self):
        for profile, producer in (({}, {}), ({"lint_cap": "uncapped"}, {"lint_cap": "uncapped"})):
            self.assertEqual(lint_profile_arguments({"profile": profile}, producer, None), [])
        for profile, producer in (({"lint_cap": "allow"}, {}), ({}, {"lint_cap": "allow"}),
                                  ({"lint_cap": "allow"}, {"lint_cap": "allow"})):
            with self.subTest(profile=profile, producer=producer), self.assertRaises(ValueError):
                lint_profile_arguments({"profile": profile}, producer, None)

    def test_capped_observation_requires_both_explicit_owners(self):
        self.assertEqual(lint_profile_arguments(
            {"profile": {"lint_cap": "allow"}}, {"lint_cap": "allow"}, "allow"),
            ["--cap-lints=allow"])
        for profile, producer in (({}, {}), ({"lint_cap": "allow"}, {}),
                                  ({}, {"lint_cap": "allow"})):
            with self.subTest(profile=profile, producer=producer), self.assertRaises(ValueError):
                lint_profile_arguments({"profile": profile}, producer, "allow")
        with self.assertRaisesRegex(ValueError, "unsupported"):
            lint_profile_arguments({"profile": {}}, {}, "warn")

    def test_all_files_are_checked_even_when_the_compiler_never_opens_them(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = {"files": []}
            for name in ("lib.rs", "not_yet_opened.rs"):
                path = root / name
                data = b"pub fn f() {}"
                path.write_bytes(data)
                manifest["files"].append({"path": name, "bytes": len(data),
                                          "sha256": hashlib.sha256(data).hexdigest()})
            verify_sources(manifest, root)
            (root / "not_yet_opened.rs").write_bytes(b"pub fn g() {}")
            with self.assertRaisesRegex(ValueError, "changed pinned source"):
                verify_sources(manifest, root)
            (root / "not_yet_opened.rs").unlink()
            with self.assertRaisesRegex(ValueError, "missing or changed"):
                verify_sources(manifest, root)

    def test_paths_and_links_cannot_escape_the_verified_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "source"
            root.mkdir()
            outside = root.parent / "outside.rs"
            outside.write_bytes(b"source")
            (root / "link.rs").symlink_to(outside)
            for path in ("../outside.rs", str(outside), "link.rs"):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    verify_sources({"files": [{"path": path, "bytes": 6,
                                               "sha256": hashlib.sha256(b"source").hexdigest()}]}, root)
