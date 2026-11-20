#!/usr/bin/env python3
"""Grow a bring-up skip list until one crate compiles in the unqualified profile.

Each iteration runs the compiler with the current skip list. Every failure that
the compiler attributes to a function (its ``bringup-blocker`` lines) appends
that function to the list; the next iteration replaces it by a trapping stub.
The loop stops when the crate compiles, when no failure names a new function
(the remaining ones have no owning function, or a listed function failed again)
or at the iteration limit.

The skip list is the census of the artifact pipeline: one row per function that
does not yet compile, with the message that stopped it and the stage that
recorded it (``stopped`` when no stage recorded it). ``waived.tsv`` names the
crate-level publication fences that the last compilation passed over, and
``unproven.tsv`` the functions that kept their bodies although a borrow,
lifetime or const-cleanup proof of them failed. None of these is a
qualification. Run this under the resource and storage guards.
"""
import argparse
import collections
import json
import pathlib
import re
import subprocess
import sys
import time


def rows(path):
    """Return the skip list's (path, offset, name, cause) rows."""
    found = []
    if not path.exists():
        return found
    for row in path.read_text().splitlines():
        if not row or row.startswith("#"):
            continue
        fields = row.split("\t") + ["", ""]
        found.append((fields[0], int(fields[1]), fields[2], fields[3]))
    return found


def waived(stderr):
    """Return the publication fences that the compilation passed over."""
    prefix = "bringup-waived\t"
    return [line[len(prefix):] for line in stderr.splitlines() if line.startswith(prefix)]


def unproven(stderr):
    """Return the functions that kept their bodies without a soundness proof."""
    prefix = "bringup-unproven\t"
    return [line[len(prefix):] for line in stderr.splitlines() if line.startswith(prefix)]


def diagnose(stderr):
    """Return attributed blockers, unattributed messages and the last error."""
    blockers, unattributed, message = [], [], None
    for line in stderr.splitlines():
        if line.startswith("bringup-blocker\t"):
            fields = line.split("\t")
            if len(fields) >= 4 and fields[2].isdigit():
                blockers.append((fields[1], int(fields[2]), fields[3],
                                 fields[4] if len(fields) > 4 else None,
                                 fields[5] if len(fields) > 5 else None))
        elif line.startswith("bringup-unattributed\t"):
            unattributed.append(line.split("\t", 1)[1])
        elif line.startswith("ERROR: "):
            message = line[len("ERROR: "):]
    return blockers, unattributed, message


def cause(message):
    """Drop the leading source position so equal causes group together."""
    return re.sub(r"^(file \d+: \d+: |input: )", "", message or "no error line")


def summarize(skip_list, out):
    """Write the per-cause counts of the whole skip list, not only this run."""
    counts = collections.Counter(row[3] or "unrecorded" for row in rows(skip_list))
    lines = [f"{count}\t{text}" for text, count in counts.most_common()]
    (out / "summary.tsv").write_text("".join(line + "\n" for line in lines))
    return counts


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--compiler", required=True, type=pathlib.Path)
    parser.add_argument("--skip-list", required=True, type=pathlib.Path)
    parser.add_argument("--out", required=True, type=pathlib.Path)
    parser.add_argument("--max-iterations", type=int, default=2000)
    parser.add_argument("--timeout", type=float, default=600.0)
    parser.add_argument("arguments", nargs=argparse.REMAINDER)
    options = parser.parse_args()
    arguments = options.arguments[1:] if options.arguments[:1] == ["--"] else options.arguments
    if not arguments:
        parser.error("compiler arguments are required after --")
    options.out.mkdir(parents=True, exist_ok=True)
    options.skip_list.touch()
    keys = {(row[0], row[1]) for row in rows(options.skip_list)}
    records = []
    fences = []
    bodies = []
    outcome = "iteration limit"
    stop = None
    for iteration in range(1, options.max_iterations + 1):
        command = [str(options.compiler), "--bringup-skip-list", str(options.skip_list), *arguments]
        start = time.monotonic()
        try:
            result = subprocess.run(command, capture_output=True, text=True,
                                    timeout=options.timeout, check=False)
        except subprocess.TimeoutExpired:
            outcome = "timeout"
            break
        seconds = round(time.monotonic() - start, 2)
        (options.out / "last.stderr").write_text(result.stderr)
        fences = waived(result.stderr)
        (options.out / "waived.tsv").write_text("".join(fence + "\n" for fence in fences))
        bodies = unproven(result.stderr)
        (options.out / "unproven.tsv").write_text("".join(body + "\n" for body in bodies))
        if result.returncode == 0:
            outcome = "compiled"
            stop = {"iteration": iteration, "seconds": seconds}
            break
        blockers, unattributed, message = diagnose(result.stderr)
        added = []
        with options.skip_list.open("a") as listed:
            for path, offset, name, own, stage in blockers:
                if (path, offset) in keys:
                    continue
                keys.add((path, offset))
                text = cause(own or message)
                listed.write(f"{path}\t{offset}\t{name}\t{text}"
                             + (f"\t{stage}" if stage else "") + "\n")
                added.append({"iteration": iteration, "path": path, "offset": offset,
                              "name": name, "message": text, "stage": stage})
        records.extend(added)
        print(f"{iteration}\t{seconds}s\t+{len(added)}\tlisted {len(keys)}"
              f"\tunattributed {len(unattributed)}\t{cause(message)}", flush=True)
        if not added:
            outcome = ("unattributed failure" if unattributed or not blockers
                       else "listed function failed again")
            stop = {"iteration": iteration, "seconds": seconds, "status": result.returncode,
                    "message": message, "unattributed": unattributed,
                    "repeated": [list(blocker) for blocker in blockers]}
            break
    counts = summarize(options.skip_list, options.out)
    (options.out / "records.json").write_text(json.dumps(
        {"outcome": outcome, "listed": len(keys), "added": len(records), "stop": stop,
         "waived": fences, "unproven": bodies, "command": arguments, "records": records},
        indent=1) + "\n")
    print(f"outcome: {outcome}; listed {len(keys)}; distinct causes {len(counts)}"
          f"; waived fences {len(fences)}; unproven bodies {len(bodies)}")
    if stop and stop.get("message"):
        print(f"last failure: {stop['message']}")
        for text in stop["unattributed"]:
            print(f"unattributed: {text}")
        for path, offset, name, text, *_ in stop["repeated"]:
            print(f"repeated: {path}:{offset} {name}: {text}")
    return 0 if outcome == "compiled" else 1


if __name__ == "__main__":
    sys.exit(main())
