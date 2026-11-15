#!/usr/bin/env python3
"""Release one implemented component as a finite course contract.

The recipe, in order (state is kept under obj/publication/UNIT):

  inventory UNIT              draft outcome tables -> declared candidates (no compiler runs)
  probe UNIT --app BIN        pinned rustc, the candidate and the native lane observe every candidate
  review UNIT                 list rejection diagnostics that read like an implementation gap
  declare UNIT                passing candidates -> the unit manifest; everything else -> deferred.json
  checks UNIT --app BIN       metadata, independent rustc, references, layout, text and native checks
  summary UNIT                family and deferred tables for the release record
  wire UNIT --record DOC ...  plan, order table, ROADMAP and handout rows
  ledger ID=TEXT ...          requirement-row publication strings

A candidate is published only when its independently declared outcome is what both
compilers produce. Nothing here writes a reference: `checks --bless` calls the course
writer, which refuses unless every declared outcome already matches.
"""
import argparse
import collections
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import re
import subprocess
import sys
import time

from case_inputs import support_files
from course import ROOT, run_case
from unit_tables import EMPTY_UNIT_REASON, discovery_claims, inventory, load_config

# Wording that reports a missing implementation rather than a Rust rule. A rejection that
# passes only through such wording is the right status for the wrong reason.
GAP_WORDING = re.compile(
    r"not implemented|unsupported|not supported|not yet|bootstrap|profile|unavailable|not available|"
    r"requires its actual|requires actual|cannot yet|is not qualified|budget|limit|requires a root main", re.I)
POSITION = re.compile(r"file \d+: \d+: ")


def state(root, unit):
    return root / "obj/publication" / unit


def read(root, unit, name):
    path = state(root, unit) / name
    if not path.is_file():
        raise ValueError(f"missing {path.relative_to(root)}; run the earlier recipe step first")
    return json.loads(path.read_text())


def write(root, unit, name, document):
    folder = state(root, unit)
    folder.mkdir(parents=True, exist_ok=True)
    (folder / name).write_text(json.dumps(document, indent=1, ensure_ascii=False) + "\n")


def run_inventory(root, unit):
    document, foreign = inventory(root, unit)
    write(root, unit, "candidates.json", document)
    cases = document["cases"]
    accept = sum(case["exit_status"] == 0 for case in cases)
    print(f"{unit}: {len(cases)} candidates ({accept} accept/{len(cases) - accept} reject); "
          f"{len(document['undeclared_sources'])} undeclared sources in claimed families")
    for name in document["undeclared_sources"]:
        print("  undeclared:", name)
    if foreign:
        print(f"  skipped {len(foreign)} sources owned by another assignment: {foreign[:6]}")
    for row, reason in document["unrepresented_rows"].items():
        print(f"  not representable: {row}: {reason}")
    notes = collections.defaultdict(list)
    for path, rows in document["bindings"].items():
        for note in sorted({note for row in rows for note in row["notes"]}):
            notes[note].append(path)
    for note, paths in notes.items():
        print(f"  note ({len(paths)}): {note}" + ("" if len(paths) > 6 else ": " + ", ".join(paths)))


