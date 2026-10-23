"""Host output preserves cfg authority, linker order and explicit failures."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import build_script_output as output
from build_script_output import Kind, parse
from host_build_script import Job, digest, execute, generated_files


class BuildScriptOutput(unittest.TestCase):
    def test_cfg_expectations_do_not_enable_configuration(self):
        result = parse(b'cargo:rustc-check-cfg=cfg(fast, values("32", "64"))\n'
                       b'cargo::rustc-cfg=fast="64"\n', 0)
        self.assertEqual(result.cfg_arguments(), ('--cfg', 'fast="64"'))
        self.assertEqual([row.kind for row in result.directives], [Kind.CHECK_CFG, Kind.CFG])

    def test_native_arguments_keep_order_scope_and_embedded_equals(self):
        result = parse(b'cargo::rustc-link-arg=first.o\n'
                       b'cargo:rustc-flags=-L native=lib -lstatic=foo\n'
                       b'cargo:rustc-link-arg-bin=demo=-Wl,name=value\n'
                       b'cargo:rustc-link-lib=bar\n'
                       b'cargo:rustc-cdylib-link-arg=last.o\n', 0)
        self.assertEqual([row.arguments for row in result.directives], [
            ('all-supported', 'first.o'), ('native=lib',), ('static=foo',),
            ('bin', 'demo', '-Wl,name=value'), ('bar',), ('cdylib', 'last.o')])
        self.assertEqual([row.line for row in result.directives], [1, 2, 2, 3, 4, 5])

    def test_environment_and_metadata_remain_separate(self):
        result = parse(b'cargo:rustc-env=VALUE=old\n'
                       b'cargo::rustc-env=VALUE=a=b\n'
                       b'cargo:rustc-env=EMPTY=\n'
                       b'cargo:include=path=part\n'
                       b'cargo::metadata=include=another\n', 0)
        self.assertEqual(result.environment(), {'VALUE': 'a=b', 'EMPTY': ''})
        self.assertEqual(result.directives[-2].arguments, ('include', 'path=part'))
        self.assertEqual(result.directives[-1].kind, Kind.METADATA)
        self.assertEqual(result.cfg_arguments(), ())

    def test_non_directive_bytes_are_not_parsed_as_rust_or_commands(self):
        result = parse(b'\xff ordinary log\n cargo::error=not an instruction\n'
                       b'cargo:warning=hello\r\n'
                       b'cargo::rustc-cfg=enabled', 0)
        self.assertEqual(result.directives[0].arguments, ('hello',))
        self.assertEqual(result.cfg_arguments(), ('--cfg', 'enabled'))

    def test_failure_is_never_an_empty_success(self):
        for status in (1, 86, 101, -6, -9):
            with self.subTest(status=status), self.assertRaisesRegex(ValueError, 'status'):
                parse(b'cargo:rustc-cfg=partial\n', status)
        with self.assertRaisesRegex(ValueError, 'reported an error'):
            parse(b'cargo:rustc-cfg=partial\ncargo::error=failed after output\n', 0)

    def test_malformed_and_unimplemented_directives_are_diagnosed(self):
        for text in (b'cargo::unknown=value', b'cargo:=value', b'cargo:rustc-env=NAME',
                     b'cargo:rustc-cfg=', b'cargo:rustc-link-arg-bin=demo=',
                     b'cargo:rustc-cfg=\xff', b'cargo:rustc-env=A=\0'):
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, 'line 1'):
                parse(text, 0)
        for text in (b'-Copt-level=3', b'-l', b'-L', b'--extern foo=x'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse(b'cargo:rustc-flags=' + text, 0)

    def test_budgets_cover_expanded_link_flags_and_full_output(self):
        with patch.object(output, 'MAX_BYTES', 3), self.assertRaisesRegex(ValueError, 'byte budget'):
            parse(b'logs', 0)
        with patch.object(output, 'MAX_LINE', 8), self.assertRaisesRegex(ValueError, 'line 1'):
            parse(b'cargo:warning=message', 0)
        with patch.object(output, 'MAX_DIRECTIVES', 1), self.assertRaisesRegex(ValueError, 'count'):
            parse(b'cargo:rustc-flags=-lone -ltwo', 0)

    def test_rerun_inputs_preserve_real_paths_and_environment_names(self):
        result = parse(b'cargo:rerun-if-changed=directory with spaces/header.h\n'
                       b'cargo::rerun-if-env-changed=CC\n', 0)
        self.assertEqual([row.kind for row in result.directives],
                         [Kind.RERUN_FILE, Kind.RERUN_ENVIRONMENT])
        self.assertEqual(result.directives[0].arguments, ('directory with spaces/header.h',))


class HostTask(unittest.TestCase):
    def job(self, root):
        package = root / 'package'
        package.mkdir()
        source = package / 'build.rs'
        source.write_text('fn main() {}')
        compiler = root / 'compiler'
        compiler.write_bytes(b'pinned host compiler')
        return Job(package, source, '2021', compiler, digest(compiler),
                   {'build.rs': digest(source)}, {'COMPILE_ONLY': 'yes'},
                   {'TARGET': 'x86_64-unknown-linux-gnu'}, ('some-feature',))

    @staticmethod
    def process(command, *, cwd, env, stdout, stderr, timeout):
        if '--crate-name' in command:
            Path(command[command.index('-o') + 1]).write_bytes(b'host executable')
        else:
            assert env['CARGO_FEATURE_SOME_FEATURE'] == '1'
            assert 'COMPILE_ONLY' not in env
            (Path(env['OUT_DIR']) / 'generated.rs').write_text('pub const VALUE:u8=7;')
            stdout.write(b'cargo::rustc-cfg=generated\ncargo:rustc-env=VALUE=7\n')
        return subprocess.CompletedProcess(command, 0)

    def test_explicit_host_job_publishes_only_complete_generated_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job = self.job(root)
            folder = root / 'output'
            with patch('host_build_script.subprocess.run', side_effect=self.process) as run:
                result = execute(job, folder)
            self.assertEqual(run.call_count, 2)
            self.assertEqual(result.cfg_arguments(), ('--cfg', 'generated'))
            receipt = json.loads((folder / 'receipt.json').read_text())
            self.assertEqual(receipt['state'], 'complete')
            self.assertEqual(receipt['inputs'], job.inputs)
            self.assertEqual(receipt['generated'], {
                'generated.rs': digest(folder / 'generated/generated.rs')})
            self.assertEqual(receipt['commands'][0]['environment'], {'COMPILE_ONLY': 'yes'})
            self.assertEqual(receipt['commands'][1]['cwd'], str(job.package))
            with self.assertRaisesRegex(ValueError, 'already exists'):
                execute(job, folder)

    def test_failed_execution_retains_diagnostics_without_publishing_directives(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job = self.job(root)

            def fail(command, **kwargs):
                result = self.process(command, **kwargs)
                return result if '--crate-name' in command else subprocess.CompletedProcess(command, 86)

            with patch('host_build_script.subprocess.run', side_effect=fail), \
                    self.assertRaisesRegex(ValueError, 'status 86'):
                execute(job, root / 'output')
            receipt = json.loads((root / 'output/receipt.json').read_text())
            self.assertEqual(receipt['state'], 'failed')
            self.assertNotIn('directives', receipt)
            self.assertTrue((root / 'output/execute.stdout').read_bytes())

    def test_changed_pin_fails_before_any_process_or_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            job = self.job(root)
            job.source.write_text('changed input')
            with patch('host_build_script.subprocess.run') as run, \
                    self.assertRaisesRegex(ValueError, 'changed'):
                execute(job, root / 'output')
            run.assert_not_called()
            self.assertFalse((root / 'output').exists())

    def test_generated_symlinks_never_read_files_outside_the_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            generated = root / 'generated'
            generated.mkdir()
            outside = root / 'private'
            outside.write_text('private')
            (generated / 'link').symlink_to(outside)
            with self.assertRaisesRegex(ValueError, 'regular file'):
                generated_files(generated)
            (generated / 'link').unlink()
            (generated / 'link').symlink_to(root, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'symlink'):
                generated_files(generated)


if __name__ == '__main__':
    unittest.main()
