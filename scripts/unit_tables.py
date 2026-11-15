"""Turn a component's draft outcome tables into declared course-case candidates.

No compiler runs here. A draft row keeps its independent pinned-Rust outcome as
the contract outcome; what the reference compiler does with it is observed later.
"""
import hashlib
import json
from pathlib import Path
import re

from unit_graphs import graph_rows

ENTRY_FREE_REASON = ("Complete entry-free source contract; compare relocatable object acceptance "
                     "without adding main.")
CODEGEN_REASON = ("The draft outcome table declares acceptance and typed lowering only; "
                  "no runtime outcome was declared for this source.")
LIBRARY_REASON = "Library crate of a dependency graph; compare relocatable object acceptance."
EMPTY_UNIT_REASON = ("Entry-free source contract with no exported root: acceptance only, "
                     "and both objects must define nothing.")
# Draft tables name the independent outcome in several historical ways, strongest first.
RUST_STATUS_KEYS = ("rustc_expected", "rustc_status", "rustc_hypothesis")
COPIED_KEYS = ("args", "auxiliary", "library", "source_provider", "warnings", "environment", "crates", "externs")
CALLS_CHECK = re.compile(r"\bcheck\s*\(")
DEFINES_CHECK = re.compile(r"\bfn\s+check\b")


def load_config(root, unit):
    """Return (owning assignment, table configuration) for one declared unit."""
    config = json.loads((root / "scripts/unit_tables.json").read_text())
    if config.get("schema") != 1 or unit not in config.get("units", {}):
        raise ValueError(f"no draft-table configuration for unit: {unit}")
    owners = [entry["owner"] for entry in json.loads((root / "assignment-plan.json").read_text())["units"]
              if entry["id"] == unit]
    if len(owners) != 1:
        raise ValueError(f"unknown assignment unit: {unit}")
    return owners[0], config["units"][unit]


def integer(value):
    return isinstance(value, int) and not isinstance(value, bool)


def rust_status(row, native_rows):
    """The row's independent outcome, or None when the table only records the candidate's."""
    rustc = row.get("rustc")
    if isinstance(rustc, dict):
        return rustc["status"]
    if integer(rustc):
        return rustc
    for key in RUST_STATUS_KEYS:
        if key in row:
            return row[key]
    if "accepted" in row:
        return 0 if row["accepted"] else 1
    if "rustc_run_status" in row:
        return 0  # A declared run implies independent acceptance.
    if "expected_status" in row or "expected_compile_status" in row:
        return row.get("expected_status", row.get("expected_compile_status"))
    if native_rows and row.get("status", row.get("expected")) == 0:
        return 0
    return None


def candidate_status(row):
    """The outcome the draft table recorded for the reference compiler, when it has one."""
    legacy = row.get("exit_status", row.get("required_candidate_status", row.get("expected")))
    if legacy is None and integer(row.get("frontend")):
        legacy = row["frontend"]
    if legacy is None:
        legacy = row.get("frontend_status", row.get("frontend_expected_after_implementation"))
    return legacy


def native_declaration(row, native_rows):
    native = row.get("native")
    if isinstance(native, dict):
        return dict(native)
    if (native is True or row.get("native_valid") is True and row.get("native_expected") == 0
            or row.get("rustc_run_status") == 0 or native_rows):
        traps = bool(row.get("traps") or row.get("panic"))
        return {"kind": "execute", "outcome": "panic" if traps else "pass"}
    return {"kind": "codegen", "reason": CODEGEN_REASON}


def host_providers(row, table_provider):
    """Host C sources a row links: its own `.c` companions, then the table's shared provider."""
    named = [path for path in row.get("auxiliary", []) if path.endswith(".c")]
    if "host_provider" in row:
        named.append(row["host_provider"])
    if table_provider is not None:
        named.append(table_provider)
    return list(dict.fromkeys(named))


