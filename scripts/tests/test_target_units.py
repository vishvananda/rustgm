"""Frozen target units plan exact RustGM invocations or fail with typed gaps."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import stat
import sys
import tempfile
from types import SimpleNamespace
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from build_script_output import parse as parse_script
from build_units import parse
import stdlib_targets
from target_units import TargetPackage, execute, plan_units

ROOT = Path(__file__).resolve().parents[2]
TRIPLE = 'x86_64-unknown-linux-gnu'
OUT = Path('/out')
# RustGM's fixed course profile, as Cargo would resolve a matching custom profile.
PROFILE = dict(name='release', opt_level='0', lto='false', codegen_backend=None, codegen_units=None,
               debuginfo=0, split_debuginfo=None, debug_assertions=False, overflow_checks=True,
               rpath=False, incremental=False, panic='abort', strip={'deferred': 'None'})


def frozen(root):
    outcomes, graphs = stdlib_targets.load()
    plan, _ = graphs[root]
    packages = stdlib_targets.bind(outcomes, plan, Path('/frozen/rust'), Path('/frozen/vendor'))
    return plan, packages


def plan_frozen(root, scripts=None):
    plan, packages = frozen(root)
    return {step.crate_name: step for step in plan_units(
        plan, packages, OUT / root, source_core=stdlib_targets.source_core(plan), scripts=scripts)}


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
    def test_frozen_graphs_authenticate_and_place_core_first(self):
        _, graphs = stdlib_targets.load()
        self.assertEqual({root: len(plan.units) for root, (plan, _) in graphs.items()},
                         {'core': 1, 'alloc': 5, 'std': 19})
        for root, (plan, _) in graphs.items():
            steps = plan_units(plan, frozen(root)[1], OUT / root, source_core=stdlib_targets.source_core(plan))
            self.assertEqual(steps[0].crate_name, 'core', root)
        with tempfile.TemporaryDirectory() as temp:
            copy = Path(temp)
            for name in ('core', 'alloc', 'std'):
                (copy / f'{name}.json').write_bytes((stdlib_targets.GRAPHS / f'{name}.json').read_bytes())
            (copy / 'core.json').write_bytes((copy / 'core.json').read_bytes() + b' ')
            with self.assertRaisesRegex(ValueError, 'observed hash'):
                stdlib_targets.load(graphs=copy)

    def test_core_is_an_exact_source_core_invocation_with_typed_profile_gaps(self):
        core = plan_frozen('core')['core']
        name = f'core@{core.disambiguator}'
        self.assertRegex(core.disambiguator, r'^h[0-9a-f]{20}$')
        self.assertEqual(core.arguments, (
            '--emit-lowir', '--edition=2024', '--crate-type=lib', '--crate-name', 'core',
            '--crate-disambiguator', core.disambiguator, '--ub-checks=no', '--source-core', name,
            '--metadata-output', '/out/core/unit-0/core.rgcrate', '-o', '/out/core/unit-0/core.lowir',
            '/frozen/rust/library/core/src/lib.rs'))
        self.assertEqual([(gap.field, gap.value) for gap in core.gaps], [
            ('profile.debuginfo', '2'), ('profile.debug_assertions', 'true'),
            ('package.environment', core.package)])
        # Package identities, not invocation paths, select the disambiguator.
        outcomes, graphs = stdlib_targets.load()
        plan, _ = graphs['core']
        moved = stdlib_targets.bind(outcomes, plan, Path('/elsewhere'), Path('/vendor'))
        self.assertEqual(plan_units(plan, moved, OUT / 'x')[0].disambiguator, core.disambiguator)
        self.assertEqual(plan_frozen('std')['core'].disambiguator, core.disambiguator)

    def test_every_std_unit_reports_all_settings_rustgm_cannot_express(self):
        steps = plan_frozen('std')
        profile = {'profile.debuginfo', 'profile.debug_assertions', 'package.environment'}
        expected = {name: set(profile) for name in steps}
        expected['cfg_if'].add('edition')
        expected['rustc_demangle'].add('edition')
        expected['std'] |= {'crate_types', 'build-script.output'}
        expected['compiler_builtins'].add('build-script.output')
        expected['libc'].add('build-script.output')
        self.assertEqual({name: {gap.field for gap in step.gaps} for name, step in steps.items()}, expected)
        self.assertEqual(list(steps), ['core', 'compiler_builtins', 'alloc', 'rustc_std_workspace_core',
                                       'cfg_if', 'rustc_std_workspace_alloc', 'hashbrown', 'libc',
                                       'panic_abort', 'rustc_demangle', 'std_detect', 'unwind', 'std'])
        self.assertIn(('crate_types', 'dylib'), {(gap.field, gap.value) for gap in steps['std'].gaps})
        # panic_abort's -Cpanic=abort only restates the fixed abort target.
        self.assertNotIn('rustflags', {gap.field for gap in steps['panic_abort'].gaps})

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
        template = ROOT / 'pa34/tests/spec/stdlib-host/compiler-builtins/expected.stdout.template'
        data = template.read_bytes().replace(b'{PACKAGE_ROOT}', b'/frozen/rust/library/compiler-builtins/compiler-builtins')
        script = SimpleNamespace(output=parse_script(data, 0), folder=Path('/out/script'), key='frozen')
        step = plan_frozen('std', {3: script})['compiler_builtins']
        self.assertNotIn('build-script.output', {gap.field for gap in step.gaps})
        cfgs = [step.arguments[i + 1] for i, part in enumerate(step.arguments) if part == '--cfg']
        self.assertEqual(cfgs, ['feature="arch"', 'feature="compiler-builtins"', 'feature="default"',
                                'feature="unmangled-names"', 'intrinsics_enabled', 'mem_unaligned'])
        self.assertEqual(dict(step.environment)['OUT_DIR'], '/out/script')
        declared = data.count(b'rustc-check-cfg=')
        self.assertEqual(declared, 135)
        # Every script expectation plus Cargo's two implicit feature/docsrs rows.
        self.assertEqual(sum(row.startswith('check-cfg cfg(') for row in step.lint_inputs), declared + 2)
        self.assertIn('metadata compiler-rt /frozen/rust/library/compiler-builtins/compiler-builtins/compiler-rt',
                      step.recorded_directives)
        unscripted = plan_frozen('std')['compiler_builtins']
        self.assertNotEqual(unscripted.disambiguator, step.disambiguator)


class Expressibility(unittest.TestCase):
    def test_editions_crate_types_flags_profiles_and_modifiers_are_typed_gaps(self):
        flagged = deepcopy(PROFILE) | {'rustflags': ['-Cpanic=abort', '-C', 'panic=abort', '-Zfoo'],
                                       'debuginfo': 1, 'panic': 'unwind', 'opt_level': '3'}
        units = [row('old', edition='2015'), row('shared', kind='dylib'), row('flags', profile=flagged),
                 row('root', dependencies=[edge(0, 'old', noprelude=True), edge(1, 'shared', public=True),
                                           edge(2, 'flags', nounused=True)])]
        plan, packages = authored(units, [3])
        steps = {step.crate_name: step for step in plan_units(plan, packages, OUT)}
        self.assertEqual([(gap.field, gap.value) for gap in steps['old'].gaps], [('edition', '2015')])
        self.assertEqual([(gap.field, gap.value) for gap in steps['shared'].gaps], [('crate_types', 'dylib')])
        self.assertEqual([(gap.field, gap.value) for gap in steps['flags'].gaps], [
            ('profile.opt_level', '"3"'), ('profile.debuginfo', '1'), ('profile.panic', '"unwind"'),
            ('rustflags', '-Zfoo')])
        self.assertEqual([(gap.field, gap.value) for gap in steps['root'].gaps], [('dependency.noprelude', 'old')])
        self.assertTrue({'public dependency shared', 'nounused dependency flags'} <= set(steps['root'].lint_inputs))

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
Path(value('-o')).write_text('' if name in {fail!r} else 'lowir ' + name)
if name in {fail!r}:
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

    def fake(self, fail=(), nometa=()):
        path = self.root / 'rustgm'
        path.write_text(FAKE.format(python=sys.executable, log=str(self.log), fail=set(fail), nometa=set(nometa)))
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
        return path, hashlib.sha256(path.read_bytes()).hexdigest()

    def graph(self, base_edition='2024'):
        units = [row('base', edition=base_edition), row('mid', edition='2021', dependencies=[edge(0, 'base')]),
                 row('other'), row('app', kind='bin', dependencies=[edge(1, 'mid')])]
        plan, packages = authored(units, [3, 2])
        folder = self.root / 'out'
        return plan_units(plan, packages, folder), packages, folder

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
        self.assertEqual(receipt['failure'], {'unit': 1, 'crate': 'mid', 'kind': 'compile', 'detail': {'status': 1}})
        self.assertEqual(receipt['blocked'], [{'unit': 3, 'crate': 'app', 'blocked_by': 1}])
        self.assertEqual(receipt['not_run'], [{'unit': 2, 'crate': 'other'}])
        self.assertEqual(len(self.invocations()), 2)
        self.assertFalse((folder / 'unit-1/mid.lowir').exists())
        self.assertTrue((folder / 'unit-1/mid.lowir.rejected').exists())

    def test_unsupported_unit_fails_before_any_process_starts(self):
        steps, packages, folder = self.graph(base_edition='2018')
        rustgm, sha = self.fake()
        receipt = execute(steps, packages, rustgm, sha, folder, {'root': 'authored'})
        self.assertEqual(receipt['failure']['kind'], 'unsupported')
        self.assertEqual(receipt['failure']['detail'][0]['field'], 'edition')
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
