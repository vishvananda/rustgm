"""Frozen target units plan exact RustGM invocations or fail with typed gaps."""
from copy import deepcopy
from dataclasses import replace
import hashlib
import json
from pathlib import Path
import stat
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from build_script_output import parse as parse_script
from build_units import parse
import stdlib_plan
import stdlib_targets
from target_units import TargetPackage, execute, plan_units

ROOT = Path(__file__).resolve().parents[2]
TRIPLE = 'x86_64-unknown-linux-gnu'
OUT = Path('/out')
# RustGM's fixed course profile, as Cargo would resolve a matching custom profile.
PROFILE = dict(name='release', opt_level='0', lto='false', codegen_backend=None, codegen_units=None,
               debuginfo=0, split_debuginfo=None, debug_assertions=False, overflow_checks=True,
               rpath=False, incremental=False, panic='abort', strip={'deferred': 'None'})


def frozen(root, name='course-profile'):
    outcomes, graphs = stdlib_targets.load(name)
    plan, _ = graphs[root]
    packages = stdlib_targets.bind(outcomes, plan, Path('/frozen/rust'), Path('/frozen/vendor'))
    return plan, packages


def plan_frozen(root, scripts=None, name='course-profile', **options):
    plan, packages = frozen(root, name)
    return {step.crate_name: step for step in plan_units(
        plan, packages, OUT / root, source_core=stdlib_targets.source_core(plan), scripts=scripts, **options)}


def builtins_script():
    template = stdlib_targets.TEMPLATE.read_bytes()
    data = template.replace(b'{PACKAGE_ROOT}', b'/frozen/rust/library/compiler-builtins/compiler-builtins')
    return SimpleNamespace(output=parse_script(data, 0), folder=Path('/out/script/generated'), key='frozen'), data


def edge(index, alias, **flags):
    return dict(index=index, extern_crate_name=alias, public=False, noprelude=False, nounused=False) | flags


def row(name, kind='lib', edition='2024', dependencies=(), profile=None):
    return dict(pkg_id=f'path+file:///src/{name}#1.0.0',
                target=dict(kind=[kind], crate_types=[kind], name=name, src_path=f'/src/{name}/src/lib.rs',
                            edition=edition, doc=True, doctest=True, test=True),
                profile=deepcopy(profile or PROFILE), platform=TRIPLE, mode='build', features=[],
                dependencies=list(dependencies))


def authored(units, roots):
    plan = parse(json.dumps(dict(version=1, units=units, roots=roots)).encode(), TRIPLE)
    packages = {unit.pkg_id: TargetPackage(f'local {unit.target.name}', 'k' + unit.target.name,
                                           Path(f'/src/{unit.target.name}'), Path(f'/src/{unit.target.name}'),
                                           (('CARGO_PKG_NAME', unit.target.name),), False)
                for unit in plan.units}
    return plan, packages


