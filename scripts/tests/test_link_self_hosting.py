"""The native stage link preserves the recorded closure of the strict binary."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from link_self_hosting import closure


def target(unit, library, dependencies, arguments):
    return {'unit': unit, 'crate_name': f'unit{unit}', 'library': library,
            'dependencies': dependencies, 'arguments': arguments,
            'outputs': [('metadata', f'unit{unit}.rgm')]}


class LinkSelfHosting(unittest.TestCase):
    def setUp(self):
        self.steps = [
            target(1, True, [], []),
            target(0, True, [1], ['--extern', 'base=unit1.rgm']),
            target(2, True, [], []),
            target(3, False, [1, 0], ['--extern', 'dep=unit0.rgm',
                                      '--dependency-metadata', 'unit1.rgm']),
        ]
        self.receipt = {'steps': self.steps, 'completed': [{} for _ in self.steps]}

    def test_binary_first_then_its_transitive_libraries_in_target_order(self):
        self.assertEqual([step['unit'] for step, _ in closure(self.receipt, 3)], [3, 1, 0])

    def test_swapped_dependencies_are_rejected(self):
        self.steps[-1]['dependencies'].reverse()
        with self.assertRaisesRegex(ValueError, 'frozen target order'):
            closure(self.receipt, 3)

    def test_missing_transitive_library_is_rejected(self):
        self.steps[-1]['dependencies'].remove(1)
        with self.assertRaisesRegex(ValueError, 'ordered transitive libraries'):
            closure(self.receipt, 3)

    def test_extra_compiler_dependency_is_rejected(self):
        self.steps[-1]['arguments'].extend(['--dependency-metadata', 'unit2.rgm'])
        with self.assertRaisesRegex(ValueError, 'closure differs from compiler inputs'):
            closure(self.receipt, 3)

    def test_library_cannot_be_selected_as_the_binary(self):
        with self.assertRaisesRegex(ValueError, 'selected binary'):
            closure(self.receipt, 0)


if __name__ == '__main__':
    unittest.main()
