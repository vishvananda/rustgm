"""Host/target identity and planner failures are independent of source acceptance."""
from copy import deepcopy
from dataclasses import replace
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import build_units
import cargo_query
from build_units import Execution, parse
from cargo_plan import Job, execute
from host_build_script import digest

TRIPLE = 'x86_64-unknown-linux-gnu'
PROFILE = dict(name='dev', opt_level='0', lto='false', codegen_backend=None,
               codegen_units=None, debuginfo=2, split_debuginfo=None,
               debug_assertions=True, overflow_checks=True, rpath=False,
               incremental=False, panic='unwind', strip={'deferred': 'None'})


def edge(index, alias):
    return dict(index=index, extern_crate_name=alias, public=False, noprelude=False, nounused=False)


def row(package, kind='lib', *, host=False, mode='build', features=(), dependencies=()):
    script = kind == 'custom-build'
    return dict(pkg_id=package, target=dict(kind=[kind], crate_types=['bin' if script else kind],
                name='build-script-build' if script else package,
                src_path=f'/{package}/build.rs' if script else f'/{package}/src/lib.rs',
                edition='2021' if package == 'shared' else '2024',
                doc=not script, doctest=not script, test=not script),
                profile=deepcopy(PROFILE), platform=None if host else TRIPLE, mode=mode,
                features=list(features), dependencies=list(dependencies))


def graph():
    return dict(version=1, roots=[0], units=[
        row('app', dependencies=[edge(1, 'build_script_build'), edge(4, 'renamed')]),
        row('app', 'custom-build', mode='run-custom-build', dependencies=[edge(2, 'build_script_build')]),
        row('app', 'custom-build', host=True, dependencies=[edge(3, 'renamed')]),
        row('shared', host=True, features=['host_side']),
        row('shared', features=['target_side'])])


def decode(value):
    return parse(json.dumps(value).encode(), TRIPLE)