class FrozenPlans(unittest.TestCase):
    def test_both_graph_sets_authenticate_and_place_core_first(self):
        for name in stdlib_targets.SETS:
            _, graphs = stdlib_targets.load(name)
            self.assertEqual({root: len(plan.units) for root, (plan, _) in graphs.items()},
                             {'core': 1, 'alloc': 5, 'std': 19})
            for root, (plan, _) in graphs.items():
                steps = plan_units(plan, frozen(root, name)[1], OUT / root,
                                   source_core=stdlib_targets.source_core(plan))
                self.assertEqual(steps[0].crate_name, 'core', (name, root))
        with tempfile.TemporaryDirectory() as temp:
            copy = Path(temp) / 'course-profile'
            copy.mkdir()
            for root in ('core', 'alloc', 'std'):
                (copy / f'{root}.json').write_bytes((stdlib_targets.GRAPHS / f'course-profile/{root}.json').read_bytes())
            (copy / 'core.json').write_bytes((copy / 'core.json').read_bytes() + b' ')
            with self.assertRaisesRegex(ValueError, 'observed hash'):
                stdlib_targets.load(graphs=Path(temp))

    def test_course_profile_graphs_differ_from_history_only_in_their_declared_profile(self):
        observed = json.loads(stdlib_targets.SETS['course-profile'].read_text())
        self.assertEqual(observed['base_outcomes']['sha256'],
                         hashlib.sha256(stdlib_targets.SETS['dev-profile'].read_bytes()).hexdigest())
        self.assertEqual(observed['configuration']['settings'], dict(stdlib_plan.SETTINGS))
        pins, graphs = stdlib_targets.load()
        _, history = stdlib_targets.load('dev-profile')
        for root, (plan, _) in graphs.items():
            stdlib_plan.check(plan, history[root][0], pins, Path('/frozen/rust'), Path('/frozen/vendor'))
        changed = replace(graphs['core'][0].units[0], profile_json=history['core'][0].units[0].profile_json)
        with self.assertRaisesRegex(ValueError, 'course profile'):
            stdlib_plan.check(replace(graphs['core'][0], units=(changed,)), history['core'][0], pins,
                              Path('/frozen/rust'), Path('/frozen/vendor'))

    def test_core_is_an_exact_source_core_invocation_without_profile_gaps(self):
        core = plan_frozen('core')['core']
        name = f'core@{core.disambiguator}'
        self.assertRegex(core.disambiguator, r'^h[0-9a-f]{20}$')
        self.assertEqual(core.arguments, (
            '--emit-lowir', '--edition=2024', '--crate-type=lib', '--crate-name', 'core',
            '--crate-disambiguator', core.disambiguator, '--ub-checks=no', '--source-core', name,
            '--metadata-output', '/out/core/unit-0/core.rgcrate', '-o', '/out/core/unit-0/core.lowir',
            '/frozen/rust/library/core/src/lib.rs'))
        # Only the unreadable test root remains: its manifest environment.
        self.assertEqual([(gap.field, gap.value) for gap in core.gaps], [('package.environment', core.package)])
        self.assertEqual((core.lint_cap, core.timeout_seconds), (None, 120))
        outcomes, graphs = stdlib_targets.load()
        plan, _ = graphs['core']
        moved = stdlib_targets.bind(outcomes, plan, Path('/elsewhere'), Path('/vendor'))
        self.assertEqual(plan_units(plan, moved, OUT / 'x')[0].disambiguator, core.disambiguator)
        self.assertEqual(plan_frozen('std')['core'].disambiguator, core.disambiguator)

    def test_historical_dev_profile_reports_the_mismatched_profile(self):
        core = plan_frozen('core', name='dev-profile')['core']
        self.assertEqual([(gap.field, gap.value) for gap in core.gaps], [
            ('profile.debuginfo', '2'), ('profile.debug_assertions', 'true'),
            ('package.environment', core.package)])
        self.assertNotEqual(plan_frozen('core')['core'].disambiguator, core.disambiguator)

    def test_every_std_unit_reports_only_settings_rustgm_cannot_express(self):
        steps = plan_frozen('std')
        expected = {name: {'package.environment'} for name in steps}
        # Amended for the rlib-only std decision: std's dylib is dropped, not a gap.
        expected['std'].add('build-script.output')
        expected['compiler_builtins'].add('build-script.output')
        expected['libc'].add('build-script.output')
        self.assertEqual({name: {gap.field for gap in step.gaps} for name, step in steps.items()}, expected)
        self.assertEqual(list(steps), ['core', 'compiler_builtins', 'alloc', 'rustc_std_workspace_core',
                                       'cfg_if', 'rustc_std_workspace_alloc', 'hashbrown', 'libc',
                                       'panic_abort', 'rustc_demangle', 'std_detect', 'unwind', 'std'])
        plan, _ = stdlib_targets.load()[1]['std']
        std = next(unit for unit in plan.units if unit.target.name == 'std')
        self.assertEqual(std.target.crate_types, ('dylib', 'rlib'))
        self.assertEqual({name: step.dropped_crate_types for name, step in steps.items() if step.dropped_crate_types},
                         {'std': ('dylib',)})
        self.assertIn('--crate-type=lib', steps['std'].arguments)
        metadata = steps['std'].arguments[steps['std'].arguments.index('--metadata-output') + 1]
        self.assertTrue(metadata.endswith('/std.rgcrate'), metadata)
        # panic_abort's -Cpanic=abort only restates the fixed abort target.
        self.assertNotIn('rustflags', {gap.field for gap in steps['panic_abort'].gaps})
        self.assertEqual({name for name, step in steps.items() if step.lint_cap == 'allow'},
                         {'cfg_if', 'hashbrown', 'libc', 'rustc_demangle'})

    def test_dependents_use_source_client_aliases_and_transitive_inventory(self):
        steps = plan_frozen('std')
        core, builtins = steps['core'], steps['compiler_builtins']
        workspace = steps['rustc_std_workspace_core']
        self.assertEqual(steps['cfg_if'].arguments, (
            '--emit-lowir', '--edition=2018', '--crate-type=lib', '--crate-name', 'cfg_if',
            '--crate-disambiguator', steps['cfg_if'].disambiguator, '--ub-checks=no',
            '--source-client', f'core@{core.disambiguator}', '--cap-lints=allow',
            '--cfg', 'feature="core"', '--cfg', 'feature="rustc-dep-of-std"',
            '--extern', f'core={dict(workspace.outputs)["metadata"]}',
            '--dependency-metadata', dict(core.outputs)['metadata'],
            '--dependency-metadata', dict(builtins.outputs)['metadata'],
            '--metadata-output', '/out/std/unit-1/cfg_if.rgcrate', '-o', '/out/std/unit-1/cfg_if.lowir',
            '/frozen/vendor/cfg-if-1.0.4/src/lib.rs'))
        alloc = plan_frozen('alloc')['alloc']
        self.assertIn('public dependency core', alloc.lint_inputs)
        self.assertEqual(alloc.arguments[alloc.arguments.index('--source-client') + 1],
                         f'core@{core.disambiguator}')
        self.assertEqual([alloc.arguments[i + 1] for i, part in enumerate(alloc.arguments) if part == '--extern'],
                         ['compiler_builtins=/out/alloc/unit-1/compiler_builtins.rgcrate',
                          'core=/out/alloc/unit-4/core.rgcrate'])

    def test_compiler_builtins_consumes_its_frozen_script_output(self):
        script, data = builtins_script()
        step = plan_frozen('std', {3: script})['compiler_builtins']
        self.assertNotIn('build-script.output', {gap.field for gap in step.gaps})
        cfgs = [step.arguments[i + 1] for i, part in enumerate(step.arguments) if part == '--cfg']
        self.assertEqual(cfgs, ['feature="arch"', 'feature="compiler-builtins"', 'feature="default"',
                                'feature="unmangled-names"', 'intrinsics_enabled', 'mem_unaligned'])
        self.assertEqual(dict(step.environment)['OUT_DIR'], '/out/script/generated')
        declared = data.count(b'rustc-check-cfg=')
        self.assertEqual(declared, 135)
        # Every script expectation plus Cargo's two implicit feature/docsrs rows.
        self.assertEqual(sum(row.startswith('check-cfg cfg(') for row in step.lint_inputs), declared + 2)
        self.assertIn('metadata compiler-rt /frozen/rust/library/compiler-builtins/compiler-builtins/compiler-rt',
                      step.recorded_directives)
        unscripted = plan_frozen('std')['compiler_builtins']
        self.assertNotEqual(unscripted.disambiguator, step.disambiguator)


