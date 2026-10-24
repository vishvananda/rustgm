"""Variable reservations must not borrow authority from stale slot records."""
from contextlib import ExitStack
import fcntl
import io
import json
import multiprocessing
import os
from pathlib import Path
import queue
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from resource_locks import (active_records, inherited, inherited_budget, lease,
                            peer_budget, process_identity, try_lock)
from resource_policy import (DEFAULTS, GIB, OutputLimitError, atomic_json,
                             output_budget, policy_identity, usage, validate)
from resource_runner import finished, run, stop, supervise
from resources import set_schema
import storage


def capped_policy(root, jobs=3):
    return {**DEFAULTS, 'schema': 2, 'jobs': jobs,
            'worktrees': {str(root): {'name': 'test', 'outputs': ['obj']}}}


def paused_publication(directory, ready, publish, release):
    directory = Path(directory)
    original = atomic_json
    def write(path, record):
        ready.set()
        if not publish.wait(5):
            raise RuntimeError('publication test timed out')
        original(path, record)
    with patch('resource_locks.atomic_json', side_effect=write):
        with lease(directory, Path('/publisher'), {**DEFAULTS, 'schema': 2},
                   'ordinary', wait=0, budget=8 * GIB):
            release.wait(5)


def take_snapshot(directory, started, results):
    started.set()
    results.put(active_records(Path(directory), 2))