class Units(unittest.TestCase):
    def test_host_features_and_target_context_are_distinct(self):
        plan = decode(graph())
        self.assertEqual(plan.order, (3, 2, 1, 4, 0))
        self.assertEqual(plan.units[3].pkg_id, plan.units[4].pkg_id)
        self.assertEqual(plan.units[3].features, ('host_side',))
        self.assertEqual(plan.units[4].features, ('target_side',))
        self.assertEqual(plan.units[1].execution, Execution.HOST_RUN)
        self.assertEqual(plan.units[1].platform, TRIPLE)
        self.assertEqual(plan.units[2].execution, Execution.HOST_COMPILE)
        self.assertEqual(plan.units[0].execution, Execution.TARGET_COMPILE)
        self.assertEqual(plan.units[4].target.edition, '2021')
        self.assertEqual(plan.units[0].dependencies[1].extern_crate_name, 'renamed')

    def test_complete_profile_and_dependency_flags_are_retained(self):
        data = graph()
        data['units'][0]['profile']['overflow_checks'] = False
        data['units'][0]['dependencies'][1].update(public=True, noprelude=True, nounused=True)
        plan = decode(data)
        self.assertFalse(json.loads(plan.units[0].profile_json)['overflow_checks'])
        self.assertEqual(plan.units[0].dependencies[1], build_units.Dependency(4, 'renamed', True, True, True))

    def test_inactive_rows_cycles_missing_and_duplicate_edges_fail(self):
        bad = []
        data = graph()
        data['units'].append(row('inactive'))
        bad.append(data)
        data = graph()
        data['units'][4]['dependencies'] = [edge(0, 'cycle')]
        bad.append(data)
        data = graph()
        data['units'][0]['dependencies'][0]['index'] = 99
        bad.append(data)
        data = graph()
        data['units'][0]['dependencies'].append(edge(4, 'renamed'))
        bad.append(data)
        for data in bad:
            with self.subTest(data=data), self.assertRaises(ValueError):
                decode(data)

    def test_cross_domain_libraries_and_wrong_host_script_fail(self):
        for owner, dependency in ((0, edge(3, 'host_leak')), (2, edge(4, 'target_leak'))):
            data = graph()
            data['units'][owner]['dependencies'].append(dependency)
            with self.subTest(owner=owner), self.assertRaisesRegex(ValueError, 'crosses host/target'):
                decode(data)
        data = graph()
        data['units'][1]['features'] = ['unpaired']
        with self.assertRaisesRegex(ValueError, 'exact host compiler'):
            decode(data)

    def test_explicit_host_procedural_macro_is_preserved_without_execution(self):
        data = graph()
        data['units'].append(row('derive', 'proc-macro', host=True))
        data['units'][0]['dependencies'].append(edge(5, 'derive'))
        plan = decode(data)
        self.assertEqual(plan.units[5].execution, Execution.HOST_COMPILE)
        self.assertLess(plan.order.index(5), plan.order.index(0))

    def test_unrecognized_protocols_and_empty_graphs_fail(self):
        variants = []
        for key, value in (('version', 2), ('version', True), ('roots', []), ('roots', [True]),
                           ('roots', [0, 0]), ('units', []), ('extra', 'unsupported')):
            data = graph()
            data[key] = value
            variants.append(data)
        for key, value in (('mode', 'check'), ('features', ['same', 'same']), ('is_std', True)):
            data = graph()
            data['units'][0][key] = value
            variants.append(data)
        for data in variants:
            with self.subTest(data=data), self.assertRaises(ValueError):
                decode(data)
        with self.assertRaisesRegex(ValueError, 'duplicate JSON'):
            parse(b'{"version":1,"version":1}', TRIPLE)

    def test_bounded_graph_walk_is_iterative(self):
        count = 1500
        units = [row(f'crate_{i}', dependencies=[] if i == count - 1 else [edge(i + 1, 'next')])
                 for i in range(count)]
        data = dict(version=1, roots=[0], units=units)
        self.assertEqual(decode(data).order, tuple(reversed(range(count))))
        for name, value in (('MAX_BYTES', 4), ('MAX_UNITS', 2), ('MAX_EDGES', 2)):
            with patch.object(build_units, name, value), self.assertRaisesRegex(ValueError, 'budget'):
                decode(data)

    def test_duplicate_unit_does_not_merge_different_feature_contexts(self):
        data = graph()
        data['units'][3] = deepcopy(data['units'][4])
        with self.assertRaisesRegex(ValueError, 'duplicate build unit'):
            decode(data)