class Combined(unittest.TestCase):
    """One run relocates the graph, executes host scripts and consumes their output."""

    def test_relocation_moves_package_ids_and_sources_together(self):
        pins, graphs = stdlib_targets.load()
        plan, _ = graphs['std']
        packages = stdlib_targets.bind(pins, plan, Path('/now/rust'), Path('/now/vendor'))
        moved, rebound = stdlib_targets.relocate(plan, packages)
        core = next(unit for unit in moved.units if unit.target.name == 'core')
        self.assertEqual(core.pkg_id, 'path+file:///now/rust/library/core#0.0.0')
        self.assertEqual(core.target.src_path, '/now/rust/library/core/src/lib.rs')
        libc = next(unit for unit in moved.units if unit.target.name == 'libc')
        self.assertEqual(libc.target.src_path, '/now/vendor/libc-0.2.185/src/lib.rs')
        self.assertEqual(set(rebound), {unit.pkg_id for unit in moved.units})
        self.assertTrue(all(package.planned_root == package.root for package in rebound.values()))
        before = plan_units(plan, packages, OUT, source_core=stdlib_targets.source_core(plan))
        after = plan_units(moved, rebound, OUT, source_core=stdlib_targets.source_core(moved))
        self.assertEqual([(s.crate_name, s.disambiguator, s.arguments) for s in before],
                         [(s.crate_name, s.disambiguator, s.arguments) for s in after])
        unit = replace(plan.units[0], target=replace(plan.units[0].target, src_path='/elsewhere/lib.rs'))
        with self.assertRaisesRegex(ValueError, 'outside its bound package'):
            stdlib_targets.relocate(replace(plan, units=(unit, *plan.units[1:])), packages)

    def test_host_outputs_are_checked_against_history_and_consumed(self):
        pins, graphs = stdlib_targets.load()
        plan = graphs['std'][0]
        plan, packages = stdlib_targets.relocate(plan, stdlib_targets.bind(
            pins, plan, Path('/frozen/rust'), Path('/frozen/vendor')))
        script, data = builtins_script()
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp) / 'unit-3/generated'
            folder.mkdir(parents=True)
            (folder.parent / 'execute.stdout').write_bytes(data)
            outputs = {3: replace_namespace(script, folder=folder),
                       8: SimpleNamespace(output=SimpleNamespace(directives=(None,) * 27), folder=folder, key='l'),
                       15: SimpleNamespace(output=SimpleNamespace(directives=(None,) * 8), folder=folder, key='s')}
            stdlib_targets.check_scripts(plan, outputs, pins, packages)
            (folder.parent / 'execute.stdout').write_bytes(data + b'cargo:rustc-cfg=extra\n')
            with self.assertRaisesRegex(ValueError, 'frozen template'):
                stdlib_targets.check_scripts(plan, outputs, pins, packages)
            outputs[8] = SimpleNamespace(output=SimpleNamespace(directives=()), folder=folder, key='l')
            with self.assertRaisesRegex(ValueError, 'libc'):
                stdlib_targets.check_scripts(plan, {8: outputs[8]}, pins, packages)
        inputs = [SimpleNamespace(cargo_id=unit.pkg_id) for unit in plan.units]
        calls = []

        def host(selected_plan, selected, contexts, folder, lint_caps):
            calls.append(({s.cargo_id for s in selected}, lint_caps, contexts))
            return {3: script}

        with patch.object(stdlib_targets, 'contexts', return_value='contexts'), \
                patch.object(stdlib_targets.host_graph, 'execute', side_effect=host), \
                patch.object(stdlib_targets, 'digest', return_value='receipt'):
            generated, receipt = stdlib_targets.host_scripts(plan, packages, inputs,
                                                             {'rustc': (Path('/rustc'), 'sha')}, Path('/out'))
        owners = {unit.pkg_id for unit in plan.units if unit.target.kind == ('custom-build',)}
        self.assertEqual(calls, [(owners, {next(p for p in owners if 'libc' in p): 'allow'}, 'contexts')])
        self.assertEqual((generated, receipt), ({3: script}, 'receipt'))
        step = {s.crate_name: s for s in plan_units(plan, packages, OUT, scripts=generated,
                                                    source_core=stdlib_targets.source_core(plan))}
        self.assertNotIn('build-script.output', {gap.field for gap in step['compiler_builtins'].gaps})
        core_only = stdlib_targets.load()[1]['core'][0]
        self.assertEqual(stdlib_targets.host_scripts(core_only, {}, [], {}, Path('/out')), ({}, None))