def observe(stage, mode, case, app, directory, tools):
    """Observe one candidate; a passing record carries the case as it can be declared."""
    from check_native import REVIEWED_WARNING_CODES, check_case, declares_empty_lowir
    from check_syntax import independent_rustc
    record = {"path": case["path"], "verdict": "defer", "stage": "rustc-declaration", "message": None}
    try:
        folder = directory / "syntax" / case["path"].removesuffix(".rs")

        def independent(candidate):
            result, output = independent_rustc(stage, mode, candidate, folder)
            output.unlink(missing_ok=True)
            return result

        lints = dict(case, rustc=dict(case["rustc"], default_lints=True))
        result = independent(case)
        if result.returncode != case["rustc"]["status"] and not case["rustc"].get("default_lints", False):
            # Draft tables were qualified with default lints and the course checker caps them:
            # a case declares default_lints exactly when its outcome needs a lint level.
            retry = independent(lints)
            if retry.returncode == case["rustc"]["status"]:
                case, result = lints, retry
        record["rustc_status"] = result.returncode
        if result.returncode != case["rustc"]["status"]:
            lines = result.stderr.decode(errors="replace").strip().splitlines()
            raise ValueError(f"pinned rustc status {result.returncode} contradicts the declared "
                             f"{case['rustc']['status']}: {lines[0] if lines else ''}")
        record["stage"] = "frontend"
        _, actual = run_case(ROOT, stage, mode, case, [str(app)], 30)
        first = actual[1].decode(errors="replace").strip().splitlines()
        record.update(frontend_status=int(actual[2]), frontend_message=first[0] if first else "",
                      lowir_bytes=len(actual[0]), lowir_lines=actual[0].count(b"\n"))
        if actual[2] != f"{case['exit_status']}\n".encode():
            raise ValueError(record["frontend_message"] or "accepted where Rust rejects")
        if case["exit_status"] == 0:
            codes = sorted(set(re.findall(r"^warning\[([a-z_]+)\]: ", actual[1].decode(errors="replace"), re.M)))
            if codes and "warnings" not in case:
                # A successful warning is declared only when pinned rustc reports the same lints.
                record["stage"] = "frontend-warnings"
                reported = independent(lints).stderr.decode(errors="replace")
                for code in codes:
                    if code not in REVIEWED_WARNING_CODES:
                        raise ValueError(f"warning[{code}] is outside the reviewed warning vocabulary")
                    if code not in reported:
                        raise ValueError(f"the candidate reports warning[{code}], which pinned rustc does not")
                case = dict(case, warnings=codes)
            if case["rustc"]["kind"] == "library" and not actual[0] and not declares_empty_lowir(case):
                # The native checker then requires that neither compiler's object defines anything.
                case = dict(case, native={"kind": "object", "lowir": "empty", "reason": EMPTY_UNIT_REASON})
        record["stage"] = "native"
        evidence = check_case(stage, mode, case, app, directory / "native", tools, observed_warnings=True)
        if not evidence["passed"]:
            raise ValueError(evidence["error"])
        record.update(verdict="pass", case=case)
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        record["message"] = " ".join(str(error).split())[:400]
    return record


def run_probe(root, unit, app, jobs):
    from backend import load_tools
    from check_native import candidate_identity
    candidates = read(root, unit, "candidates.json")
    version = subprocess.check_output(["rustc", "--version"], text=True).strip()
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected pinned rustc 1.98.0: {version}")
    identity = candidate_identity(app)
    tools = load_tools(("lowir", "lowir2native", "cppgm"))
    directory = state(root, unit) / "probe"
    started = time.time()
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        records = list(pool.map(lambda case: observe(candidates["stage"], candidates["mode"], case, app,
                                                     directory, tools), candidates["cases"]))
    if candidate_identity(app) != identity:
        raise ValueError("candidate bytes changed during the probe")
    write(root, unit, "probe.json", {"rustc": version, **identity, "seconds": round(time.time() - started, 2),
                                     "backend": json.loads((root / "backend.lock.json").read_text()),
                                     "cases": records})
    passed = sum(record["verdict"] == "pass" for record in records)
    print(f"{unit}: {passed}/{len(records)} candidates pass; candidate {identity['candidate_sha256'][:16]}")
    for record in records:
        if record["verdict"] != "pass":
            print(f"  DEFER [{record['stage']}] {record['path']}: {record['message']}")


def gap_rejections(candidates, probe):
    """Group passing rejections whose first diagnostic reads like an implementation gap."""
    declared = {case["path"]: case for case in candidates["cases"]}
    flagged = collections.defaultdict(list)
    for record in probe["cases"]:
        message = POSITION.sub("", record.get("frontend_message", ""))
        if (record["verdict"] == "pass" and declared[record["path"]]["exit_status"] == 1
                and GAP_WORDING.search(message)):
            flagged[message].append(record["path"])
    return dict(flagged)


def decisions(root, unit, extra=()):
    """Reviewed deferrals: the committed table configuration plus this invocation's."""
    reviewed = dict(load_config(root, unit)[1].get("review", {}))
    for item in extra:
        path, separator, reason = item.partition("=")
        if not separator or not reason.strip():
            raise ValueError("an exclusion is PATH=REASON")
        reviewed[path] = reason
    return reviewed


