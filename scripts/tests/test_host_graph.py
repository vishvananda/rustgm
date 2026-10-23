"""Host DAG sharing, context isolation, bounded receipts and failed publication."""
from collections import Counter
from copy import deepcopy
from dataclasses import replace
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from build_environment import Context
import host_graph
from host_graph_fixtures import packages
from host_process import digest
from package_snapshot import Snapshot
import test_build_units as units


def graph(snapshots):
    """The independently observed Cargo graph, with explicit renamed edges."""
    app, helper, native = (p.cargo_id for p in snapshots)
    edge, row = units.edge, units.row
    rows = [
        row(app, dependencies=[edge(1, 'build_script_build'), edge(4, 'renamed'), edge(10, 'native_sys')]),
        row(app, 'custom-build', mode='run-custom-build',
            dependencies=[edge(2, 'build_script_build'), edge(12, 'build_script_build')]),
        row(app, 'custom-build', host=True, dependencies=[edge(3, 'renamed')]),
        row(helper, host=True, features=['host_side'], dependencies=[edge(5, 'build_script_build'), edge(9, 'native_sys')]),
        row(helper, features=['target_side'], dependencies=[edge(6, 'build_script_build'), edge(10, 'native_sys')]),
        row(helper, 'custom-build', host=True, mode='run-custom-build', features=['host_side'],
            dependencies=[edge(7, 'build_script_build'), edge(11, 'build_script_build')]),
        row(helper, 'custom-build', mode='run-custom-build', features=['target_side'],
            dependencies=[edge(8, 'build_script_build'), edge(12, 'build_script_build')]),
        row(helper, 'custom-build', host=True, features=['host_side']),
        row(helper, 'custom-build', host=True, features=['target_side']),
        row(native, host=True, dependencies=[edge(11, 'build_script_build')]),
        row(native, dependencies=[edge(12, 'build_script_build')]),
        row(native, 'custom-build', host=True, mode='run-custom-build', dependencies=[edge(13, 'build_script_build')]),
        row(native, 'custom-build', mode='run-custom-build', dependencies=[edge(13, 'build_script_build')]),
        row(native, 'custom-build', host=True),
    ]
    by_id = {p.cargo_id: p for p in snapshots}
    for unit in rows:
        package = by_id[unit['pkg_id']]
        script = unit['target']['kind'] == ['custom-build']
        unit['target'].update(name='build-script-build' if script else package.name.replace('-', '_'),
                              src_path=str(package.root / ('build.rs' if script else 'src/lib.rs')),
                              edition='2024' if package.name == 'app' else '2021')
    return dict(version=1, roots=[0], units=rows)


def setup(root):
    snapshots, _ = packages(root)
    compiler, facts = root / 'compiler', root / 'facts'
    compiler.write_text('pinned compiler')
    facts.write_text('explicit host unwind and target abort configuration')
    context = Context(units.TRIPLE, units.TRIPLE, compiler, digest(compiler),
                      (('panic', ('unwind',)),), ((facts, digest(facts)),))
    contexts = {None: context, units.TRIPLE: replace(context, cfg=(('panic', ('abort',)),))}
    return snapshots, contexts, graph(snapshots)


def process(command, *, cwd, env, stdout, stderr, timeout):
    """Only process effects are substituted; graph validation and hashing are real."""
    if '--crate-name' in command:
        Path(command[command.index('-o') + 1]).write_text('host artifact ' + str(cwd))
    else:
        name = env['CARGO_PKG_NAME']
        if name == 'native-sys':
            stdout.write(b'cargo:root-dir=old\ncargo::metadata=root-dir=headers=ready\ncargo::metadata=empty=\n')
        elif name == 'helper':
            stdout.write(b'cargo::rustc-check-cfg=cfg(ready)\ncargo::rustc-cfg=ready\n'
                         b'cargo::rustc-env=LOCAL_VALUE=checked\ncargo::metadata=hidden=not-transitive\n')
        else:
            stdout.write(b'cargo::rustc-env=APP_VALUE=17\n')
        (Path(env['OUT_DIR']) / 'value.rs').write_text(name)
    return subprocess.CompletedProcess(command, 0)