def replace_namespace(value, **changes):
    return SimpleNamespace(**{**vars(value), **changes})


class Expressibility(unittest.TestCase):
    def test_legacy_editions_map_to_their_explicit_driver_option(self):
        units = [row('old', edition='2015'), row('mid', edition='2018', dependencies=[edge(0, 'old')])]
        plan, packages = authored(units, [1])
        steps = {step.crate_name: step for step in plan_units(plan, packages, OUT)}
        for name, edition in (('old', '2015'), ('mid', '2018')):
            self.assertEqual(steps[name].gaps, ())
            self.assertEqual(steps[name].arguments[1], f'--edition={edition}')

    def test_crate_types_flags_profiles_and_modifiers_are_typed_gaps(self):
        flagged = deepcopy(PROFILE) | {'rustflags': ['-Cpanic=abort', '-C', 'panic=abort', '-Zfoo'],
                                       'debuginfo': 1, 'panic': 'unwind', 'opt_level': '3'}
        units = [row('old', edition='2015'), row('shared', kind='dylib'), row('flags', profile=flagged),
                 row('root', dependencies=[edge(0, 'old', noprelude=True), edge(1, 'shared', public=True),
                                           edge(2, 'flags', nounused=True)])]
        plan, packages = authored(units, [3])
        steps = {step.crate_name: step for step in plan_units(plan, packages, OUT)}
        self.assertEqual(steps['old'].gaps, ())
        self.assertEqual([(gap.field, gap.value) for gap in steps['shared'].gaps], [('crate_types', 'dylib')])
        self.assertEqual([(gap.field, gap.value) for gap in steps['flags'].gaps], [
            ('profile.opt_level', '"3"'), ('profile.debuginfo', '1'), ('profile.panic', '"unwind"'),
            ('rustflags', '-Zfoo')])
        self.assertEqual([(gap.field, gap.value) for gap in steps['root'].gaps], [('dependency.noprelude', 'old')])
        self.assertTrue({'public dependency shared', 'nounused dependency flags'} <= set(steps['root'].lint_inputs))

    def test_an_rlib_beside_a_dylib_builds_only_the_rlib_and_other_artifacts_stay_gaps(self):
        def kinds(name, *types):
            unit = row(name)
            unit['target'] |= dict(kind=list(types), crate_types=list(types))
            return unit
        units = [kinds('both', 'dylib', 'rlib'), kinds('lib_both', 'lib', 'dylib'), kinds('only', 'dylib'),
                 kinds('c_only', 'cdylib'), kinds('static', 'rlib', 'staticlib'), kinds('cdylib', 'rlib', 'cdylib'),
                 row('root', dependencies=[edge(0, 'both'), edge(1, 'lib_both')])]
        plan, packages = authored(units, [6, 2, 3, 4, 5])
        steps = {step.crate_name: step for step in plan_units(plan, packages, OUT)}
        for name in ('both', 'lib_both'):
            self.assertEqual((steps[name].gaps, steps[name].dropped_crate_types), ((), ('dylib',)))
            self.assertTrue({'--crate-type=lib', '--metadata-output'} <= set(steps[name].arguments))
        for name, kind in (('only', 'dylib'), ('c_only', 'cdylib'), ('static', 'staticlib'), ('cdylib', 'cdylib')):
            self.assertEqual([(gap.field, gap.value) for gap in steps[name].gaps], [('crate_types', kind)])
            self.assertEqual(steps[name].dropped_crate_types, ())
        self.assertEqual((steps['root'].gaps, steps['root'].dropped_crate_types), ((), ()))

    def test_lint_caps_and_timeouts_are_explicit_per_unit(self):
        steps = plan_frozen('std', timeouts={'core': 240})
        self.assertEqual({name: step.timeout_seconds for name, step in steps.items() if step.timeout_seconds != 120},
                         {'core': 240})
        capped = plan_frozen('core', lint_cap='allow')['core']
        self.assertEqual(capped.lint_cap, 'allow')
        self.assertIn('--cap-lints=allow', capped.arguments)
        for options, message in (({'timeouts': {'missing': 9}}, 'name exactly one'),
                                 ({'timeouts': {'core': 0}}, 'positive whole'),
                                 ({'timeouts': {'core': 1.5}}, 'positive whole'),
                                 ({'lint_cap': 'warn'}, 'only analysis lint cap')):
            with self.subTest(options=options), self.assertRaisesRegex(ValueError, message):
                plan_frozen('core', **options)
        self.assertEqual(stdlib_targets.timeouts(['core=240']), {'core': 240})
        for value in (['core'], ['core=-1'], ['core=1', 'core=2'], ['core=1e3']):
            with self.subTest(value=value), self.assertRaises(ValueError):
                stdlib_targets.timeouts(value)

    def test_a_graph_without_target_units_cannot_plan_success(self):
        plan, packages = authored([row('only')], [0])
        empty = SimpleNamespace(units=(), order=(), roots=())
        with self.assertRaisesRegex(ValueError, 'no target compile unit'):
            plan_units(empty, packages, OUT)
        with self.assertRaisesRegex(ValueError, 'bound package'):
            plan_units(plan, {}, OUT)


