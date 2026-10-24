"""Batching must preserve every requested test, ignore lane and failure."""
from contextlib import redirect_stdout
import io
import json
import os
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from maintainer_batch import artifacts, build_command, inventory, package_environment, run_batch, selection


def result(stdout="", code=0):
    return SimpleNamespace(stdout=stdout, stderr="", returncode=code)


class MaintainerBatchTests(unittest.TestCase):
    def args(self, **kwargs):
        return SimpleNamespace(**dict({"suites": ["loans", "values::one"], "lib_case": [],
            "release": False, "all_features": False, "ignored": False, "list": False,
            "evidence_root": "obj/batch"}, **kwargs))

    def artifact(self, root, group="suite_memory"):
        (root / "dev").mkdir(exist_ok=True)
        (root / "dev/Cargo.toml").write_text('[package]\nname = "rustgm"\nversion = "0.1.0"\n')
        binary = root / group
        binary.write_bytes(b"test executable")
        return json.dumps({"reason": "compiler-artifact", "profile": {"test": True},
                           "target": {"kind": ["test"], "name": group}, "executable": str(binary)})

    def test_one_build_per_group_and_private_target(self):
        groups = selection({"loans": "suite_memory", "values": "suite_memory"},
                           ["loans", "values::one"], ["body::owner"])
        command = build_command(groups, self.args(release=True))
        self.assertEqual(command.count("--test"), 1)
        self.assertEqual(command.count("--lib"), 1)
        self.assertIn("--release", command)
        self.assertIn("--no-run", command)
        for names, private in (([], []), (["loans", "loans"], []), (["unknown"], []),
                               (["loans::"], []), ([], ["owner"])):
            with self.subTest(names=names), self.assertRaises(ValueError):
                selection({"loans": "suite_memory"}, names, private)

    def test_selection_excludes_lookalikes_and_rejects_overlapping_requests(self):
        listing = "loans::one: test\nloans::two: test\ntemplate_loans::one: test\n"
        self.assertEqual(inventory(listing, "suite_memory", [("loans", None)]),
                         ["loans::one", "loans::two"])
        self.assertEqual(inventory(listing, "lib", ["loans::one"]), ["loans::one"])
        with self.assertRaisesRegex(ValueError, "overlapping"):
            inventory(listing, "suite_memory", [("loans", None), ("loans", "one")])

    def test_missing_duplicate_and_nonexistent_executables_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            row = self.artifact(root)
            self.assertEqual(set(artifacts(row, {"suite_memory": []})), {"suite_memory"})
            for output, groups in ((row + "\n" + row, {"suite_memory": []}),
                                   (row, {"suite_other": []}), ("", {"suite_memory": []})):
                with self.subTest(output=output), self.assertRaises(ValueError):
                    artifacts(output, groups)
            (root / "suite_memory").unlink()
            with self.assertRaisesRegex(ValueError, "missing"):
                artifacts(row, {"suite_memory": []})

    def test_batch_builds_once_and_executes_combined_regular_inventory(self):
        with (tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()),
              patch.dict(os.environ, {"CARGO_PKG_NAME": "unrelated-parent", "RUST_TEST_THREADS": "2"})):
            root = Path(directory)
            runner = Mock(side_effect=[result(self.artifact(root)),
                result("loans::one: test\nloans::native: test\nvalues::one: test\n"),
                result("loans::native: test\n"),
                result("test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out;\n")])
            self.assertEqual(run_batch({"loans": "suite_memory", "values": "suite_memory"},
                                      self.args(), root=root, runner=runner), 0)
            calls = [call.args[0] for call in runner.call_args_list]
            self.assertEqual(sum(command[0] == "cargo" for command in calls), 1)
            self.assertEqual(calls[-1][1:], ["--exact", "loans::one", "values::one"])
            self.assertEqual(runner.call_args_list[-1].kwargs["cwd"], root / "dev")
            self.assertNotIn("env", runner.call_args_list[0].kwargs)
            for call in runner.call_args_list[1:]:
                self.assertEqual(call.kwargs["env"]["CARGO_PKG_NAME"], "rustgm")
                self.assertEqual(call.kwargs["env"]["CARGO_MANIFEST_DIR"], str(root / "dev"))
                self.assertEqual(call.kwargs["env"]["RUST_TEST_THREADS"], "2")
            self.assertEqual(os.environ["CARGO_PKG_NAME"], "unrelated-parent")
            receipt = json.loads((root / "obj/batch/receipt.json").read_text())
            self.assertEqual(receipt["state"], "passed")
            self.assertEqual(len(receipt["binaries"]["suite_memory"]["sha256"]), 64)
            self.assertEqual(receipt["package_environment"]["CARGO_PKG_NAME"], "rustgm")
            self.assertNotIn("RUST_TEST_THREADS", receipt["package_environment"])

    def test_package_environment_handles_missing_fields_and_prerelease_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.artifact(root)
            manifest = root / "dev/Cargo.toml"
            manifest.write_text('[package]\nname = "rustgm"\nversion = "1.2.3-beta.4+build"\n'
                                'authors = ["One", "Two"]\nrust-version = "1.98"\n')
            environment = package_environment(root)
            self.assertEqual(environment["CARGO_PKG_VERSION_PRE"], "beta.4")
            self.assertEqual(environment["CARGO_PKG_VERSION_PATCH"], "3")
            self.assertEqual(environment["CARGO_PKG_AUTHORS"], "One:Two")
            self.assertEqual(environment["CARGO_PKG_RUST_VERSION"], "1.98")
            self.assertEqual(environment["CARGO_PKG_DESCRIPTION"], "")
            self.assertEqual(environment["CARGO_MANIFEST_PATH"], str(manifest))
            for version in ('"invalid"', '{workspace = true}'):
                manifest.write_text('[package]\nname = "rustgm"\nversion = ' + version + '\n')
                with self.subTest(version=version), self.assertRaises(ValueError):
                    package_environment(root)

    def test_native_failure_is_retained_and_never_cached_as_passed(self):
        with tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()):
            root = Path(directory)
            runner = Mock(side_effect=[result(self.artifact(root)),
                result("loans::native: test\n"),
                result("test result: FAILED. 0 passed; 1 failed; 0 ignored;\n", 101)])
            self.assertEqual(run_batch({"loans": "suite_memory"},
                self.args(suites=["loans"], ignored=True), root=root, runner=runner), 1)
            self.assertEqual(runner.call_args_list[-1].args[0][1:],
                             ["--ignored", "--exact", "loans::native"])
            receipt = json.loads((root / "obj/batch/receipt.json").read_text())
            self.assertEqual(receipt["state"], "failed")
            self.assertEqual(receipt["steps"][-1]["status"], 101)

    def test_empty_execution_and_missing_selection_cannot_pass(self):
        for listing in ("loans::one: test\n", "unrelated::one: test\n"):
            with tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()):
                root = Path(directory)
                runner = Mock(side_effect=[result(self.artifact(root)), result(listing), result(),
                    result("test result: ok. 0 passed; 0 failed; 0 ignored;\n")])
                with self.assertRaises(ValueError):
                    run_batch({"loans": "suite_memory"}, self.args(suites=["loans"]),
                              root=root, runner=runner)
                receipt = json.loads((root / "obj/batch/receipt.json").read_text())
                self.assertEqual(receipt["state"], "failed")

    def test_failed_build_keeps_logs_without_launching_a_test(self):
        with tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()):
            runner = Mock(return_value=result("compiler error", 101))
            root = Path(directory)
            self.assertEqual(run_batch({"loans": "suite_memory", "values": "suite_memory"},
                                      self.args(), root=root, runner=runner), 101)
            self.assertEqual(runner.call_count, 1)
            receipt = json.loads((root / "obj/batch/receipt.json").read_text())
            self.assertEqual(receipt["state"], "failed")

    def test_completed_target_cleanup_preserves_failed_outputs_evidence_and_cache(self):
        with tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()):
            root = Path(directory)
            rows = "\n".join(self.artifact(root, name)
                             for name in ("suite_memory", "suite_other"))
            folder = root / "obj/generated"
            folder.mkdir(parents=True)
            older = folder / "older-failure.o"
            older.write_bytes(b"retained earlier failure")
            os.utime(older, (1, 1))
            cache = root / "obj/rust/debug/cache.o"
            cache.parent.mkdir(parents=True)
            passed = folder / "passed.o"
            failed = folder / "failed.o"
            source = folder / "case.rs"

            def runner(command, **_kwargs):
                if command[0] == "cargo":
                    return result(rows)
                first = Path(command[0]).name == "suite_memory"
                name = "loans::native" if first else "values::native"
                if "--list" in command:
                    return result(name + ": test\n")
                if first:
                    passed.write_bytes(b"completed native object")
                    source.write_text("fn main() {}")
                    cache.write_bytes(b"reusable build object")
                    return result("test result: ok. 1 passed; 0 failed; 0 ignored;\n")
                self.assertFalse(passed.exists())
                self.assertTrue(older.exists() and source.exists() and cache.exists())
                failed.write_bytes(b"preserve failed target")
                return result("test result: FAILED. 0 passed; 1 failed; 0 ignored;\n", 101)

            status = run_batch({"loans": "suite_memory", "values": "suite_other"},
                              self.args(suites=["loans", "values"], ignored=True),
                              root=root, runner=runner)
            self.assertEqual(status, 1)
            self.assertTrue(failed.exists())
            receipt = json.loads((root / "obj/batch/receipt.json").read_text())
            self.assertEqual(receipt["state"], "failed")
            self.assertEqual([row["group"] for row in receipt["cleanup"]], ["suite_memory"])
            self.assertTrue((root / "obj/batch/suite_other.stderr").exists())