def right_reasons(root, unit):
    """Flagged rejections reviewed as naming the Rust rule itself, with the reason on record."""
    return load_config(root, unit)[1].get("accepted", {})


def run_review(root, unit):
    flagged = gap_rejections(read(root, unit, "candidates.json"), read(root, unit, "probe.json"))
    reviewed = {**decisions(root, unit), **right_reasons(root, unit)}
    for message, paths in sorted(flagged.items(), key=lambda item: -len(item[1])):
        pending = [path for path in paths if path not in reviewed]
        print(f"{len(paths)} ({len(pending)} undecided) {message} :: {pending[:4] or paths[:4]}")
    print(f"{unit}: {len(flagged)} rejection diagnostics read like an implementation gap")


def run_declare(root, unit, extra=()):
    candidates, probe = read(root, unit, "candidates.json"), read(root, unit, "probe.json")
    stage = candidates["stage"]
    reviewed = decisions(root, unit, extra)
    accepted = right_reasons(root, unit)
    undecided = sorted(path for paths in gap_rejections(candidates, probe).values() for path in paths
                       if path not in reviewed and path not in accepted)
    if undecided:
        raise ValueError("rejections that read like an implementation gap need a reviewed decision in "
                         "unit_tables.json: `review` defers one (or --exclude PATH=REASON), `accepted` "
                         f"records why its wording is the Rust rule: {undecided}")
    if set(reviewed) & set(accepted):
        raise ValueError(f"a rejection cannot be both deferred and accepted: {sorted(set(reviewed) & set(accepted))}")
    observed = {record["path"]: record for record in probe["cases"]}
    if set(observed) != {case["path"] for case in candidates["cases"]}:
        raise ValueError("the probe does not cover the current candidates; probe again")
    kept, deferred = [], []
    for case in candidates["cases"]:
        record = observed[case["path"]]
        if case["path"] in reviewed:
            deferred.append({"path": case["path"], "rust": case["rustc"]["status"], "stage": "review",
                             "message": reviewed[case["path"]]})
        elif record["verdict"] == "pass":
            kept.append(record["case"])
        else:
            deferred.append({"path": case["path"], "rust": case["rustc"]["status"],
                             "stage": record["stage"], "message": record["message"]})
    sources = {source.relative_to(root / stage).as_posix() for family in candidates["families"]
               for source in (root / stage / family).glob("*.rs")}
    claimed = {case["path"] for case in kept} | {path for case in kept for path in support_files(case)
                                                  if path.endswith(".rs")}
    modes = sorted({case["mode"] for case in kept if "mode" in case} - {candidates["mode"]})
    manifest = {"schema": 1, "mode": candidates["mode"], **({"additional_modes": modes} if modes else {}),
                "language": "Rust 1.98.0", "edition": "2024", "target": "x86_64-unknown-linux-gnu",
                "discovery": discovery_claims(root, stage, claimed, sources), "cases": kept}
    path = root / stage / "tests/units" / f"{unit}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n")
    write(root, unit, "deferred.json", deferred)
    accept = sum(case["exit_status"] == 0 for case in kept)
    execute = sum(case.get("native", {}).get("kind") == "execute" for case in kept)
    print(f"{path.relative_to(root)}: {len(kept)} cases ({accept} accept/{len(kept) - accept} reject; "
          f"{execute} native executions); {len(deferred)} deferred")


def run_checks(root, unit, app, jobs, bless):
    """The official checks in release order; stop at the first failure."""
    stage = load_config(root, unit)[0]
    text = str((state(root, unit) / "text").relative_to(root))
    selected = [stage, "--unit", unit, "--app", str(app), "--jobs", str(jobs)]
    steps = [("metadata", ["assignment_metadata.py", "--unit", unit]),
             ("rustc", ["check_syntax.py", *selected])]
    if bless:
        steps.append(("bless", ["course.py", *selected, "--bless", "--evidence-root", text]))
    steps += [("layout", ["check_layout.py", "--unit", unit]),
              ("text", ["course.py", *selected, "--evidence-root", text]),
              ("native", ["check_native.py", *selected])]
    state(root, unit).mkdir(parents=True, exist_ok=True)
    for name, command in steps:
        started = time.time()
        result = subprocess.run([sys.executable, str(root / "scripts" / command[0]), *command[1:]],
                                cwd=root, capture_output=True, text=True, check=False)
        output = result.stdout + result.stderr
        (state(root, unit) / f"{name}.log").write_text(output)
        lines = [line for line in output.splitlines()
                 if line.strip() and not line.startswith(("Evidence:", "Removed")) and "PASS (1/1)" not in line]
        print(f"{name}: status {result.returncode} in {time.time() - started:.1f}s | {lines[-1] if lines else ''}")
        if result.returncode:
            for line in lines[-12:-1]:
                print("  " + line)
            raise ValueError(f"{unit}: {name} failed; see {state(root, unit).relative_to(root)}/{name}.log")
    print(f"{unit}: every release check passes")


