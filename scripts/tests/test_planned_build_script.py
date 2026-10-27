"""Host execution uses exact graph ownership and separately frozen target facts."""
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
from build_environment import Context, package_environment
import host_profile
import manifest_lints
from host_build_script import digest
from planned_build_script import execute, pair
import test_build_units as units
import test_package_snapshot as snapshots


def setup(root, lint_table='', *, manifest_prefix=''):
    source = snapshots.archive(root, [('sample-1.0.0/build.rs', b'fn main() {}')],
                               manifest=(manifest_prefix + '[package]\nname="sample"\nversion="1.0.0"\n'
                                         'edition="2021"\n' + lint_table).encode())
    vendor = root / 'vendor'
    vendor.mkdir()
    package = snapshots.extract(source, digest(source), 'sample', '1.0.0', vendor)
    compiler = root / 'compiler'
    compiler.write_bytes(b'pinned host compiler')
    facts = root / 'context.json'
    facts.write_text('{"target_arch":"x86_64","panic":"abort"}')
    context = Context(units.TRIPLE, units.TRIPLE, compiler, digest(compiler),
                      (('target_arch', ('x86_64',)), ('panic', ('abort',)), ('unix', ())),
                      ((facts, digest(facts)),))
    data = dict(version=1, roots=[0], units=[
        units.row(package.cargo_id, dependencies=[units.edge(1, 'build_script_build')]),
        units.row(package.cargo_id, 'custom-build', mode='run-custom-build',
                  dependencies=[units.edge(2, 'build_script_build')]),
        units.row(package.cargo_id, 'custom-build', host=True)])
    for index, row in enumerate(data['units']):
        row['target']['src_path'] = str(package.root / ('src/lib.rs' if index == 0 else 'build.rs'))
        row['target']['edition'] = '2021'
    data['units'][1]['profile'].update(opt_level='2', panic='abort')
    data['units'][2]['profile'].update(debug_assertions=False, overflow_checks=False)
    return package, context, data