class Graph(unittest.TestCase):
    def test_host_nodes_once_with_exact_dependency_and_target_context(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            snapshots, contexts, data = setup(root)
            plan = units.decode(data)
            with patch('host_process.subprocess.run', side_effect=process) as run:
                results = host_graph.execute(plan, snapshots, contexts, root / 'output')
            self.assertEqual(len(results), 5)
            self.assertEqual(run.call_count, 11)
            receipt = json.loads((root / 'output/receipt.json').read_text())
            calls = receipt['commands']
            compilations = [r for r in calls if r['name'] == 'compile']
            executions = [r for r in calls if r['name'] == 'execute']
            self.assertEqual(len(compilations), 6)
            self.assertEqual(len(executions), 5)
            self.assertTrue(all('--target' not in r['command'] for r in calls))
            native = [r for r in executions if r['environment']['CARGO_PKG_NAME'] == 'native-sys']
            self.assertEqual(native[0]['command'], native[1]['command'])
            self.assertEqual({r['environment']['CARGO_CFG_PANIC'] for r in native}, {'abort', 'unwind'})
            helpers = [r for r in executions if r['environment']['CARGO_PKG_NAME'] == 'helper']
            self.assertEqual({r['environment']['CARGO_CFG_FEATURE'] for r in helpers}, {'host_side', 'target_side'})
            app = next(r for r in executions if r['environment']['CARGO_PKG_NAME'] == 'app')
            self.assertEqual(app['environment']['DEP_RUSTGM_NATIVE_ROOT_DIR'], 'headers=ready')
            self.assertEqual(app['environment']['DEP_RUSTGM_NATIVE_EMPTY'], '')
            self.assertNotIn('DEP_RUSTGM_NATIVE_HIDDEN', app['environment'])
            helper = next(r for r in compilations if '--crate-type' in r['command']
                          and 'rlib' in r['command'] and r['environment']['CARGO_PKG_NAME'] == 'helper')
            self.assertIn('ready', helper['command'])
            self.assertEqual(helper['environment']['LOCAL_VALUE'], 'checked')
            self.assertNotIn('DEP_RUSTGM_NATIVE_ROOT_DIR', helper['environment'])
            app_compile = next(r for r in compilations if r['environment']['CARGO_PKG_NAME'] == 'app')
            self.assertTrue(any(s.startswith('renamed=') for s in app_compile['command']))
            self.assertNotIn('LOCAL_VALUE', app_compile['environment'])
            self.assertNotIn('OUT_DIR', app_compile['environment'])
            self.assertEqual(receipt['state'], 'complete')

    def test_shared_packages_are_verified_twice_and_receipt_writes_stay_linear(self):
        for width in (8, 32, 128):
            with self.subTest(width=width), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                snapshots, contexts, data = setup(root)
                # Many independent consumers share one already-built host library.
                for index in range(width):
                    clone = deepcopy(data['units'][2])
                    clone['target']['name'] = f'consumer_{index}'
                    clone['target'].update(kind=['lib'], crate_types=['rlib'])
                    clone['dependencies'] = [units.edge(3, 'renamed')]
                    data['units'][2]['dependencies'].append(units.edge(len(data['units']), f'consumer_{index}'))
                    data['units'].append(clone)
                plan = units.decode(data)
                visits, writes = Counter(), Counter()
                verify, write = Snapshot.verify, Path.write_text

                def counted(snapshot):
                    visits[snapshot.cargo_id] += 1
                    return verify(snapshot)

                def record(path, text, *args, **kwargs):
                    writes[path.name] += 1
                    return write(path, text, *args, **kwargs)

                with patch.object(Snapshot, 'verify', counted), patch.object(Path, 'write_text', record), \
                        patch('host_process.subprocess.run', side_effect=process) as run:
                    host_graph.execute(plan, snapshots, contexts, root / 'output')
                self.assertEqual(set(visits.values()), {2})
                self.assertEqual(run.call_count, 11 + width)
                self.assertEqual(writes['receipt.json'], 3)
                self.assertEqual(writes['commands.json'], 3 * (11 + width))
                self.assertEqual(len((root / 'output/progress.jsonl').read_text().splitlines()), 11 + width)

    def test_preflight_rejects_missing_packages_features_modifiers_and_profiles(self):
        for variant in ('package', 'feature', 'modifier', 'profile', 'context', 'source', 'proc-macro', 'order'):
            with self.subTest(variant=variant), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                snapshots, contexts, data = setup(root)
                if variant == 'package':
                    snapshots = snapshots[:-1]
                elif variant == 'feature':
                    data['units'][3]['features'] = ['absent']
                elif variant == 'modifier':
                    data['units'][2]['dependencies'][0]['public'] = True
                elif variant == 'profile':
                    data['units'][2]['profile']['incremental'] = True
                elif variant == 'context':
                    contexts.pop(None)
                elif variant == 'source':
                    data['units'][3]['target']['src_path'] = str(root / 'unowned.rs')
                elif variant == 'proc-macro':
                    data['units'][3]['target'].update(kind=['proc-macro'], crate_types=['proc-macro'])
                plan = units.decode(data)
                if variant == 'order':
                    plan = replace(plan, order=tuple(reversed(plan.order)))
                with patch('host_process.subprocess.run') as run, self.assertRaises(ValueError):
                    host_graph.execute(plan, snapshots, contexts, root / 'output')
                run.assert_not_called()
                self.assertFalse((root / 'output').exists())

    def test_graph_budget_failure_never_publishes_success(self):
        for limit in ('MAX_GENERATED_FILES', 'MAX_GENERATED_BYTES', 'MAX_ARTIFACT_BYTES', 'MAX_SCRIPT_OUTPUT_BYTES'):
            with self.subTest(limit=limit), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                snapshots, contexts, data = setup(root)
                with patch.object(host_graph, limit, 0), patch('host_process.subprocess.run', side_effect=process), \
                        self.assertRaisesRegex(ValueError, 'budget'):
                    host_graph.execute(units.decode(data), snapshots, contexts, root / 'output')
                receipt = json.loads((root / 'output/receipt.json').read_text())
                self.assertEqual(receipt['state'], 'failed')

    def test_process_failure_and_caught_query_are_not_success(self):
        for variant in ('compile', 'execute', 'timeout', 'query', 'query-shape', 'metadata'):
            with self.subTest(variant=variant), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                snapshots, contexts, data = setup(root)

                def failing(command, **kwargs):
                    if variant == 'timeout':
                        raise subprocess.TimeoutExpired(command, 120)
                    result = process(command, **kwargs)
                    compile = '--crate-name' in command
                    if variant == 'compile' and compile or variant == 'execute' and not compile:
                        return subprocess.CompletedProcess(command, 86)
                    if variant == 'query' and not compile:
                        Path(kwargs['env']['RUSTGM_PLAN_QUERY_LOG']).write_text('{"state":"rejected","status":1}\n')
                    if variant == 'query-shape' and not compile:
                        Path(kwargs['env']['RUSTGM_PLAN_QUERY_LOG']).write_text('[]\n')
                    if variant == 'metadata' and not compile:
                        kwargs['stdout'].write(b'cargo::metadata=root_dir=ambiguous\n')
                    return result

                with patch('host_process.subprocess.run', side_effect=failing), \
                        self.assertRaises((ValueError, subprocess.TimeoutExpired)):
                    host_graph.execute(units.decode(data), snapshots, contexts, root / 'output')
                receipt = json.loads((root / 'output/receipt.json').read_text())
                self.assertEqual(receipt['state'], 'failed')

    def test_changed_source_artifact_or_generated_input_invalidates_whole_graph(self):
        for variant in ('source', 'artifact', 'generated', 'inventory'):
            with self.subTest(variant=variant), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                snapshots, contexts, data = setup(root)
                finish = host_graph.Build.finish

                def changing(build):
                    if variant == 'source':
                        (snapshots[0].root / 'build.rs').write_text('changed')
                    elif variant == 'artifact':
                        next(iter(build.artifacts.values())).path.write_text('changed')
                    elif variant == 'generated':
                        (next(iter(build.generated.values())).folder / 'value.rs').write_text('changed')
                    else:
                        (build.library_folder / 'unowned.rlib').write_text('changed')
                    return finish(build)

                with patch.object(host_graph.Build, 'finish', changing), \
                        patch('host_process.subprocess.run', side_effect=process), self.assertRaises(ValueError):
                    host_graph.execute(units.decode(data), snapshots, contexts, root / 'output')
                self.assertEqual(json.loads((root / 'output/receipt.json').read_text())['state'], 'failed')


if __name__ == '__main__':
    unittest.main()
