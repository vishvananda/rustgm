"""The PA37 whole-graph procedure keeps frozen inputs and a strict stage chain."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import build_self_hosting as whole
from compare_self_hosting import compare
from host_process import digest
from link_self_hosting import stage_inputs
from stage1_run import artifact


class StrictSelfHosting(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.home = Path(temp.name).resolve()
        self.sources = self.home / 'inputs'
        self.sources.mkdir()
        for name in ('core.rs', 'dependency.rs', 'main.rs'):
            (self.sources / name).write_text('// original source\n')
        self.configuration = {
            'core': {'path': 'core.rs', 'disambiguator': 'frozen_core', 'cfg': ['checked']},
            'guests': [
                {'name': 'dependency', 'edition': '2018', 'path': 'dependency.rs'},
                {'name': 'app', 'edition': '2024', 'path': 'main.rs', 'cfg': []},
            ],
        }
        self.tools = {
            'cppgm': {'sha256': 'pinned cppgm'}, 'linker': {'sha256': 'pinned linker'},
            'backend_lock_sha256': 'pinned backend lock',
        }
        self.plan = self.home / 'graph.json'
        self.stage0 = self.home / 'stage0'
        self.stage0.write_bytes(b'maintainer compiler')
        with patch.object(whole, 'tool_pins', return_value=whole.link_pins(self.tools)):
            whole.freeze(self.configuration, self.sources, self.stage0, self.stage0, self.plan)
        self.stages = [self.home / f'stage{i}' / 'target' for i in (1, 2)]
        self.links = [stage.parent / 'native' / 'rustgm' for stage in self.stages]

    @staticmethod
    def measure(command, *, cwd, stdout, stderr, limit, environment, diagnostic=''):
        stdout.write_text('')
        stderr.write_text(diagnostic)
        Path(command[command.index('-o') + 1]).write_bytes(b'whole graph LowIR')
        links = Path(command[command.index('--native-links-output') + 1])
        links.write_text('{"schema":1,"target":"x86_64-linux","libraries":[]}')
        return {'argv': command, 'cwd': str(cwd), 'status': 0,
                'timed_out': False, 'signal': None}

    def build(self, index, compiler, diagnostic=''):
        def measure(*args, **kwargs):
            return self.measure(*args, **kwargs, diagnostic=diagnostic)

        with patch.object(whole, 'measure', side_effect=measure):
            return whole.build(self.plan, self.sources, compiler, digest(compiler),
                               self.stages[index], 20)

    def link(self, index, receipt):
        folder = self.stages[index]
        compiler = self.links[index]
        compiler.parent.mkdir()
        compiler.write_bytes(b'self-built compiler')
        target = folder / 'receipt.json'
        whole.save(compiler.parent / 'receipt.json', {
            'state': 'complete',
            'target_receipt': str(target), 'target_sha256': digest(target),
            'linked_compiler': artifact(compiler),
            'units': stage_inputs(folder, receipt, None),
            'tools': self.tools,
            'implementation_sha256': {'link_self_hosting.py': 'pinned stage linker'},
        })

    def stages_and_links(self):
        first = self.build(0, self.stage0)
        self.link(0, first)
        second = self.build(1, self.links[0])
        self.link(1, second)
        return first, second

    def test_same_graph_reproduces_outputs_and_retains_compiler_chain(self):
        first, _ = self.stages_and_links()
        argument = first['command']['argv']
        self.assertLess(argument.index('dependency@2018=dependency.rs'),
                        argument.index('app@2024=main.rs'))
        self.assertEqual(argument[argument.index('--source-guest-cfg') + 1], 'app')
        self.assertEqual(argument[-1], 'core.rs')
        result = compare(*self.stages, *self.links)
        self.assertEqual(result['bringup_records'], 0)
        self.assertEqual(result['linked_compiler']['sha256'], digest(self.links[0]))
        self.assertEqual(set(result['outputs']), {'lowir', 'native_links'})

    def test_link_optimization_must_match_even_for_equal_artifacts(self):
        self.stages_and_links()
        for level, binary in enumerate(self.links):
            path = binary.parent / 'receipt.json'
            receipt = json.loads(path.read_bytes())
            receipt['tools']['object_optimization'] = level
            whole.save(path, receipt)
        with self.assertRaisesRegex(ValueError, 'different native link tools'):
            compare(*self.stages, *self.links)

    def test_changed_source_byte_breaks_frozen_inventory(self):
        (self.sources / 'dependency.rs').write_text('// changed source\n')
        with self.assertRaisesRegex(ValueError, 'source inventory differs'):
            self.build(0, self.stage0)

    def test_zero_exit_with_a_waiver_fails_strict_stage(self):
        with self.assertRaisesRegex(ValueError, 'reports a bring-up record'):
            self.build(0, self.stage0, diagnostic='bringup-waived\tcontract\n')
        receipt = json.loads((self.stages[0] / 'receipt.json').read_bytes())
        self.assertEqual(receipt['state'], 'failed')

    def test_changed_lowir_cannot_reproduce_stage_one(self):
        first, _ = self.stages_and_links()
        lowir = Path(first['outputs']['lowir']['path'])
        lowir.write_bytes(b'whole graph LowIx')
        with self.assertRaisesRegex(ValueError, 'lowir differs between stages'):
            compare(*self.stages, *self.links)

    def test_stage_two_requires_the_linked_stage_one_compiler(self):
        first = self.build(0, self.stage0)
        self.link(0, first)
        second = self.build(1, self.stage0)
        self.link(1, second)
        with self.assertRaisesRegex(ValueError, 'stage 2 did not use'):
            compare(*self.stages, *self.links)


if __name__ == '__main__':
    unittest.main()
