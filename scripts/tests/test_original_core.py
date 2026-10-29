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
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from observe_original_core import (
    SOURCE_CONTRACT_DEFAULTS, lint_profile_arguments, source_contract_profile_arguments,
    verify_sources, worker,
)


class OriginalCoreInputs(unittest.TestCase):
    def test_explicit_resources_keep_legacy_defaults_and_real_zero(self):
        args, limits = source_contract_profile_arguments({"profile": {}}, {})
        self.assertEqual(args, [])
        self.assertEqual(limits, SOURCE_CONTRACT_DEFAULTS)
        limits["work"] = 7
        self.assertEqual(SOURCE_CONTRACT_DEFAULTS["work"], 1_000_000)
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
        explicit = {"source_contract_limits": dict(SOURCE_CONTRACT_DEFAULTS)}
        for profile, producer in (({}, explicit), (explicit, {})):
            with self.subTest(profile=profile), self.assertRaisesRegex(ValueError, "explicit"):
                source_contract_profile_arguments({"profile": profile}, producer)
        changed = {"source_contract_limits": {**SOURCE_CONTRACT_DEFAULTS, "work": 2_000_000}}
        with self.assertRaisesRegex(ValueError, "differ"):
            source_contract_profile_arguments({"profile": explicit}, changed)
        invalid = [None, [], {}, {**SOURCE_CONTRACT_DEFAULTS, "extra": 1}]
        invalid.extend({**SOURCE_CONTRACT_DEFAULTS, "work": value}
                       for value in (True, False, -1, 0x1_0000_0000, 1.0, "1000000"))
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
                                      expected_cli_sha256=None)
            commands = []

            def observed(command, out, _deadline):
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
            self.assertEqual(len(commands), 1)
            for flag in ("--source-contract-work-limit=14500624",
                         "--source-contract-attempted-bytes=67108864",
                         "--source-contract-retained-bytes=67108864"):
                self.assertIn(flag, commands[0])
            for name in ("binding.json", "result.json"):
                self.assertEqual(json.loads((args.out / name).read_text())["source_contract_limits"], limits)
            self.assertEqual(json.loads((args.out / "result.json").read_text())["status"], 1)

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