def family(path):
    parts = path.split("/")
    return "/".join(parts[2:-1]) if parts[:2] == ["tests", "spec"] and len(parts) > 3 else "/".join(parts[1:-1])


def run_summary(root, unit):
    stage = load_config(root, unit)[0]
    cases = json.loads((root / stage / "tests/units" / f"{unit}.json").read_text())["cases"]
    rows = collections.defaultdict(lambda: [0, 0, 0])
    for case in cases:
        row = rows[family(case["path"])]
        row[0] += 1
        row[1] += case["exit_status"] == 0
        row[2] += case.get("native", {}).get("kind") == "execute"
    print("| Family | Cases | Accept | Reject | Executed |\n| --- | --- | --- | --- | --- |")
    for name, (count, accept, executed) in sorted(rows.items()):
        print(f"| `{name}` | {count} | {accept} | {count - accept} | {executed} |")
    natives = collections.Counter(case.get("native", {}).get("kind", "rejected") for case in cases)
    print(f"\n{len(cases)} cases; native kinds {dict(natives)}; "
          f"{sum('warnings' in case for case in cases)} declared warnings; "
          f"{sum(case['rustc'].get('default_lints', False) for case in cases)} default-lint outcomes\n")
    groups = collections.OrderedDict()
    for item in read(root, unit, "deferred.json"):
        message = POSITION.sub("", (item["message"] or "").replace(str(root) + "/", ""))[:220]
        key = ("accept" if item["rust"] == 0 else "reject", f"[{item['stage']}] {message}")
        groups.setdefault(key, []).append(item["path"].removeprefix("tests/spec/"))
    print("| Fixtures | Rust | Observed with the candidate |\n| --- | --- | --- |")
    for (rust, message), paths in groups.items():
        print(f"| {', '.join(f'`{path}`' for path in paths)} ({len(paths)}) | {rust} | {message} |")


def replace_once(text, old, new, label):
    if text.count(old) != 1:
        raise ValueError(f"{label}: anchor matched {text.count(old)} times: {old[:70]!r}")
    return text.replace(old, new)


def table_row(text, pattern, row, label):
    """Replace the unit's row, or add it after the last row of the same table."""
    rows = re.findall(pattern, text, re.M)
    if not rows:
        raise ValueError(f"{label}: no component table row to extend")
    own = [line for line in rows if line.startswith("|" + row.split("|")[1] + "|")]
    if own:
        return replace_once(text, own[0], row, label)
    return replace_once(text, rows[-1] + "\n", rows[-1] + "\n" + row + "\n", label)


