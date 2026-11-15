"""Production fixture access stays forbidden while test inputs remain usable."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from rust_audit import inspect_file


class AuditBoundaries(unittest.TestCase):
    def inspect(self, relative, source):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / relative
            path.parent.mkdir(parents=True)
            path.write_text(source)
            return inspect_file(path, root, {})[0]

    def test_fixture_paths_are_allowed_only_in_integration_tests(self):
        source = 'fn source() -> &\'static str { "../pa10/tests/spec/example.rs" }'
        for relative in ("dev/src/body/example.rs", "dev/bin/example.rs"):
            with self.subTest(path=relative):
                self.assertIn("compiler contains a fixture/reference dependency",
                              self.inspect(relative, source))
        self.assertEqual(self.inspect("dev/tests/example.rs", source), [])

    def test_body_dependencies_remain_directed(self):
        for dependency in ("resolve", "lex", "sema", "types"):
            with self.subTest(dependency=dependency):
                self.assertEqual(self.inspect("dev/src/body/example.rs",
                                              f"use crate::{dependency}::Example;"), [])
        for owner, dependency in (("body", "native"), ("body", "lower"), ("lowir", "body")):
            with self.subTest(owner=owner, dependency=dependency):
                findings = self.inspect(f"dev/src/{owner}/example.rs",
                                        f"use crate::{dependency}::Example;")
                self.assertTrue(any("forbidden phase dependency" in finding for finding in findings))

    def test_defining_proof_pipeline_preserves_phase_direction(self):
        for owner, dependencies in (("pipeline", ("sema", "body", "borrow")),
                                    ("lower", ("pipeline",))):
            for dependency in dependencies:
                with self.subTest(owner=owner, dependency=dependency):
                    self.assertEqual(self.inspect(f"dev/src/{owner}/example.rs",
                                                  f"use crate::{dependency}::Example;"), [])
        for owner, dependency in (("sema", "pipeline"), ("body", "pipeline"),
                                  ("borrow", "pipeline"), ("pipeline", "lower"),
                                  ("pipeline", "native")):
            with self.subTest(owner=owner, dependency=dependency):
                findings = self.inspect(f"dev/src/{owner}/example.rs",
                                        f"use crate::{dependency}::Example;")
                self.assertTrue(any("forbidden phase dependency" in finding for finding in findings))

    def test_pipeline_cannot_read_process_environment(self):
        findings = self.inspect("dev/src/pipeline/mod.rs",
                                'fn read() { let _ = std::env::var("RUSTGM_MODE"); }')
        self.assertTrue(any("environment" in finding for finding in findings))

    def test_ffi_is_accepted_only_in_the_bridge_boundary_file(self):
        boundary = "dev/backend-bridge/src/ffi.rs"
        declared = 'unsafe extern "C" {\n    fn cppgm_lowir_abi_version() -> u32;\n}\n'
        called = ("fn version() -> u32 {\n    // SAFETY: the function takes no argument.\n"
                  "    unsafe { cppgm_lowir_abi_version() }\n}\n")
        self.assertEqual(self.inspect(boundary, declared + called), [])
        exported = 'pub unsafe extern "C" fn version() -> u32 {\n    0\n}\n'
        for relative in ("dev/backend-bridge/src/lib.rs", "dev/backend-bridge/bin/inprocess.rs",
                         "dev/src/lowir/writer.rs", "dev/bin/emission.rs"):
            for source in (declared, called, exported):
                with self.subTest(path=relative, source=source):
                    self.assertTrue(any("unsafe/FFI implementation is prohibited" in finding
                                        for finding in self.inspect(relative, source)))

    def test_bridge_boundary_justifies_each_unsafe_block_and_has_no_unsafe_items(self):
        boundary = "dev/backend-bridge/src/ffi.rs"
        bare = "fn version() -> u32 {\n    // The library is linked.\n    unsafe { version_of() }\n}\n"
        self.assertTrue(any("lacks a // SAFETY: justification" in finding
                            for finding in self.inspect(boundary, bare)))
        for source in ("unsafe fn read() {}\n", "unsafe impl Send for Module {}\n",
                       "unsafe trait Raw {}\n", 'pub unsafe extern "C" fn read() {}\n'):
            with self.subTest(source=source):
                self.assertTrue(any("unsafe/FFI implementation is prohibited" in finding
                                    for finding in self.inspect(boundary, source)))
