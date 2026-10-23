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
from host_build_script import digest
from planned_build_script import execute, pair
import test_build_units as units
import test_package_snapshot as snapshots


def setup(root):
    package = snapshots.snapshot(root, [('sample-1.0.0/build.rs', b'fn main() {}')])
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

    def test_manifest_compiler_policy_is_never_silently_ignored(self):
        with tempfile.TemporaryDirectory() as temporary:
            package, _, _ = setup(Path(temporary))
            manifest = package.root / 'Cargo.toml'
            manifest.write_text(manifest.read_text() + '\n[lints.rust]\nunused="deny"\n')
            with self.assertRaisesRegex(ValueError, 'manifest lint/unstable policy'):
                package_environment(package)


if __name__ == '__main__':
    unittest.main()