def normalize(root, row, *, pa, unit, table, default_feature, environment=None, native_rows=False,
              entry_free=None, prelude=None, provider=None):
    """One draft row becomes (course case, draft candidate status, notes)."""
    notes = []
    path = row.get("path") or row.get("input")
    if not isinstance(path, str):
        raise ValueError(f"{table}: a row names no source")
    path = path.removeprefix(pa + "/")
    source = root / pa / path
    if not source.is_file():
        raise ValueError(f"{table}: missing source {path}")
    legacy = candidate_status(row)
    status = rust_status(row, native_rows)
    if status is None:
        status = legacy
        notes.append("no independent rustc row in the draft table; verified by the probe")
    if legacy is None:
        legacy = status
    if status not in (0, 1) or legacy not in (0, 1):
        raise ValueError(f"{table}: {path}: undeclared outcome")
    rustc = dict(row["rustc"]) if isinstance(row.get("rustc"), dict) else {"kind": "program", "status": status}
    rustc.setdefault("emit", "obj")
    rustc.pop("reason", None)
    declared = row.get("source_sha256", row.get("sha256"))
    actual = hashlib.sha256(source.read_bytes()).hexdigest()
    if declared is not None and declared != actual:
        notes.append(f"draft table hash {declared[:12]} differs from current source")
    case = {"path": path, "source_sha256": actual, "exit_status": status, "input": "file",
            "features": list(row.get("features") or [default_feature]), "work_units": [unit],
            "rustc": rustc}
    case.update({key: row[key] for key in COPIED_KEYS if key in row})
    modules = [name for name in row.get("auxiliary", []) if not name.endswith(".c")]
    if modules:
        case["auxiliary"] = modules
    else:
        case.pop("auxiliary", None)
    if "mode" in row and row["mode"] != "--emit-lowir":
        case["mode"] = row["mode"]
    if environment is not None and "environment" not in case:
        case["environment"] = environment
    if status == 0:
        case["native"] = native_declaration(row, native_rows)
        if rustc["kind"] == "library" and not isinstance(row.get("native"), dict):
            case["native"] = {"kind": "object", "reason": LIBRARY_REASON}
    text = source.read_text()
    if prelude is not None and CALLS_CHECK.search(text) and not DEFINES_CHECK.search(text):
        # The maintainer native harness compiles these bodies behind its fixed check prelude.
        case["prelude"] = prelude
        del case["input"]
        notes.append("compiled behind the declared check prelude")
    if entry_free is not None and re.search(r"\bfn\s+main\s*\(", source.read_text()) is None:
        # No root main: the established course shape is an explicit library identity (as in PA24).
        case["rustc"]["kind"] = "library"
        case["library"] = dict(entry_free)
        if status == 0:
            case["native"] = {"kind": "object", "reason": ENTRY_FREE_REASON}
        notes.append("entry-free source declared as a library")
    providers = host_providers(row, provider)
    if providers and case.get("native", {}).get("kind") in ("execute", "codegen"):
        case["native"]["providers"] = providers
    if legacy != status:
        notes.append(f"draft candidate status {legacy} differs from Rust {status} (recorded gap)")
    return case, legacy, notes


def merge(cases, case, table, notes, bindings):
    """Join duplicate primaries only when their independent outcome agrees."""
    path = case["path"]
    bindings.setdefault(path, []).append({"table": table, "notes": notes})
    old = cases.get(path)
    if old is None:
        cases[path] = case
        return
    if old["exit_status"] != case["exit_status"] or old["rustc"]["status"] != case["rustc"]["status"]:
        raise ValueError(f"conflicting declared outcomes for {path}")
    for key in ("prelude", "input", "crates", "externs", "library"):
        if old.get(key) != case.get(key):
            raise ValueError(f"conflicting {key} declarations for {path}")
    native, previous = case.get("native", {}), old.get("native", {})
    providers = previous.get("providers") or native.get("providers")
    if native.get("kind") == "execute" and previous.get("kind") != "execute":
        old["native"] = previous = dict(native)
    if providers and previous.get("kind") in ("execute", "codegen"):
        previous["providers"] = providers
    old["features"].extend(feature for feature in case["features"] if feature not in old["features"])
    for key in COPIED_KEYS:
        if key in case and key not in old:
            old[key] = case[key]


