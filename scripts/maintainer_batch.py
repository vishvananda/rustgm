"""Build focused targets once, then run exact inventories without reentering Cargo."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

from maintainer_suites import selected_tests
from storage import cleanup_roots


def selection(suites, names, library_cases):
    """Keep suite boundaries and private test identities explicit."""
    groups = {}
    for name in names:
        suite, separator, case = name.partition("::")
        if suite not in suites or (separator and not case):
            raise ValueError(f"unknown or empty suite selection: {name}")
        groups.setdefault(suites[suite], []).append((suite, case if separator else None))
    if library_cases:
        if any(not name or "::" not in name for name in library_cases):
            raise ValueError("private tests require their complete module path")
        groups["lib"] = list(library_cases)
    if not groups or len(set(names)) != len(names) or len(set(library_cases)) != len(library_cases):
        raise ValueError("batch requires nonempty, unique selections")
    return groups


def build_command(groups, args):
    command = ["cargo", "test", "--manifest-path", "dev/Cargo.toml", "--locked",
               "--no-run", "--message-format=json"]
    if args.release:
        command.append("--release")
    if args.all_features:
        command.append("--all-features")
    for group in groups:
        command += ["--lib"] if group == "lib" else ["--test", group]
    return command


def artifacts(output, groups):
    result = {}
    for line in output.splitlines():
        row = json.loads(line)
        if (row.get("reason") != "compiler-artifact" or not row.get("executable")
                or not row.get("profile", {}).get("test")):
            continue
        target = row["target"]
        group = "lib" if target["kind"] == ["lib"] else target["name"]
        if group not in groups:
            continue
        if group in result:
            raise ValueError(f"duplicate Cargo test executable: {group}")
        executable = Path(row["executable"])
        if not executable.is_absolute() or not executable.is_file():
            raise ValueError(f"missing Cargo test executable: {group}")
        result[group] = executable
    if set(result) != set(groups):
        raise ValueError("Cargo did not produce every requested test target")
    return result


def inventory(listing, group, selectors):
    selected = []
    for selector in selectors:
        if group == "lib":
            module, _, case = selector.rpartition("::")
            selected.extend(selected_tests(listing, module, case))
        else:
            selected.extend(selected_tests(listing, *selector))
    if len(set(selected)) != len(selected):
        raise ValueError("overlapping selections would execute a test twice")
    return selected


def run_batch(suites, args, *, root, runner=subprocess.run):
    groups = selection(suites, args.suites, args.lib_case)
    folder = root / (args.evidence_root or f"obj/qualification/focused/{time.time_ns()}")
    cleanup_roots(root, [folder])
    folder.mkdir(parents=True, exist_ok=False)
    receipt = {"profile": "release" if args.release else "debug", "groups": groups,
               "ignored": args.ignored, "state": "running", "steps": []}

    def save():
        (folder / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")

    def run(name, command, cwd):
        print(f"START {name}", flush=True)
        started = time.monotonic()
        result = runner([str(arg) for arg in command], cwd=cwd, capture_output=True, text=True)
        hashes = {}
        for stream in ("stdout", "stderr"):
            output = getattr(result, stream).encode()
            (folder / f"{name}.{stream}").write_bytes(output)
            hashes[stream] = hashlib.sha256(output).hexdigest()
        receipt["steps"].append({"name": name, "command": list(map(str, command)),
                                 "cwd": str(cwd), "status": result.returncode,
                                 "seconds": time.monotonic() - started,
                                 "sha256": hashes})
        save()
        print(f"END {name}: status {result.returncode}", flush=True)
        if result.returncode:
            print((result.stdout + result.stderr)[-12000:], flush=True)
        return result

    save()
    try:
        built = run("build", build_command(groups, args), root)
        if built.returncode:
            return built.returncode
        binaries = artifacts(built.stdout, groups)
        selected = {}
        flags = ["--ignored"] if args.ignored else []
        # Enumerate all targets before any execution; one missing case fails the batch.
        for group, binary in binaries.items():
            listed = run(group + "-inventory", [binary, *flags, "--list"], root / "dev")
            if listed.returncode:
                return listed.returncode
            listing = listed.stdout
            if not args.ignored:
                ignored = run(group + "-ignored", [binary, "--ignored", "--list"], root / "dev")
                if ignored.returncode:
                    return ignored.returncode
                ignored_rows = set(ignored.stdout.splitlines())
                listing = "\n".join(row for row in listing.splitlines() if row not in ignored_rows)
            selected[group] = inventory(listing, group, groups[group])
        receipt["selected"] = selected
        receipt["binaries"] = {group: {"path": str(binary),
                                      "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
                               for group, binary in binaries.items()}
        save()
        failed = 0
        for group, binary in binaries.items():
            if args.list:
                print("\n".join(selected[group]))
                continue
            result = run(group, [binary, *flags, "--exact", *selected[group]], root / "dev")
            counts = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;",
                                result.stdout)
            if (len(counts) != 1 or sum(map(int, counts[0][:2])) != len(selected[group])
                    or counts[0][2] != "0"):
                raise ValueError(f"{group}: executed inventory differs from selected tests")
            failed |= result.returncode != 0
        receipt["state"] = "failed" if failed else "listed" if args.list else "passed"
        return int(failed)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        receipt["error"] = str(error)
        raise
    finally:
        if receipt["state"] == "running":
            receipt["state"] = "failed"
        save()
        print(f"Focused evidence: {folder}", flush=True)
