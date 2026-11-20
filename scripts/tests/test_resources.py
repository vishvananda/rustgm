"""Shared scheduling and disk failures must protect independent worktrees."""
from contextlib import ExitStack
import fcntl
import io
import json
import multiprocessing
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from resource_locks import active_records, holders, inherited, lease, try_lock
from resource_policy import DEFAULTS, GIB, UsageScan, atomic_json, output_roots, validate, width
from resource_runner import run, sample_scope, scope_cgroup, scope_command, scope_notes, supervise
from resources import set_jobs


def contender(directory, root, kind, queue, release):
    with lease(Path(directory), Path(root), DEFAULTS, kind, wait=8):
        queue.put(root)
        release.wait(8)


class ResourceTests(unittest.TestCase):
    def test_reviewed_policy_and_output_boundaries(self):
        validate(DEFAULTS)
        validate({**DEFAULTS, "jobs": 3})
        for key, value in [("jobs", 4), ("reserve_gib", 149), ("worker_gib", 51),
                           ("cpu_percent", 801), ("memory_gib", 13), ("tasks", 257)]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate({**DEFAULTS, key: value})
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'obj').mkdir()
            (root / 'obj/link').symlink_to(root)
            for path in ['outside', '/tmp', 'obj/../escape', 'obj/link/cache']:
                with self.subTest(path=path), self.assertRaises(ValueError):
                    output_roots(root, {"outputs": [path]})
            self.assertEqual(output_roots(root, {"outputs": ['obj/a/b', 'obj/a']}),
                             [root / 'obj/a'])

    def test_usage_includes_caches_and_pins_without_following_links(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ['rust/cache', 'pinned/candidate', 'evidence/log']:
                path = root / name
                path.parent.mkdir(exist_ok=True)
                path.write_bytes(b'x' * 8192)
            (root / 'pinned/.keep-artifacts').touch()
            (root / 'cycle').symlink_to(root)
            scan = UsageScan([root])
            steps = 0
            while not scan.done:
                scan.step(1)
                steps += 1
                self.assertLess(steps, 30)
            self.assertGreaterEqual(scan.total, 3 * 8192)
            self.assertLess(scan.total, 100_000)

    def test_two_slots_and_one_lease_per_worktree(self):
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            directory = Path(folder)
            stack.enter_context(lease(directory, Path('/first'), DEFAULTS, 'ordinary', wait=0))
            stack.enter_context(lease(directory, Path('/second'), DEFAULTS, 'ordinary', wait=0))
            self.assertEqual(len(active_records(directory, 2)), 2)
            for root in ['/third', '/first']:
                with self.subTest(root=root), self.assertRaisesRegex(ValueError, 'timed out'):
                    with lease(directory, Path(root), DEFAULTS, 'ordinary', wait=0):
                        self.fail('exceeded concurrency')
        # ExitStack closes all kernel leases even when a command fails.

    def test_queued_job_names_the_slot_holders(self):
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            directory = Path(folder)
            for root in ['/first', '/second']:
                stack.enter_context(lease(directory, Path(root), DEFAULTS, 'ordinary', wait=0))
            with patch('resource_locks.time.sleep', side_effect=ValueError('test wait ended')), \
                 patch('sys.stdout', io.StringIO()) as output:
                with self.assertRaisesRegex(ValueError, 'test wait ended'):
                    with lease(directory, Path('/third'), DEFAULTS, 'ordinary', wait=5):
                        self.fail('no slot available')
            self.assertRegex(output.getvalue(),
                             r'waiting for a heavy-job slot held by first for \d+s, second for \d+s')
        self.assertEqual(holders([{'slot': 0, 'state': 'acquiring'}]), 'an acquiring job')
        self.assertEqual(holders([]), 'a finishing job')

    def test_three_slots_keep_worktree_and_exclusive_limits(self):
        policy = {**DEFAULTS, 'jobs': 3}
        with tempfile.TemporaryDirectory() as folder, ExitStack() as stack:
            directory = Path(folder)
            for root in ['/first', '/second', '/third']:
                stack.enter_context(lease(directory, Path(root), policy, 'ordinary', wait=0))
            self.assertEqual(len(active_records(directory, 3)), 3)
            for root, kind in [('/fourth', 'ordinary'), ('/first', 'ordinary'),
                               ('/exclusive', 'exclusive')]:
                with self.subTest(root=root), self.assertRaisesRegex(ValueError, 'timed out'):
                    with lease(directory, Path(root), policy, kind, wait=0):
                        self.fail('exceeded concurrency or exclusivity')

    def test_slot_change_requires_idle_controller_and_preserves_other_limits(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            path = root / 'rustgm-resources.json'
            atomic_json(path, DEFAULTS)
            with patch('resources.discover', return_value=(directory, DEFAULTS)):
                with lease(directory, Path('/worker'), DEFAULTS, 'ordinary', wait=0):
                    with self.assertRaisesRegex(ValueError, 'finish guarded'):
                        set_jobs(root, 3)
                    self.assertEqual(json.loads(path.read_text()), DEFAULTS)
                set_jobs(root, 3)
            self.assertEqual(json.loads(path.read_text()), {**DEFAULTS, 'jobs': 3})

    def test_waiting_job_cannot_launch_with_stale_policy(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            policy = {**DEFAULTS, 'worktrees': {str(root): {'name': 'test', 'outputs': ['obj']}}}
            with patch('resource_runner.discover', side_effect=[
                (directory, policy), (directory, {**policy, 'jobs': 3})
            ]), patch('resource_runner.supervise') as launch:
                with self.assertRaisesRegex(ValueError, 'policy changed'):
                    run(root, ['not-launched'], {})
                launch.assert_not_called()
            self.assertEqual(active_records(directory, 3), [])

    def test_exclusive_waiter_blocks_new_ordinary_admission(self):
        context = multiprocessing.get_context('spawn')
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            queue = context.Queue()
            release = context.Event()
            ordinary_release = context.Event()
            first = lease(directory, Path('/first'), DEFAULTS, 'ordinary', wait=0)
            first.__enter__()
            exclusive = context.Process(target=contender,
                                        args=(folder, '/exclusive', 'exclusive', queue, release))
            ordinary = context.Process(target=contender,
                                       args=(folder, '/ordinary', 'ordinary', queue, ordinary_release))
            try:
                exclusive.start()
                deadline = time.monotonic() + 5
                while True:
                    with (directory / 'turnstile.lock').open('a+') as handle:
                        blocked = not try_lock(handle, fcntl.LOCK_SH)
                    if blocked:
                        break
                    self.assertLess(time.monotonic(), deadline)
                    time.sleep(.01)
                ordinary.start()
                first.__exit__(None, None, None)
                first = None
                self.assertEqual(queue.get(timeout=5), '/exclusive')
                self.assertTrue(queue.empty())
                release.set()
                self.assertEqual(queue.get(timeout=5), '/ordinary')
                ordinary_release.set()
                for process in [exclusive, ordinary]:
                    process.join(5)
                    self.assertEqual(process.exitcode, 0)
                self.assertEqual(active_records(directory, 2), [])
            finally:
                if first is not None:
                    first.__exit__(None, None, None)
                release.set()
                ordinary_release.set()
                for process in [exclusive, ordinary]:
                    if process.pid is not None:
                        if process.is_alive():
                            process.terminate()
                        process.join(5)

    def test_inherited_lease_requires_ancestor_and_same_worktree(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            path = directory / 'slot-0.json'
            record = {'pid': os.getppid(), 'token': 'nonce', 'root': '/work', 'kind': 'ordinary'}
            atomic_json(path, record)
            env = {'RUSTGM_RESOURCE_LEASE': str(path), 'RUSTGM_RESOURCE_TOKEN': 'nonce'}
            self.assertTrue(inherited(directory, Path('/work'), env, 'ordinary'))
            third = directory / 'slot-2.json'
            atomic_json(third, record)
            third_env = {**env, 'RUSTGM_RESOURCE_LEASE': str(third)}
            self.assertTrue(inherited(directory, Path('/work'), third_env, 'ordinary', 3))
            with self.assertRaisesRegex(ValueError, 'invalid inherited'):
                inherited(directory, Path('/work'), third_env, 'ordinary', 2)
            for root, kind in [('/other', 'ordinary'), ('/work', 'exclusive')]:
                with self.assertRaises(ValueError):
                    inherited(directory, Path(root), env, kind)
            record['pid'] = 999_999_999
            atomic_json(path, record)
            with self.assertRaisesRegex(ValueError, 'ancestor'):
                inherited(directory, Path('/work'), env, 'ordinary')

    def test_systemd_properties_and_literal_argument_preservation(self):
        command = ['python3', '-c', 'print("$(not shell)")']
        actual = scope_command(command, 'rustgm-test.scope', DEFAULTS)
        self.assertEqual(actual[-3:], command)
        for value in ['CPUQuota=800%', 'MemoryMax=12G', 'MemorySwapMax=0', 'TasksMax=256']:
            self.assertIn(value, actual)

    def test_width_follows_the_cpu_grant_up_to_the_reviewed_limit(self):
        self.assertEqual(width(DEFAULTS), '8')
        self.assertEqual(width(DEFAULTS, 2), '2')
        self.assertEqual(width({**DEFAULTS, 'cpu_percent': 100}, 2), '1')
        self.assertEqual(width({**DEFAULTS, 'cpu_percent': 300}), '3')
        self.assertEqual(width({**DEFAULTS, 'cpu_percent': 50}), '1')
        for cpus, expected in [(88, '8'), (4, '4'), (None, '1')]:
            with self.subTest(cpus=cpus), patch('resource_policy.os.cpu_count', return_value=cpus):
                self.assertEqual(width(), expected)

    def test_receipt_keeps_scope_memory_and_task_high_water_marks(self):
        with tempfile.TemporaryDirectory() as folder:
            scope = Path(folder)
            record = {}
            sample_scope(scope, record)
            self.assertEqual(record, {})
            for peak, anonymous, kills, tasks in [(900, 500, 0, 40), (1200, 300, 1, 90), (1200, 700, 1, 90)]:
                (scope / 'memory.peak').write_text(f'{peak}\n')
                (scope / 'memory.stat').write_text(f'anon {anonymous}\nfile 400\n')
                (scope / 'memory.events').write_text(f'low 0\nmax 3\noom 1\noom_kill {kills}\n')
                (scope / 'pids.peak').write_text(f'{tasks}\n')
                (scope / 'pids.events').write_text('max 2\n')
                sample_scope(scope, record)
            self.assertEqual(record, {'memory_peak_bytes': 1200, 'memory_anon_peak_bytes': 700,
                                      'memory_oom_kills': 1, 'tasks_peak': 90, 'tasks_refused': 2})
        # A bound cap is named; ordinary use stays silent.
        self.assertEqual(scope_notes({'memory_anon_peak_bytes': 8 * GIB, 'tasks_peak': 40}, DEFAULTS), [])
        self.assertEqual(scope_notes({'memory_anon_peak_bytes': 11 * GIB}, DEFAULTS),
                         ['anonymous memory peaked at 11.0 of the 12 GiB memory cap'])
        self.assertEqual(scope_notes({'memory_oom_kills': 2, 'memory_anon_peak_bytes': 12 * GIB,
                                      'tasks_refused': 3}, DEFAULTS),
                         ['2 process(es) were OOM-killed at the 12 GiB memory cap',
                          '3 task(s) were refused at the 256-task cap'])
        # A process is sampled only once it has entered its own scope.
        self.assertIsNone(scope_cgroup(os.getpid(), 'rustgm-job-absent.scope'))
        self.assertIsNone(scope_cgroup(999_999_999, 'rustgm-job-absent.scope'))

    def test_low_space_refuses_launch_and_releases_slot(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            policy = {**DEFAULTS, 'worktrees': {str(root): {'name': 'test', 'outputs': ['obj']}}}
            with patch('resource_runner.discover', return_value=(directory, policy)), \
                 patch('resource_runner.usage', return_value=0), \
                 patch('resource_runner.free_space', side_effect=ValueError('insufficient reserve')), \
                 patch('resource_runner.supervise') as launch:
                with self.assertRaisesRegex(ValueError, 'reserve'):
                    run(root, ['not-launched'], {})
                launch.assert_not_called()
            self.assertEqual(active_records(directory, 2), [])

    def test_disk_exhaustion_stops_running_process_and_retains_receipt(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'control'
            directory.mkdir()
            path = directory / 'slot-0.json'
            record = {'pid': os.getpid(), 'token': 'test-disk-stop', 'slot': 0,
                      'root': str(root), 'kind': 'ordinary'}
            process_ids = []
            real_popen = subprocess.Popen
            def launch(*args, **kwargs):
                process = real_popen(*args, **kwargs)
                process_ids.append(process.pid)
                return process
            def terminate(process, _unit):
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
            with patch('resource_runner.scope_command', side_effect=lambda command, *_: command), \
                 patch('resource_runner.subprocess.Popen', side_effect=launch), \
                 patch('resource_runner.free_space', side_effect=ValueError('reserve crossed')), \
                 patch('resource_runner.stop', side_effect=terminate), \
                 patch('sys.stderr', io.StringIO()):
                result = supervise([sys.executable, '-c', 'import time; time.sleep(60)'],
                                   root, dict(os.environ), DEFAULTS, {'outputs': ['obj']},
                                   record, path, ())
            self.assertEqual(result, 1)
            with self.assertRaises(ProcessLookupError):
                os.kill(process_ids[0], 0)
            receipt = json.loads((root / 'obj/parallel/resources/test-disk-stop.json').read_text())
            self.assertEqual(receipt['error'], 'reserve crossed')
            self.assertNotIn('token', receipt)

    def test_worker_quota_stops_a_live_writer(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'obj').mkdir()
            (root / 'obj/large').write_bytes(b'x' * 8192)
            directory = root / 'control'
            directory.mkdir()
            record = {'pid': os.getpid(), 'token': 'test-quota', 'slot': 0,
                      'root': str(root), 'kind': 'ordinary', 'budget_bytes': 4096}
            def terminate(process, _unit):
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
            with patch('resource_runner.scope_command', side_effect=lambda command, *_: command), \
                 patch('resource_runner.free_space', return_value=1000 * GIB), \
                 patch('resource_runner.stop', side_effect=terminate), \
                 patch('sys.stderr', io.StringIO()):
                result = supervise([sys.executable, '-c', 'import time; time.sleep(60)'],
                                   root, dict(os.environ), {**DEFAULTS, 'schema': 2},
                                   {'outputs': ['obj']}, record, directory / 'slot-0.json', ())
            self.assertEqual(result, 1)
            self.assertIn('output cap 4096 bytes', record['error'])
            self.assertGreaterEqual(record['used_bytes'], 8192)