def skipped(config, row):
    origin = row.get("maintainer_origin", "").split(":")[0]
    named = row.get("path") or row.get("input") or ""
    return (origin in config.get("skip_origins", ())
            or any(part in named for part in config.get("skip_paths_containing", ())))


def table_rows(root, pa, entry):
    """The rows of one configured table, and the rows its configuration skips with a reason."""
    data = json.loads((root / pa / entry["table"]).read_text())
    if "graph" in entry:
        return graph_rows(data, entry)
    return (data[entry.get("rows", "cases")] if isinstance(data, dict) else data), {}


def inventory(root, unit):
    """Read every configured table; return the candidate document and the rows owned elsewhere."""
    pa, config = load_config(root, unit)
    cases, bindings, foreign, unrepresented = {}, {}, [], {}
    for entry in config["tables"]:
        label = entry["table"] + ("#" + entry["rows"] if "rows" in entry else "")
        options = {"pa": pa, "unit": unit, "default_feature": entry["feature"],
                   "entry_free": config.get("entry_free")}
        rows, omitted = table_rows(root, pa, entry)
        unrepresented.update({f"{label}: {name}": reason for name, reason in omitted.items()})
        for row in rows:
            named = row.get("path") or row.get("input") or ""
            if skipped(config, row):
                continue
            if re.match(r"pa\d+/", named) and not named.startswith(pa + "/"):
                foreign.append(named)
                continue
            observer = None
            if isinstance(row.get("native"), dict) and "path" in row["native"]:
                # The row names a separate standalone runtime observer, not a course declaration.
                observer = row["native"]
                row = {key: value for key, value in row.items() if key != "native"}
            case, _, notes = normalize(root, row, table=label, environment=entry.get("environment"),
                                       native_rows=entry.get("native_rows", False),
                                       prelude=entry.get("prelude"), provider=entry.get("provider"), **options)
            if case["path"] in config.get("default_lints", ()):
                case["rustc"]["default_lints"] = True
            named = config.get("providers", {}).get(case["path"])
            if named and case.get("native", {}).get("kind") in ("execute", "codegen"):
                case["native"]["providers"] = list(named)
            merge(cases, case, label, notes, bindings)
            if observer is not None and observer["path"] != named:
                label_observer = label + " (native observer)"
                case, _, notes = normalize(
                    root, {"path": observer["path"], "sha256": observer.get("sha256"), "status": 0},
                    table=label_observer, native_rows=True, **options)
                merge(cases, case, label_observer, notes, bindings)
    auxiliary = {path for case in cases.values() for path in case.get("auxiliary", [])}
    families = sorted({str(Path(path).parent) for path in cases})
    undeclared = [name for family in families
                  for name in sorted(source.relative_to(root / pa).as_posix()
                                     for source in (root / pa / family).glob("*.rs"))
                  if name not in cases and name not in auxiliary]
    document = {"unit": unit, "stage": pa, "mode": "--emit-lowir", "families": families,
                "cases": list(cases.values()), "bindings": bindings, "undeclared_sources": undeclared,
                "unrepresented_rows": unrepresented}
    return document, sorted(set(foreign))


def discovery_claims(root, pa, kept, family_sources):
    """Claim a whole directory only when every source beneath it is published."""
    kept = set(kept)
    by_directory = {}
    for path in sorted(set(family_sources) | kept):
        by_directory.setdefault(str(Path(path).parent), []).append(path)
    claims = []
    for directory, sources in sorted(by_directory.items()):
        present = {source.relative_to(root / pa).as_posix() for source in (root / pa / directory).rglob("*.rs")}
        nested = any(other != directory and (other.startswith(directory + "/") or directory.startswith(other + "/"))
                     for other in by_directory)
        if present and present <= kept and not nested:
            claims.append(directory)
        else:
            claims.extend(path for path in sources if path in kept)
    return claims