class OutputCapTests(unittest.TestCase):
    def test_cap_is_opt_in_absolute_and_bounded_by_configuration(self):
        policy = {**DEFAULTS, 'schema': 2}
        validate(policy)
        with self.assertRaises(ValueError):
            validate({**policy, 'schema': 3})
        self.assertEqual(output_budget(DEFAULTS), 50 * GIB)
        self.assertEqual(output_budget(policy), 50 * GIB)
        self.assertEqual(output_budget(policy, 8), 8 * GIB)
        self.assertEqual(output_budget(policy, .5), GIB // 2)
        self.assertEqual(output_budget(policy, 50), 50 * GIB)
        with self.assertRaisesRegex(ValueError, 'schema 2'):
            output_budget(DEFAULTS, 50)
        for value in [0, -1, 51, float('nan'), float('inf'), True, 1 / (2 * GIB)]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                output_budget(policy, value)
        with self.assertRaises(ValueError):
            output_budget({**policy, 'worker_gib': 16}, 17)
        with patch('resource_runner.discover', return_value=None), \
             patch('resource_runner.subprocess.run') as launch:
            with self.assertRaisesRegex(ValueError, 'configured'):
                run(Path('/unconfigured'), ['unused'], {}, max_output_gib=8)
            launch.assert_not_called()

    def test_unknown_acquiring_legacy_and_malformed_peers_reserve_configured_cap(self):
        policy = {**DEFAULTS, 'schema': 2}
        row = {'schema': 2, 'policy_identity': policy_identity(policy),
               'state': 'acquired', 'budget_bytes': 8 * GIB,
               'pid': os.getpid(), 'process_start': process_identity(os.getpid())[1]}
        self.assertEqual(peer_budget(row, policy), 8 * GIB)
        for changes in [{'schema': 1}, {'state': 'acquiring'}, {'state': 'finished'},
                        {'policy_identity': 'foreign'}, {'budget_bytes': None},
                        {'budget_bytes': 0}, {'budget_bytes': True},
                        {'budget_bytes': 51 * GIB}, {'pid': 999_999_999},
                        {'process_start': -1}]:
            with self.subTest(changes=changes):
                self.assertEqual(peer_budget({**row, **changes}, policy), 50 * GIB)
        self.assertEqual(peer_budget({}, policy), 50 * GIB)
        self.assertEqual(peer_budget(row, DEFAULTS), 50 * GIB)

    def test_mixed_reservations_include_unused_self_and_full_unknown_peer(self):
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            root = Path(folder)
            policy = capped_policy(root)
            directory = root / 'control'
            stack.enter_context(lease(directory, Path('/peer'), policy,
                                      'ordinary', wait=0, budget=16 * GIB))
            # Simulate an acquiring/legacy owner with a stale small JSON row.
            unknown = stack.enter_context((directory / 'slot-1.lock').open('a+'))
            fcntl.flock(unknown, fcntl.LOCK_EX)
            atomic_json(directory / 'slot-1.json',
                        {'slot': 1, 'state': 'acquiring', 'budget_bytes': GIB})
            with patch('resource_runner.discover', return_value=(directory, policy)), \
                 patch('resource_runner.usage', return_value=6 * GIB), \
                 patch('resource_runner.free_space') as space, \
                 patch('resource_runner.supervise', return_value=0) as launch:
                self.assertEqual(run(root, ['unused'], {}, wait=0, max_output_gib=8), 0)
                space.assert_called_once_with(root, policy, (2 + 16 + 50) * GIB)
                record = launch.call_args.args[5]
                self.assertEqual(record['budget_bytes'], 8 * GIB)
                self.assertEqual(record['reservation_bytes'], 68 * GIB)
                self.assertEqual(policy['worker_gib'], 50)

    def test_cap_failure_before_launch_counts_pins_and_cache_and_keeps_receipt(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            policy = capped_policy(root)
            directory = root / 'control'
            for name in ['rust/cache', 'pins/binary']:
                path = root / 'obj' / name
                path.parent.mkdir(parents=True)
                path.write_bytes(b'x' * 8192)
            (root / 'obj/pins/.keep-artifacts').touch()
            with patch('resource_runner.discover', return_value=(directory, policy)), \
                 patch('resource_policy.free_space'), \
                 patch('resource_runner.supervise') as launch:
                with self.assertRaises(OutputLimitError):
                    run(root, ['unused'], {}, wait=0, max_output_gib=4096 / GIB)
                launch.assert_not_called()
            receipts = list((root / 'obj/parallel/resources').glob('*.json'))
            self.assertEqual(len(receipts), 1)
            receipt = json.loads(receipts[0].read_text())
            self.assertEqual(receipt['budget_bytes'], 4096)
            self.assertGreaterEqual(receipt['used_bytes'], 16384)
            self.assertEqual(receipt['returncode'], 1)
            self.assertEqual(receipt['state'], 'finished')
            self.assertNotIn('token', receipt)
            self.assertEqual(active_records(directory, 3), [])

    def test_usage_at_the_limit_succeeds_and_final_scan_cannot_skip_a_short_command(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            policy = capped_policy(root)
            directory = root / 'control'
            directory.mkdir()
            process = Mock(pid=123, returncode=0)
            process.poll.return_value = 0
            record = {'token': 'final-cap', 'slot': 0, 'budget_bytes': 4096}
            with patch('resource_policy.UsageScan') as scan, patch('resource_policy.free_space'):
                scan.return_value.done = False
                def complete():
                    scan.return_value.done = True
                    scan.return_value.total = 4096
                scan.return_value.step.side_effect = complete
                self.assertEqual(usage(root, {'outputs': ['obj']}, policy, 4096), 4096)
            with patch('resource_runner.subprocess.Popen', return_value=process), \
                 patch('resource_runner.scope_active', return_value=False), \
                 patch('resource_runner.usage', side_effect=OutputLimitError(8192, 4096)), \
                 patch('resource_runner.stop') as terminate, patch('sys.stderr', io.StringIO()):
                result = supervise(['unused'], root, {}, policy, {'outputs': ['obj']},
                                   record, directory / 'slot-0.json', ())
                terminate.assert_called_once_with(process, 'rustgm-job-final-cap.scope')
            self.assertEqual(result, 1)
            receipt = json.loads((root / 'obj/parallel/resources/final-cap.json').read_text())
            self.assertEqual(receipt['used_bytes'], 8192)
            self.assertEqual(receipt['budget_bytes'], 4096)
            self.assertEqual(receipt['limits']['worker_gib'], 50)

    def test_live_incremental_overshoot_stops_scope_and_records_partial_scan(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            policy = capped_policy(root)
            process = Mock(pid=123, returncode=0)
            process.poll.return_value = None
            record = {'token': 'live-cap', 'budget_bytes': 4096}
            with patch('resource_runner.subprocess.Popen', return_value=process), \
                 patch('resource_runner.UsageScan') as scan, \
                 patch('resource_runner.free_space'), \
                 patch('resource_runner.stop') as terminate, patch('sys.stderr', io.StringIO()):
                scan.return_value.total = 8192
                scan.return_value.done = False
                result = supervise(['unused'], root, {}, policy, {'outputs': ['obj']},
                                   record, directory / 'slot-0.json', ())
                terminate.assert_called_once_with(process, 'rustgm-job-live-cap.scope')
                scan.return_value.step.assert_called_once()
                scan.return_value.close.assert_called_once()
            self.assertEqual(result, 1)
            self.assertEqual(record['used_bytes'], 8192)
            self.assertIn('4096 bytes', record['error'])

    def test_stopping_scope_still_runs_after_launcher_has_exited(self):
        process = Mock()
        process.poll.return_value = 0
        with patch('resource_runner.subprocess.run') as systemctl:
            stop(process, 'scope-with-descendants.scope')
        self.assertEqual(systemctl.call_args.args[0],
                         ['systemctl', '--user', 'stop', 'scope-with-descendants.scope'])

    def test_final_accounting_records_actual_output_under_the_same_cap(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            policy = capped_policy(root)
            directory = root / 'control'
            directory.mkdir()
            process = Mock(pid=123, returncode=0)
            process.poll.return_value = 0
            record = {'token': 'final-success', 'budget_bytes': 16384, 'used_bytes': 4096}
            with patch('resource_runner.subprocess.Popen', return_value=process), \
                 patch('resource_runner.scope_active', return_value=False), \
                 patch('resource_runner.usage', return_value=8192) as final_usage:
                self.assertEqual(supervise(['unused'], root, {}, policy, {'outputs': ['obj']},
                                           record, directory / 'slot-0.json', ()), 0)
                final_usage.assert_called_once_with(root, {'outputs': ['obj']}, policy, 16384)
            receipt = json.loads((root / 'obj/parallel/resources/final-success.json').read_text())
            self.assertEqual(receipt['budget_bytes'], 16384)
            self.assertEqual(receipt['supervised_budget_bytes'], 16384)
            self.assertEqual(receipt['used_bytes'], 8192)
            self.assertEqual(receipt['returncode'], 0)
            self.assertIsNone(receipt['error'])

    def test_retained_receipt_itself_cannot_make_a_tiny_cap_succeed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            process = Mock(pid=123, returncode=0)
            process.poll.return_value = 0
            record = {'token': 'receipt-cap', 'budget_bytes': 1}
            with patch('resource_runner.subprocess.Popen', return_value=process), \
                 patch('resource_runner.scope_active', return_value=False), \
                 patch('resource_policy.free_space'), \
                 patch('resource_runner.stop') as terminate, patch('sys.stderr', io.StringIO()):
                result = supervise(['unused'], root, {}, capped_policy(root), {'outputs': ['obj']},
                                   record, directory / 'slot-0.json', ())
                terminate.assert_called_once()
            self.assertEqual(result, 1)
            self.assertGreater(record['used_bytes'], 1)
            self.assertIn('output cap 1 bytes', record['error'])

    def test_receipt_block_growth_gets_one_bounded_recheck(self):
        for blocks, success in [([1, 2, 2, 2], True), ([1, 2, 2, 3], False)]:
            with self.subTest(blocks=blocks), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                directory = root / 'control'
                directory.mkdir()
                process = Mock(pid=123, returncode=0)
                process.poll.return_value = 0
                record = {'token': 'receipt-growth', 'budget_bytes': 16384}
                receipt = Mock()
                receipt.stat.side_effect = [Mock(st_blocks=value) for value in blocks]
                def write(*args, **kwargs):
                    finished(*args, **kwargs)
                    return receipt
                with patch('resource_runner.subprocess.Popen', return_value=process), \
                     patch('resource_runner.scope_active', return_value=False), \
                     patch('resource_runner.usage', side_effect=[4096, 8192]) as measure, \
                     patch('resource_runner.finished', side_effect=write), \
                     patch('resource_runner.stop') as terminate, patch('sys.stderr', io.StringIO()):
                    result = supervise(['unused'], root, {}, capped_policy(root), {'outputs': ['obj']},
                                       record, directory / 'slot-0.json', ())
                self.assertEqual(measure.call_count, 2)
                self.assertEqual(result, 0 if success else 1)
                self.assertEqual(terminate.call_count, 0 if success else 1)

    def test_nested_caps_inherit_exact_supervised_budget_and_reject_changes(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            root = Path('/work')
            policy = capped_policy(root)
            path = directory / 'slot-0.json'
            record = {'pid': os.getppid(), 'process_start': process_identity(os.getppid())[1],
                      'token': 'nonce', 'root': str(root), 'kind': 'ordinary', 'slot': 0,
                      'schema': 2, 'policy_identity': policy_identity(policy),
                      'controller': str(directory), 'state': 'running',
                      'budget_bytes': 8 * GIB, 'supervised_budget_bytes': 8 * GIB}
            atomic_json(path, record)
            env = {'RUSTGM_RESOURCE_LEASE': str(path), 'RUSTGM_RESOURCE_TOKEN': 'nonce'}
            actual = inherited(directory, root, env, 'ordinary', 3, policy=policy)
            self.assertEqual(inherited_budget(actual, policy), 8 * GIB)
            self.assertEqual(inherited_budget(actual, policy, 8 * GIB), 8 * GIB)
            for cap in [4 * GIB, 16 * GIB, 50 * GIB]:
                with self.subTest(cap=cap), self.assertRaisesRegex(ValueError, 'outer cap'):
                    inherited_budget(actual, policy, cap)
            for changes in [{'supervised_budget_bytes': None}, {'state': 'finished'}]:
                with self.subTest(changes=changes), self.assertRaisesRegex(ValueError, 'supervisor'):
                    inherited_budget({**actual, **changes}, policy)
            for changes in [{'process_start': -1}, {'controller': '/other'},
                            {'policy_identity': 'foreign'}, {'slot': 1}]:
                atomic_json(path, {**record, **changes})
                with self.subTest(changes=changes), self.assertRaises(ValueError):
                    inherited(directory, root, env, 'ordinary', 3, policy=policy)
            atomic_json(path, record)
            with patch('resource_runner.discover', return_value=(directory, policy)), \
                 patch('resource_runner.subprocess.run') as launch, \
                 patch('resource_runner.lease') as acquire:
                launch.return_value.returncode = 0
                self.assertEqual(run(root, ['unused'], env), 0)
                self.assertEqual(run(root, ['unused'], env, max_output_gib=8), 0)
                with self.assertRaises(ValueError):
                    run(root, ['unused'], env, max_output_gib=4)
                self.assertEqual(launch.call_count, 2)
                acquire.assert_not_called()

    def test_storage_forwards_an_explicit_cap_and_leaves_inheritance_to_controller(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for arguments, maximum in [([], None), (['--max-output-gib', '8'], 8.0)]:
                with self.subTest(maximum=maximum), patch.object(storage, 'ROOT', root), \
                     patch('sys.argv', ['storage.py', 'run', '--no-prune', *arguments, '--', 'unused']), \
                     patch.object(storage, 'check_space'), \
                     patch.object(storage, 'run_guarded', return_value=0) as launch:
                    self.assertEqual(storage.main(), 0)
                    self.assertEqual(launch.call_args.kwargs, {'max_output_gib': maximum})
                    self.assertEqual(launch.call_args.args[:2], (root, ['unused']))

    def test_default_schema_one_run_keeps_full_allowance_and_thread_limits(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            policy = {**DEFAULTS, 'worktrees': {str(root): {'name': 'test', 'outputs': ['obj']}}}
            directory = root / 'control'
            with patch('resource_runner.discover', return_value=(directory, policy)), \
                 patch('resource_runner.usage', return_value=0), \
                 patch('resource_runner.free_space') as space, \
                 patch('resource_runner.supervise', return_value=0) as launch:
                self.assertEqual(run(root, ['unused'], {}, wait=0), 0)
                space.assert_called_once_with(root, policy, 50 * GIB)
                environment = launch.call_args.args[2]
                self.assertEqual(environment['CARGO_BUILD_JOBS'], '2')
                self.assertEqual(environment['RUST_TEST_THREADS'], '2')
                self.assertEqual(environment['RUSTGM_TEST_JOBS'], '2')
                self.assertEqual(launch.call_args.args[5]['budget_bytes'], 50 * GIB)

    def test_schema_activation_is_idle_only_and_keeps_all_other_policy_fields(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            path = root / 'rustgm-resources.json'
            atomic_json(path, DEFAULTS)
            with patch('resources.discover', return_value=(directory, DEFAULTS)):
                with lease(directory, Path('/active'), DEFAULTS, 'ordinary', wait=0):
                    with self.assertRaisesRegex(ValueError, 'finish guarded'):
                        set_schema(root, 2)
                # An acquiring slot with no record also prevents activation.
                with (directory / 'slot-2.lock').open('a+') as handle:
                    fcntl.flock(handle, fcntl.LOCK_EX)
                    with self.assertRaisesRegex(ValueError, 'finish guarded'):
                        set_schema(root, 2)
                self.assertEqual(json.loads(path.read_text()), DEFAULTS)
                set_schema(root, 2)
            self.assertEqual(json.loads(path.read_text()), {**DEFAULTS, 'schema': 2})

    def test_schema_rewrite_keeps_exclusive_gate_and_rechecks_acquiring_slots(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            path = root / 'rustgm-resources.json'
            atomic_json(path, DEFAULTS)
            original = atomic_json
            def checked_write(target, value):
                if target == path:
                    with (directory / 'gate.lock').open('a+') as handle:
                        self.assertFalse(try_lock(handle, fcntl.LOCK_SH))
                original(target, value)
            with patch('resources.discover', return_value=(directory, DEFAULTS)), \
                 patch('resources.atomic_json', side_effect=checked_write):
                set_schema(root, 2)
            self.assertEqual(json.loads(path.read_text())['schema'], 2)
            atomic_json(path, DEFAULTS)
            with patch('resources.discover', return_value=(directory, DEFAULTS)), \
                 patch('resources.active_records', side_effect=[[], [{'slot': 1, 'state': 'acquiring'}]]):
                with self.assertRaisesRegex(ValueError, 'acquiring leases'):
                    set_schema(root, 2)
            self.assertEqual(json.loads(path.read_text()), DEFAULTS)

    def test_mixed_caps_preserve_three_slots_worktree_lock_and_exclusive_gate(self):
        policy = {**DEFAULTS, 'schema': 2, 'jobs': 3}
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            directory = Path(folder)
            for root, cap in [('/one', 8), ('/two', 16), ('/three', 50)]:
                stack.enter_context(lease(directory, Path(root), policy,
                                          'ordinary', wait=0, budget=cap * GIB))
            self.assertEqual([peer_budget(row, policy) for row in active_records(directory, 3)],
                             [8 * GIB, 16 * GIB, 50 * GIB])
            for root, kind in [('/four', 'ordinary'), ('/one', 'ordinary'), ('/only', 'exclusive')]:
                with self.subTest(root=root), self.assertRaisesRegex(ValueError, 'timed out'):
                    with lease(directory, Path(root), policy, kind, wait=0, budget=GIB):
                        self.fail('cap changed concurrency')

    def test_snapshot_cannot_read_stale_small_cap_between_claim_and_publication(self):
        context = multiprocessing.get_context('spawn')
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            atomic_json(directory / 'slot-0.json',
                        {'slot': 0, 'schema': 2, 'state': 'running', 'budget_bytes': 1})
            ready, publish, release, started = [context.Event() for _ in range(4)]
            results = context.Queue()
            publisher = context.Process(target=paused_publication,
                                        args=(folder, ready, publish, release))
            observer = context.Process(target=take_snapshot, args=(folder, started, results))
            try:
                publisher.start()
                self.assertTrue(ready.wait(5))
                observer.start()
                self.assertTrue(started.wait(5))
                with self.assertRaises(queue.Empty):
                    results.get(timeout=.1)
                publish.set()
                rows = results.get(timeout=5)
                self.assertEqual(len(rows), 1)
                self.assertEqual(rows[0]['budget_bytes'], 8 * GIB)
                release.set()
                for process in [publisher, observer]:
                    process.join(5)
                    self.assertEqual(process.exitcode, 0)
            finally:
                publish.set()
                release.set()
                for process in [publisher, observer]:
                    if process.pid is not None:
                        if process.is_alive():
                            process.terminate()
                        process.join(5)

    def test_full_slot_wait_does_not_hold_admission_lock(self):
        policy = {**DEFAULTS, 'schema': 2}
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            directory = Path(folder)
            for root in ['/one', '/two']:
                stack.enter_context(lease(directory, Path(root), policy, 'ordinary', wait=0))
            def observe_before_sleep(_delay):
                with (directory / 'admission.lock').open('a+') as handle:
                    self.assertTrue(try_lock(handle, fcntl.LOCK_EX))
                raise ValueError('test wait ended')
            with patch('resource_locks.time.sleep', side_effect=observe_before_sleep):
                with self.assertRaisesRegex(ValueError, 'test wait ended'):
                    with lease(directory, Path('/waiter'), policy, 'ordinary', wait=1):
                        self.fail('no slot available')
