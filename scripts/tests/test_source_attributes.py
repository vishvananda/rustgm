from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_source_attributes import Gate
from crate_qualification import Check


class SourceAttributeHarness(unittest.TestCase):
    def test_declared_warning_bytes_are_required_and_new_diagnostics_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            gate = Gate(Path(sys.executable), Path(temp) / 'evidence', 30)
            warned = subprocess.CompletedProcess([], 0, b'lowir', b'warning[unused_must_use]: reason\n')
            with patch('crate_qualification.subprocess.run', return_value=warned):
                self.assertEqual(gate.run('warned', ['tool'], stderr=warned.stderr).stdout, b'lowir')
                with self.assertRaisesRegex(ValueError, 'undeclared diagnostic'):
                    gate.run('undeclared', ['tool'])
                with self.assertRaisesRegex(ValueError, 'undeclared diagnostic'):
                    gate.run('changed', ['tool'], stderr=b'warning[unused_must_use]: other\n')
            failed = subprocess.CompletedProcess([], 1, b'lowir', warned.stderr)
            with patch('crate_qualification.subprocess.run', return_value=failed):
                with self.assertRaisesRegex(ValueError, 'expected 0, got 1'):
                    gate.run('error-masquerade', ['tool'], stderr=warned.stderr)
            empty = subprocess.CompletedProcess([], 0, b'lowir', b'')
            with patch('crate_qualification.subprocess.run', return_value=empty):
                self.assertEqual(gate.run('plain', ['tool']).stdout, b'lowir')

    def test_shared_command_boundary_retains_overrides_without_inherited_environment(self):
        with tempfile.TemporaryDirectory() as temp:
            check = Check(Path(sys.executable), Path(temp) / 'commands')
            result = subprocess.CompletedProcess([], 0, b'', b'')
            with patch('crate_qualification.subprocess.run', return_value=result) as run:
                check.run('source', ['tool'], timeout=7, environment={'RUSTC_BOOTSTRAP': '1'})
            self.assertEqual(run.call_args.kwargs['timeout'], 7)
            self.assertEqual(run.call_args.kwargs['env']['RUSTC_BOOTSTRAP'], '1')
            self.assertEqual(check.rows[0]['environment'], {'RUSTC_BOOTSTRAP': '1'})
            self.assertEqual(check.rows[0]['timeout'], 7)

    def test_expired_global_budget_stops_before_another_child(self):
        with tempfile.TemporaryDirectory() as temp:
            gate = Gate(Path(sys.executable), Path(temp) / 'evidence', 30)
            gate.deadline = 0
            with patch.object(gate.check, 'run') as run:
                with self.assertRaises(TimeoutError):
                    gate.run('late', [])
                run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