def run_wire(root, unit, record, deferred, note=None):
    """Publish the release in the plan, the order table, ROADMAP and the owning handout."""
    stage = load_config(root, unit)[0]
    if not (root / record).is_file() or not record.startswith("docs/"):
        raise ValueError(f"missing release record under docs/: {record}")
    cases = len(json.loads((root / stage / "tests/units" / f"{unit}.json").read_text())["cases"])
    plan_path = root / "assignment-plan.json"
    plan = json.loads(plan_path.read_text(), object_pairs_hook=collections.OrderedDict)
    entry = next(item for item in plan["units"] if item["id"] == unit)
    if entry["publication"] != "released":
        text = plan_path.read_text()
        start = text.index(f'"id": "{unit}",')
        tail = f'"manifest": "{entry["manifest"]}"'
        end = text.index(tail, start)
        block = replace_once(text[start:end], '"publication": "unpublished"', '"publication": "released"', "plan")
        text = text[:start] + block + tail + f',\n      "release_evidence": "{record}"' + text[end + len(tail):]
        json.loads(text)
        plan_path.write_text(text)
    elif entry.get("release_evidence") != record:
        raise ValueError(f"{unit} is already released with a different record: {entry.get('release_evidence')}")
    order_path = root / "docs/assignment-order.md"
    order = order_path.read_text()
    pending = re.search(rf"^\| \d+ \| PA\d+ \| .*\(`{re.escape(unit)}`\) \| implemented \| unpublished \|$", order, re.M)
    if pending is not None:
        order_path.write_text(replace_once(order, pending[0], pending[0].replace("| unpublished |", "| released |"),
                                           "order table"))
    cell = f"{deferred} ({note})" if note else str(deferred)
    roadmap_path = root / "ROADMAP.md"
    roadmap_path.write_text(table_row(
        roadmap_path.read_text(), r"^\| `[a-z0-9-]+` \| \d+ \| .*\[record\]\(docs/.*\) \|$",
        f"| `{unit}` | {stage.removeprefix('pa')} | {cases:,} | {cell} | [record]({record}) |", "ROADMAP"))
    fixtures = "1 fixture" if deferred == 1 else f"{deferred} fixtures"
    readme_path = root / stage / "README.md"
    readme_path.write_text(table_row(
        readme_path.read_text(), r"^\| `[a-z0-9-]+` \| \[[a-z0-9-]+\.json\]\(tests/units/.*\) \|$",
        f"| `{unit}` | [{unit}.json](tests/units/{unit}.json) | {cases:,} | [{fixtures}](../{record}#deferred-list) |",
        f"{stage}/README.md"))
    print(f"{unit}: {cases} cases and {deferred} deferred wired into the plan, order table, ROADMAP and "
          f"{stage}/README.md")


def run_ledger(root, items):
    """Text-preserving edit of requirement-row publication strings."""
    path = root / "docs/assignment-requirements.json"
    text = path.read_text()
    for item in items:
        key, separator, value = item.partition("=")
        if not separator:
            raise ValueError("a ledger edit is ID=TEXT")
        start = text.index(f'"id": "{key}",')
        match = re.compile(r'"publication": "[^"\n]*"').search(text, start)
        if match is None or match.start() > text.index('"units"', start):
            raise ValueError(f"no publication line for requirement {key}")
        text = text[:match.start()] + '"publication": ' + json.dumps(value, ensure_ascii=False) + text[match.end():]
    json.loads(text)
    path.write_text(text)


def main(argv=None, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("inventory", "probe", "review", "declare", "checks", "summary", "wire"):
        command = commands.add_parser(name)
        command.add_argument("unit")
        if name in ("probe", "checks"):
            command.add_argument("--app", type=Path, required=True)
            command.add_argument("--jobs", type=int, choices=range(1, 9), default=6)
    commands.choices["checks"].add_argument("--bless", action="store_true",
                                            help="generate references once every declared outcome matches")
    commands.choices["declare"].add_argument("--exclude", action="append", default=[], metavar="PATH=REASON")
    commands.choices["wire"].add_argument("--record", required=True, help="release record under docs/")
    commands.choices["wire"].add_argument("--deferred", type=int, required=True)
    commands.choices["wire"].add_argument("--note", help="short qualifier for the ROADMAP deferred cell")
    commands.add_parser("ledger").add_argument("items", nargs="+", metavar="ID=TEXT")
    args = parser.parse_args(argv)
    try:
        if args.command == "inventory":
            run_inventory(root, args.unit)
        elif args.command == "probe":
            run_probe(root, args.unit, args.app.resolve(), args.jobs)
        elif args.command == "review":
            run_review(root, args.unit)
        elif args.command == "declare":
            run_declare(root, args.unit, args.exclude)
        elif args.command == "checks":
            run_checks(root, args.unit, args.app.resolve(), args.jobs, args.bless)
        elif args.command == "summary":
            run_summary(root, args.unit)
        elif args.command == "wire":
            run_wire(root, args.unit, args.record, args.deferred, args.note)
        else:
            run_ledger(root, args.items)
        return 0
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f"unit release failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
