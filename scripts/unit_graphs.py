"""Read draft crate-graph outcome tables as course rows with declared provider crates.

Three table shapes exist. Each becomes a list of nodes in build order; a node's
providers are every earlier node its dependencies reach, in table order.

- `named`: rows carry `name`, `kind` (rlib/bin), `crate` and `deps` (alias -> name).
- `library`: one `role: library` row for the table's `crate`, and clients that import it.
- `crates`: rows under `crates` carry `source`, `kind` (lib/bin), `name`, `disambiguator`
  and `externs` (alias -> id), with `rustc: accept|reject`.
"""
from collections import namedtuple

Node = namedtuple("Node", "id path library crate disambiguator deps rust frontend native")


def named(data, entry):
    for row in data["cases"]:
        yield Node(row["name"], row["path"], row["kind"] == "rlib", row["crate"], row["name"].replace("-", "_"),
                   row["deps"], row["rustc"], row["frontend_expected_after_implementation"], row["native"])


def library(data, entry):
    for row in data["cases"]:
        owner = row["role"] == "library"
        stem = row["path"].rsplit("/", 1)[-1].removesuffix(".rs")
        yield Node(stem, row["path"], owner, data["crate"], entry["disambiguator"],
                   {} if owner else {data["crate"]: "library"}, row["rustc"],
                   row["frontend_expected_after_implementation"], row["native"])


def crates(data, entry):
    for row in data["crates"]:
        owner = row["kind"] == "lib"
        status = {"accept": 0, "reject": 1}[row["rustc"]]
        yield Node(row.get("id") or row.get("disambiguator") or row["source"], "tests/" + row["source"], owner,
                   row.get("name"), row.get("disambiguator"), row["externs"], status, status,
                   row.get("native_exit") == 0)


SHAPES = {"named": named, "library": library, "crates": crates}


def graph_rows(data, entry):
    """Return (rows in the common draft form, {skipped row id: recorded reason})."""
    nodes, rows = {}, []
    skipped = dict(entry.get("skip_rows", {}))
    for node in SHAPES[entry["graph"]](data, entry):
        if node.id in skipped:
            continue
        if node.id in nodes or any(target not in nodes for target in node.deps.values()):
            raise ValueError(f"{entry['table']}: {node.id}: duplicate row or a dependency on a later row")
        reached, pending = set(), list(node.deps.values())
        while pending:
            name = pending.pop()
            if name not in reached:
                reached.add(name)
                pending.extend(nodes[name].deps.values())
        providers = [nodes[name] for name in nodes if name in reached]
        if any(not provider.library or provider.rust != 0 for provider in providers):
            raise ValueError(f"{entry['table']}: {node.id}: a dependency is not an accepted library crate")
        row = {"path": node.path, "exit_status": node.frontend, "native": node.native,
               "rustc": {"kind": "library" if node.library else "program", "status": node.rust}}
        if node.library:
            row["library"] = {"crate_name": node.crate, "disambiguator": node.disambiguator}
        if providers:
            row["crates"] = [{"name": provider.id, "path": provider.path, "crate_name": provider.crate,
                              "disambiguator": provider.disambiguator, "externs": dict(provider.deps)}
                             for provider in providers]
            row["externs"] = dict(node.deps)
        nodes[node.id] = node
        rows.append(row)
    missing = set(skipped) - {node.id for node in SHAPES[entry["graph"]](data, entry)}
    if missing:
        raise ValueError(f"{entry['table']}: configured skips name no row: {sorted(missing)}")
    return rows, skipped