FAKE = """#!{python}
import json, sys
from pathlib import Path
args = sys.argv[1:]
value = lambda flag: args[args.index(flag) + 1] if flag in args else None
name = value('--crate-name')
with open({log!r}, 'a') as log:
    log.write(json.dumps(args) + '\\n')
for index, part in enumerate(args):
    if part == '--extern' and not Path(args[index + 1].split('=', 1)[1]).is_file():
        sys.exit(3)
if name in {slow!r}:
    import time
    time.sleep(5)
if name in {usage!r}:
    print('USAGE: refused ' + name, file=sys.stderr)
    sys.exit(2)
Path(value('-o')).write_text('' if name in {fail!r} else 'lowir ' + name)
if name in {fail!r}:
    print('warning: noise', file=sys.stderr)
    print('ERROR: file 0: 7: rejected ' + name, file=sys.stderr)
    sys.exit(1)
if value('--metadata-output') and name not in {nometa!r}:
    Path(value('--metadata-output')).write_text('meta ' + name)
"""


class Execution(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.log = self.root / 'invocations.jsonl'

    def tearDown(self):
        self.temp.cleanup()

    def fake(self, fail=(), nometa=(), slow=(), usage=()):
        path = self.root / 'rustgm'
        path.write_text(FAKE.format(python=sys.executable, log=str(self.log), fail=set(fail), nometa=set(nometa),
                                    slow=set(slow), usage=set(usage)))
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
        return path, hashlib.sha256(path.read_bytes()).hexdigest()

    def graph(self, base_kind='lib', **options):
        units = [row('base', kind=base_kind), row('mid', edition='2021', dependencies=[edge(0, 'base')]),
                 row('other'), row('app', kind='bin', dependencies=[edge(1, 'mid')])]
        plan, packages = authored(units, [3, 2])
        folder = self.root / 'out'
        return plan_units(plan, packages, folder, **options), packages, folder

    def invocations(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def test_complete_graph_runs_dependency_first_and_hashes_every_output(self):
        steps, packages, folder = self.graph()
        rustgm, sha = self.fake()
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['state'], 'complete')
        self.assertEqual([row['crate'] for row in receipt['completed']], ['base', 'mid', 'app', 'other'])
        app = self.invocations()[2]
        self.assertEqual(app[app.index('--extern') + 1], f'mid={folder}/unit-1/mid.rgcrate')
        self.assertEqual(app[app.index('--dependency-metadata') + 1], f'{folder}/unit-0/base.rgcrate')
        self.assertIn('--crate-type=bin', app)
        self.assertNotIn('--metadata-output', app)
        meta = receipt['completed'][0]['outputs']['metadata']
        self.assertEqual(meta['sha256'], hashlib.sha256(b'meta base').hexdigest())
        saved = json.loads((folder / 'receipt.json').read_text())
        self.assertEqual(saved['commands'][1]['environment']['CARGO_CRATE_NAME'], 'mid')
        self.assertEqual(saved['rustgm']['sha256'], sha)

    def test_first_failure_stops_and_blocks_only_its_dependents(self):
        steps, packages, folder = self.graph()
        rustgm, sha = self.fake(fail={'mid'})
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['state'], 'failed')
        self.assertEqual(receipt['failure'], {'unit': 1, 'crate': 'mid', 'kind': 'compile', 'detail': {
            'status': 1, 'diagnostic': 'ERROR: file 0: 7: rejected mid'}})
        self.assertEqual(receipt['blocked'], [{'unit': 3, 'crate': 'app', 'blocked_by': 1}])
        self.assertEqual(receipt['not_run'], [{'unit': 2, 'crate': 'other'}])
        self.assertEqual(len(self.invocations()), 2)
        self.assertFalse((folder / 'unit-1/mid.lowir').exists())
        self.assertTrue((folder / 'unit-1/mid.lowir.rejected').exists())

    def test_timeouts_usage_and_lint_policy_are_typed_and_recorded(self):
        steps, packages, folder = self.graph(timeouts={'mid': 1})
        rustgm, sha = self.fake(slow={'mid'})
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual((receipt['state'], receipt['failure']['kind'], receipt['failure']['detail']),
                         ('failed', 'timeout', {'timeout_seconds': 1}))
        self.assertEqual([row['timeout_seconds'] for row in receipt['commands']], [120, 1])
        self.assertEqual([row['crate'] for row in receipt['blocked']], ['app'])
        self.assertIn('not enforced', receipt['lint_policy']['enforcement'])
        self.assertEqual([step['lint_cap'] for step in receipt['steps']], [None] * 4)
        folder = self.root / 'usage'
        plan, packages = authored([row('base')], [0])
        rustgm, sha = self.fake(usage={'base'})
        receipt = execute(plan_units(plan, packages, folder), packages, rustgm, sha, folder, {})
        self.assertEqual(receipt['failure']['kind'], 'usage')
        self.assertEqual(receipt['failure']['detail'], {'status': 2, 'diagnostic': 'USAGE: refused base'})

    def test_receipts_record_the_dropped_dylib_of_each_completed_unit(self):
        units = [row('base'), row('app', kind='bin', dependencies=[edge(0, 'base')])]
        units[0]['target'] |= dict(kind=['dylib', 'rlib'], crate_types=['dylib', 'rlib'])
        plan, packages = authored(units, [1])
        folder = self.root / 'dropped'
        rustgm, sha = self.fake()
        receipt = execute(plan_units(plan, packages, folder), packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['state'], 'complete')
        saved = json.loads((folder / 'receipt.json').read_text())
        self.assertEqual([row['dropped_crate_types'] for row in saved['steps']], [['dylib'], []])
        self.assertEqual([row['dropped_crate_types'] for row in saved['completed']], [['dylib'], []])
        self.assertIn('--crate-type=lib', self.invocations()[0])

    def test_unsupported_unit_fails_before_any_process_starts(self):
        steps, packages, folder = self.graph(base_kind='dylib')
        rustgm, sha = self.fake()
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['failure']['kind'], 'unsupported')
        self.assertEqual(receipt['failure']['detail'][0]['field'], 'crate_types')
        self.assertEqual([row['unit'] for row in receipt['blocked']], [1, 3])
        self.assertEqual(self.invocations(), [])

    def test_missing_outputs_and_changed_pins_never_pass(self):
        steps, packages, folder = self.graph()
        rustgm, sha = self.fake(nometa={'base'})
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['failure']['kind'], 'output')
        steps, packages, _ = self.graph()
        other = self.root / 'second'
        with self.assertRaisesRegex(ValueError, 'another folder'):
            execute(steps, packages, rustgm, sha, other, {'root': 'authored'})
        steps = plan_units(*authored([row('base')], [0]), other)
        receipt = execute(steps, authored([row('base')], [0])[1], rustgm, '0' * 64, other, {})
        self.assertEqual(receipt['state'], 'infrastructure failure')
        self.assertEqual(receipt['completed'], [])


if __name__ == '__main__':
    unittest.main()
