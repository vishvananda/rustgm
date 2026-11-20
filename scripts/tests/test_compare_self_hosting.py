"""PA45 requires independent strict builds and equality of every artifact."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from compare_self_hosting import compare
from host_process import digest


class CompareSelfHosting(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.home = Path(temp.name).resolve()
        self.folders = [self.home / f'stage{stage}' / 'target' for stage in (1, 2)]
        self.links = [folder.parent / 'rustgm' for folder in self.folders]
        for folder, link in zip(self.folders, self.links, strict=True):
            folder.mkdir(parents=True)
            link.write_bytes(b'linked executable')
            self.build_receipt(folder)
            self.build_link_receipt(folder, link)

    def build_receipt(self, folder):
        steps, completed = [], []
        for unit, crate in enumerate(('core', 'rustgm')):
            directory = folder / f'unit-{unit}'
            directory.mkdir()
            (directory / 'compile.stderr').touch()
            outputs = {}
            for name in ('lowir', 'metadata'):
                artifact = directory / name
                artifact.write_bytes(f'{crate} {name}'.encode())
                outputs[name] = self.artifact(artifact)
            planned = [(name, row['path']) for name, row in outputs.items()]
            arguments = ['-o', outputs['lowir']['path'], '--metadata-output', outputs['metadata']['path']]
            if unit:
                arguments += ['--extern', 'core=' + completed[0]['outputs']['metadata']['path']]
            steps.append({
                'unit': unit, 'package': crate, 'crate_name': crate, 'disambiguator': crate,
                'library': unit == 0, 'dependencies': [0] if unit else [],
                'source': f'/frozen/{crate}/main.rs', 'environment': [['LANG', 'C.UTF-8']],
                'outputs': planned, 'arguments': arguments,
            })
            completed.append({'unit': unit, 'crate': crate, 'outputs': outputs})
        receipt = {
            'state': 'complete', 'profile': 'strict', 'bringup_records': 0,
            'failure': None, 'blocked': [], 'not_run': [], 'graph': {'root': 'rustgm'},
            'rustgm': {'sha256': digest(self.links[0])}, 'implementation_sha256': {'target_units.py': 'pin'},
            'steps': steps, 'completed': completed,
        }
        (folder / 'receipt.json').write_text(json.dumps(receipt))

    def build_link_receipt(self, folder, link):
        target = folder / 'receipt.json'
        receipt = {
            'state': 'complete', 'target_receipt': str(target), 'target_sha256': digest(target),
            'linked_compiler': self.artifact(link), 'units': [{'unit': 1, 'crate': 'rustgm'}],
            'tools': {'linker': {'sha256': 'frozen linker'}},
            'implementation_sha256': {'link_self_hosting.py': 'frozen stage linker'},
        }
        (link.parent / 'receipt.json').write_text(json.dumps(receipt))

    @staticmethod
    def artifact(path):
        return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest(path)}

    def receipt(self, index):
        path = self.folders[index] / 'receipt.json'
        return path, json.loads(path.read_bytes())

    def test_matches_independent_lowir_metadata_and_linked_binary(self):
        result = compare(*self.folders, *self.links)
        self.assertEqual(result['bringup_records'], 0)
        self.assertEqual(result['linked_compiler']['sha256'], digest(self.links[0]))
        self.assertEqual([unit['crate'] for unit in result['units']], ['core', 'rustgm'])
        self.assertTrue(all(set(unit['outputs']) == {'lowir', 'metadata'} for unit in result['units']))

    def test_perturbed_stage_one_stops_at_first_different_unit_artifact(self):
        path, receipt = self.receipt(0)
        changed = self.folders[0] / 'unit-0' / 'lowir'
        changed.write_bytes(b'changed artifact')
        receipt['completed'][0]['outputs']['lowir'] = self.artifact(changed)
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'core lowir differs between stages'):
            compare(*self.folders, *self.links)

    def test_a_bringup_record_in_actual_compiler_output_fails_the_gate(self):
        log = self.folders[1] / 'unit-0' / 'compile.stderr'
        log.write_text('bringup-unproven\tcore.rs\t4\titem\tobligation\n')
        with self.assertRaisesRegex(ValueError, 'core has a bring-up record'):
            compare(*self.folders, *self.links)

    def test_all_outputs_including_binary_crate_metadata_are_required(self):
        path, receipt = self.receipt(1)
        receipt['steps'][1]['outputs'].pop()
        del receipt['completed'][1]['outputs']['metadata']
        receipt['steps'][1]['arguments'][2:4] = []
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'rustgm requires all planned artifacts'):
            compare(*self.folders, *self.links)

    def test_stage_two_must_use_stage_one(self):
        path, receipt = self.receipt(1)
        receipt['rustgm']['sha256'] = '0' * 64
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'stage 2 did not use the linked stage 1 compiler'):
            compare(*self.folders, *self.links)

    def test_linking_must_reproduce_the_stage_one_compiler(self):
        self.links[1].write_bytes(b'changed executable')
        with self.assertRaisesRegex(ValueError, 'linked RustGM compiler differs between stages'):
            compare(*self.folders, *self.links)

    def test_equal_executable_with_stale_target_receipt_is_rejected(self):
        self.build_link_receipt(self.folders[0], self.links[1])
        with self.assertRaisesRegex(ValueError, 'requires its completed link receipt'):
            compare(*self.folders, *self.links)

    def test_equal_executable_without_its_link_receipt_is_rejected(self):
        copy = self.home / 'copied-compiler'
        copy.write_bytes(self.links[1].read_bytes())
        with self.assertRaisesRegex(FileNotFoundError, 'receipt.json'):
            compare(*self.folders, self.links[0], copy)

    def test_stages_require_the_same_native_link_tools(self):
        path = self.links[1].parent / 'receipt.json'
        receipt = json.loads(path.read_bytes())
        receipt['tools']['linker']['sha256'] = 'different linker'
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'different native-link tools'):
            compare(*self.folders, *self.links)


if __name__ == '__main__':
    unittest.main()
