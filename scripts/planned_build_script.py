"""Bind authenticated script units to the explicit host task boundary.

This completes selected dependency-free script pairs. Host dependency scheduling,
DEP links metadata and target compilation remain separate graph consumers.
"""
from dataclasses import asdict
import json
from pathlib import Path
import subprocess
import time
import tomllib

from build_units import Execution, index
from build_environment import package_environment
import host_build_script
import host_profile
import manifest_lints


def pair(plan, selected, package, context):
    unit = plan.units[index(selected, len(plan.units))]
    if unit.execution is not Execution.HOST_RUN or unit.pkg_id != package.cargo_id:
        raise ValueError('selected unit is not this package host execution')
    if context.target != (unit.platform or context.host):
        raise ValueError('build-script target context differs from planned platform')
    if len(unit.dependencies) != 1:
        raise ValueError('build-script DEP metadata scheduling is not implemented')
    compiled = plan.units[unit.dependencies[0].index]
    if (compiled.execution is not Execution.HOST_COMPILE or compiled.pkg_id != unit.pkg_id
            or compiled.target != unit.target or compiled.features != unit.features):
        raise ValueError('build script lacks its exact paired host compilation')
    if compiled.dependencies:
        raise ValueError('build-script host dependency scheduling is not implemented')
    if compiled.target.kind != ('custom-build',) or compiled.target.crate_types != ('bin',):
        raise ValueError('host script compilation requires a custom-build executable')
    if Path(compiled.target.src_path) not in {package.root / name for name, _, _ in package.files}:
        raise ValueError('build-script source does not belong to its package snapshot')
    host_profile.decode(compiled.profile_json)
    host_profile.decode(unit.profile_json)
    if any(',' in feature for feature in unit.features):
        raise ValueError('build-script feature cannot be encoded in CARGO_CFG_FEATURE')
    return unit, compiled


def execute(plan, selected, package, context, folder, *, lint_cap=None):
    manifest_lints.cap_arguments(lint_cap)
    package.verify()
    context.validate()
    unit, compiled = pair(plan, selected, package, context)
    folder = folder.absolute()
    if folder.exists() or folder.is_symlink():
        raise ValueError('planned host task output already exists')
    folder.mkdir(parents=True)
    wrapper = Path(__file__).with_name('cargo_query.py').absolute()
    implementation = {name: host_build_script.digest(wrapper.with_name(name)) for name in (
        'planned_build_script.py', 'host_build_script.py', 'host_process.py', 'build_script_output.py',
        'build_environment.py', 'host_profile.py', 'cargo_query.py', 'build_units.py',
        'package_snapshot.py', 'local_package.py', 'manifest_lints.py')}
    document = tomllib.loads((package.root / 'Cargo.toml').read_text())
    common = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8',
              **package_environment(package, document)}
    queries = folder / 'queries.jsonl'
    runtime = {**common, **context.cfg_environment(), **host_profile.environment(unit.profile_json),
               'HOST': context.host, 'TARGET': unit.platform or context.host, 'NUM_JOBS': '2',
               'CARGO_ENCODED_RUSTFLAGS': '', 'CARGO_CFG_FEATURE': ','.join(unit.features),
               'RUSTC': str(wrapper), 'RUSTGM_PLAN_QUERY_COMPILER': str(context.compiler),
               'RUSTGM_PLAN_QUERY_SHA256': context.compiler_sha256,
               'RUSTGM_PLAN_TARGET': unit.platform or context.host,
               'RUSTGM_PLAN_QUERY_LOG': str(queries)}
    job = host_build_script.Job(package.root, Path(compiled.target.src_path), compiled.target.edition,
                               context.compiler, context.compiler_sha256,
                               {name: sha for name, sha, _ in package.files},
                               {**common, 'CARGO_CRATE_NAME': 'build_script_build'}, runtime,
                               compiled.features, profile_json=compiled.profile_json,
                               lints=document.get('lints'), lint_cap=lint_cap)
    receipt = {'state': 'running', 'package': package.identity, 'selected': selected,
               'run': asdict(unit), 'compile': asdict(compiled), 'host': context.host,
               'target': unit.platform or context.host, 'cfg': context.cfg,
               'configuration_inputs': [(str(path), sha) for path, sha in context.inputs],
               'implementation_sha256': implementation}
    receipt['lint_cap'] = lint_cap
    started = time.monotonic()

    def save():
        (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

    save()
    try:
        result = host_build_script.execute(job, folder / 'host')
        # A script may swallow a rejected compiler probe and return success.
        # Such a probe must never manufacture an apparent missing capability.
        observed = [] if not queries.exists() else [json.loads(row) for row in queries.read_text().splitlines()]
        if any(row.get('state') != 'query' or row.get('status') != 0 for row in observed):
            raise ValueError('build script attempted an unsupported or failed compiler query')
        package.verify()
        context.validate()
        if any(host_build_script.digest(wrapper.with_name(name)) != sha for name, sha in implementation.items()):
            raise ValueError('host task implementation changed during execution')
        host_receipt = json.loads((folder / 'host/receipt.json').read_text())
        receipt.update(state='complete', host_receipt_sha256=host_build_script.digest(folder / 'host/receipt.json'),
                       generated=host_receipt['generated'], directives=[asdict(row) for row in result.directives],
                       compiler_queries=len(observed))
        return result
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        receipt['seconds'] = time.monotonic() - started
        if queries.exists():
            receipt['queries_sha256'] = host_build_script.digest(queries)
        save()