class Planning(unittest.TestCase):
    def job(self, root):
        manifest, lockfile, source = (root / name for name in ('Cargo.toml', 'Cargo.lock', 'lib.rs'))
        for path in (manifest, lockfile, source):
            path.write_text('explicit frozen input')
        cargo, compiler = root / 'cargo', root / 'rustc'
        cargo.write_text('pinned cargo')
        compiler.write_text('pinned query compiler')
        return Job(cargo, digest(cargo), compiler, digest(compiler), manifest, lockfile,
                   tuple((path, digest(path)) for path in (manifest, lockfile, source)),
                   'app', TRIPLE), source

    @staticmethod
    def process(source, *, status=0):
        def run(command, *, cwd, env, stdout, stderr, timeout):
            data = dict(version=1, roots=[0], units=[row('app')])
            data['units'][0]['target']['src_path'] = str(source)
            stdout.write(json.dumps(data).encode())
            stderr.write(b'explicit planner diagnostics')
            return subprocess.CompletedProcess(command, status)
        return run

    def test_success_is_a_plan_not_a_compilation_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job, source = self.job(root)
            folder = root / 'output'
            with patch('cargo_plan.subprocess.run', side_effect=self.process(source)) as run:
                plan = execute(job, folder)
            self.assertEqual(len(plan.units), 1)
            self.assertEqual(run.call_count, 1)
            receipt = json.loads((folder / 'receipt.json').read_text())
            self.assertEqual(receipt['state'], 'planned')
            self.assertIn('--frozen', receipt['command'])
            self.assertIn('--unit-graph', receipt['command'])
            self.assertEqual(receipt['environment']['CARGO_HOME'], str(folder / 'cargo-home'))
            with self.assertRaisesRegex(ValueError, 'already exists'):
                execute(job, folder)

    def test_failed_process_and_missing_source_never_publish_plan(self):
        for status, missing in ((86, False), (0, True)):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                job, source = self.job(root)
                if missing:
                    job = replace(job, inputs=job.inputs[:2])
                with patch('cargo_plan.subprocess.run', side_effect=self.process(source, status=status)), \
                        self.assertRaises(ValueError):
                    execute(job, root / 'output')
                receipt = json.loads((root / 'output/receipt.json').read_text())
                self.assertEqual(receipt['state'], 'failed')
                self.assertNotIn('order', receipt)
                self.assertIn('graph.stderr_sha256', receipt)

    def test_input_mutation_during_planning_invalidates_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job, source = self.job(root)
            run = self.process(source)

            def changing(*args, **kwargs):
                result = run(*args, **kwargs)
                source.write_text('changed during planning')
                return result

            with patch('cargo_plan.subprocess.run', side_effect=changing), \
                    self.assertRaisesRegex(ValueError, 'pin'):
                execute(job, root / 'output')
            self.assertEqual(json.loads((root / 'output/receipt.json').read_text())['state'], 'failed')

    def test_changed_tool_and_ambient_config_fail_before_execution(self):
        for config in (False, True):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                job, _ = self.job(root)
                if config:
                    (root / '.cargo').mkdir()
                    (root / '.cargo/config.toml').write_text('[build]\nrustc="other"\n')
                else:
                    job.cargo.write_text('changed tool')
                with patch('cargo_plan.subprocess.run') as run, self.assertRaises(ValueError):
                    execute(job, root / 'output')
                run.assert_not_called()
                self.assertFalse((root / 'output').exists())


class Queries(unittest.TestCase):
    def test_only_observed_queries_are_allowed(self):
        self.assertFalse(cargo_query.validate(['-vV'], TRIPLE))
        for arguments in (['source.rs'], ['--print=cfg'], ['--emit=metadata', '-'],
                          ['--version', '--sysroot', '/unrecorded'], ['-vV', '-o', 'output']):
            with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                cargo_query.validate(arguments, TRIPLE)

    def test_empty_query_input_is_required_even_with_print_flags(self):
        arguments = ['-', '--crate-name', '___', '--print=file-names', '--target', TRIPLE]
        for kind in ('bin', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'):
            arguments += ['--crate-type', kind]
        arguments += ['--print=sysroot', '--print=split-debuginfo', '--print=crate-name',
                      '--print=cfg', '-Wwarnings']
        self.assertTrue(cargo_query.validate(arguments, TRIPLE))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            compiler = root / 'rustc'
            compiler.write_text('query compiler')
            environment = {'RUSTGM_PLAN_QUERY_COMPILER': str(compiler),
                           'RUSTGM_PLAN_QUERY_SHA256': digest(compiler), 'RUSTGM_PLAN_TARGET': TRIPLE,
                           'RUSTGM_PLAN_QUERY_LOG': str(root / 'queries')}
            stdin = io.TextIOWrapper(io.BytesIO(b'fn main() {}'))
            with patch.dict('os.environ', environment, clear=True), patch.object(sys, 'stdin', stdin), \
                    patch.object(sys, 'argv', ['query', *arguments]), \
                    patch('cargo_query.subprocess.run') as execute_query, \
                    self.assertRaisesRegex(ValueError, 'source input'):
                cargo_query.main()
            execute_query.assert_not_called()
            self.assertEqual(json.loads((root / 'queries').read_text())['state'], 'rejected')


if __name__ == '__main__':
    unittest.main()