class Profiles(unittest.TestCase):
    def test_observed_inferred_strip_profile_keeps_debug_zero_semantics(self):
        profile = deepcopy(units.PROFILE)
        profile.update(debuginfo=0, strip={'resolved': {'Named': 'debuginfo'}})
        self.assertIn('strip=debuginfo', host_profile.arguments(json.dumps(profile)))

    def test_codegen_flags_preserve_explicit_false_settings_and_lto_semantics(self):
        profile = deepcopy(units.PROFILE)
        profile.update(debug_assertions=False, overflow_checks=False, codegen_units=2,
                       split_debuginfo='off', panic='abort')
        flags = host_profile.arguments(json.dumps(profile))
        for setting in ('debug-assertions=no', 'overflow-checks=no', 'codegen-units=2',
                        'split-debuginfo=off', 'panic=abort', 'embed-bitcode=no'):
            self.assertIn(setting, flags)
        self.assertFalse(any(flag.startswith('lto=') for flag in flags))
        profile['lto'] = 'off'
        self.assertIn('lto=off', host_profile.arguments(json.dumps(profile)))

    def test_debug_environment_means_debug_information_not_assertions(self):
        profile = deepcopy(units.PROFILE)
        profile.update(debug_assertions=False, opt_level='3')
        self.assertEqual(host_profile.environment(json.dumps(profile)),
                         {'DEBUG': 'true', 'OPT_LEVEL': '3', 'PROFILE': 'debug'})

    def test_unknown_or_unsupported_profile_fields_never_disappear(self):
        variants = [('extra', 'ignored'), ('name', 'custom-with-unknown-parent'),
                    ('codegen_backend', '/unrecorded'), ('incremental', True),
                    ('strip', {'resolved': 'Symbols'}), ('debug_assertions', 1),
                    ('codegen_units', 1024), ('debuginfo', True)]
        for key, value in variants:
            profile = deepcopy(units.PROFILE)
            profile[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                host_profile.arguments(json.dumps(profile))


class Tasks(unittest.TestCase):
    @staticmethod
    def process(command, *, cwd, env, stdout, stderr, timeout):
        if '--crate-name' in command:
            assert 'debug-assertions=no' in command and 'panic=unwind' in command
            assert env['CARGO_PKG_NAME'] == 'sample'
            assert env['CARGO_PKG_VERSION_PATCH'] == '0'
            assert 'TARGET' not in env
            Path(command[command.index('-o') + 1]).write_bytes(b'compiled host tool')
        else:
            assert env['CARGO_CFG_PANIC'] == 'abort' and env['DEBUG'] == 'true'
            assert env['OPT_LEVEL'] == '2' and env['TARGET'] == units.TRIPLE
            assert env['RUSTC'].endswith('/cargo_query.py')
            (Path(env['OUT_DIR']) / 'generated.rs').write_text('pub const VALUE:u8=7;')
            stdout.write(b'cargo:rustc-check-cfg=cfg(enabled)\ncargo:rustc-cfg=enabled\n')
        return subprocess.CompletedProcess(command, 0)

    def test_exact_pair_generates_source_without_target_compilation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package, context, data = setup(root)
            with patch('host_build_script.subprocess.run', side_effect=self.process) as run:
                result = execute(units.decode(data), 1, package, context, root / 'output')
            self.assertEqual(run.call_count, 2)
            self.assertEqual(result.cfg_arguments(), ('--cfg', 'enabled'))
            receipt = json.loads((root / 'output/receipt.json').read_text())
            self.assertEqual(receipt['state'], 'complete')
            self.assertEqual(receipt['package'], package.identity)
            self.assertEqual(receipt['compile']['platform'], None)
            self.assertEqual(receipt['run']['platform'], units.TRIPLE)
            self.assertEqual(receipt['generated'], {'generated.rs': digest(root / 'output/host/generated/generated.rs')})

    def test_target_unit_wrong_package_and_wrong_context_never_launch(self):
        for kind in ('target', 'package', 'context'):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                package, context, data = setup(root)
                selected = 0 if kind == 'target' else 1
                if kind == 'package':
                    package = replace(package, version='2.0.0')
                if kind == 'context':
                    context = replace(context, target='aarch64-unknown-linux-gnu')
                with patch('host_build_script.subprocess.run') as run, self.assertRaises(ValueError):
                    execute(units.decode(data), selected, package, context, root / 'output')
                run.assert_not_called()
                self.assertFalse((root / 'output').exists())

    def test_unbuilt_host_dependencies_are_diagnosed_not_omitted(self):
        with tempfile.TemporaryDirectory() as temporary:
            package, context, data = setup(Path(temporary))
            data['units'].append(units.row('host_dependency', host=True))
            data['units'][2]['dependencies'] = [units.edge(3, 'renamed')]
            with self.assertRaisesRegex(ValueError, 'host dependency scheduling'):
                pair(units.decode(data), 1, package, context)

    def test_swallowed_rejected_compiler_probe_invalidates_script_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package, context, data = setup(root)

            def process(command, **kwargs):
                result = self.process(command, **kwargs)
                if '--crate-name' not in command:
                    Path(kwargs['env']['RUSTGM_PLAN_QUERY_LOG']).write_text('{"state":"rejected"}\n')
                return result

            with patch('host_build_script.subprocess.run', side_effect=process), \
                    self.assertRaisesRegex(ValueError, 'unsupported or failed compiler query'):
                execute(units.decode(data), 1, package, context, root / 'output')
            receipt = json.loads((root / 'output/receipt.json').read_text())
            self.assertEqual(receipt['state'], 'failed')
            self.assertNotIn('generated', receipt)

    def test_changed_configuration_pin_fails_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package, context, data = setup(root)
            context.inputs[0][0].write_text('changed facts')
            with patch('host_build_script.subprocess.run') as run, self.assertRaisesRegex(ValueError, 'pin'):
                execute(units.decode(data), 1, package, context, root / 'output')
            run.assert_not_called()

    def test_cfg_and_package_environment_are_explicit_and_unambiguous(self):
        with tempfile.TemporaryDirectory() as temporary:
            package, context, _ = setup(Path(temporary))
            environment = package_environment(package)
            self.assertEqual(environment['CARGO_PKG_VERSION_MAJOR'], '1')
            self.assertEqual(environment['CARGO_PKG_VERSION_PRE'], '')
            self.assertEqual(context.cfg_environment()['CARGO_CFG_UNIX'], '')
            for cfg in ((('feature', ('unselected',)),), (('target_feature', ('a,b',)),),
                        (('panic', ('abort',)), ('panic', ('unwind',)))):
                with self.assertRaises(ValueError):
                    replace(context, cfg=cfg).validate()

    def test_manifest_lints_reach_real_task_command_and_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package, context, data = setup(root, '\n[lints.rust]\nunused="deny"\n'
                'dead_code={level="allow",priority=1}\n[lints.clippy]\nall="warn"\n')
            with patch('host_build_script.subprocess.run', side_effect=self.process) as run:
                execute(units.decode(data), 1, package, context, root / 'output', lint_cap='warn')
            command = run.call_args_list[0].args[0]
            self.assertLess(command.index('--deny=unused'), command.index('--allow=dead_code'))
            self.assertIn('--warn=clippy::all', command)
            self.assertEqual(command[command.index('--cap-lints') + 1], 'warn')
            receipt = json.loads((root / 'output/host/receipt.json').read_text())
            self.assertEqual(receipt['lints']['rust']['unused'], 'deny')
            self.assertEqual(receipt['lint_cap'], 'warn')
            self.assertIn('--deny=unused', receipt['commands'][0]['command'])

    def test_unknown_manifest_policy_still_fails_before_execution(self):
        for table in ('\n[lints]\nworkspace=true\n', '\ncargo-features=["unknown"]\n',
                      '\n[lints.rust]\nunused="expect"\n'):
            with self.subTest(table=table), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                if 'cargo-features' in table:
                    # This key is at the manifest root, not under [package].
                    package, context, data = setup(root)
                    document = {'cargo-features': ['unknown']}
                    with self.assertRaisesRegex(ValueError, 'unstable policy'):
                        package_environment(package, document)
                    continue
                package, context, data = setup(root, table)
                with patch('host_build_script.subprocess.run') as run, self.assertRaises(ValueError):
                    execute(units.decode(data), 1, package, context, root / 'output')
                run.assert_not_called()

    def test_public_dependency_feature_preserves_target_edges_and_denies_host_modifiers(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package, context, data = setup(root,
                manifest_prefix='cargo-features=["public-dependency"]\n')
            data['units'].append(units.row('public_target'))
            edge = units.edge(3, 'public_target')
            edge['public'] = True
            data['units'][0]['dependencies'].append(edge)
            plan = units.decode(data)
            self.assertTrue(plan.units[0].dependencies[-1].public)
            with patch('host_build_script.subprocess.run', side_effect=self.process) as run:
                execute(plan, 1, package, context, root / 'output')
            self.assertEqual(run.call_count, 2)
            self.assertNotIn('--extern', run.call_args_list[0].args[0])
            for modifier in ('public', 'noprelude', 'nounused'):
                invalid = deepcopy(data)
                invalid['units'][1]['dependencies'][0][modifier] = True
                with self.subTest(modifier=modifier), patch('host_build_script.subprocess.run') as run, \
                        self.assertRaisesRegex(ValueError, 'host dependency modifiers'):
                    execute(units.decode(invalid), 1, package, context, root / modifier)
                run.assert_not_called()
                self.assertFalse((root / modifier).exists())


class Lints(unittest.TestCase):
    def test_frozen_cargo_priorities_and_cfg_expectations_remain_ordered(self):
        flags = manifest_lints.arguments({'rust': {
            'unused': {'level': 'deny', 'priority': -1},
            'dead_code': {'level': 'allow', 'priority': 1},
            'unexpected_cfgs': {'level': 'deny', 'check-cfg': ['cfg(host_lint_probe)']},
        }, 'clippy': {'all': {'level': 'warn', 'priority': -1}}})
        self.assertEqual(flags, ('--deny=unused', '--warn=clippy::all',
                                '--deny=unexpected_cfgs', '--allow=dead_code',
                                '--check-cfg', 'cfg(host_lint_probe)'))

    def test_unimplemented_and_malformed_policies_cannot_inject_or_discard_flags(self):
        variants = [{'workspace': True}, {'cargo': {'unknown': 'allow'}},
                    {'rust': {'dead_code': {'level': 'allow', 'priority': True}}},
                    {'rust': {'dead_code': {'level': 'allow', 'priority': 128}}},
                    {'rust': {'dead_code': {'level': 'allow', 'extra': 'ignored'}}},
                    {'rust': {'dead_code': {'level': 'allow', 'check-cfg': []}}},
                    {'rust': {'unexpected_cfgs': {'level': 'deny', 'check-cfg': ['cfg(x)\n']}}},
                    {'clippy': {'all': {'level': 'warn', 'check-cfg': ['cfg(x)']}}},
                    {'rust': {'dead_code -o injected': 'allow'}}]
        for table in variants:
            with self.subTest(table=table), self.assertRaises(ValueError):
                manifest_lints.arguments(table)
        for cap in ('', '--deny=all', True, 1):
            with self.subTest(cap=cap), self.assertRaises(ValueError):
                manifest_lints.cap_arguments(cap)


if __name__ == '__main__':
    unittest.main()
