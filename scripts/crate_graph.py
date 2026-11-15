"""Crate-graph course cases: provider crates, their metadata artifacts and one consumer.

A case lists its provider crates in build order under `crates`. Each provider names
its source, crate name and disambiguator and binds its own dependencies by alias in
`externs`; the case's own `externs` binds the consumer's. Everything a unit reaches
only through another provider is passed as dependency metadata, never by alias.

    "crates": [{"name": "first", "path": "tests/spec/scalar/first.rs",
                "crate_name": "shared", "disambiguator": "first", "externs": {}},
               {"name": "second", "path": "tests/spec/scalar/second.rs",
                "crate_name": "shared", "disambiguator": "second", "externs": {"one": "first"}}],
    "externs": {"renamed": "second"}

The candidate compiles each provider to LowIR and a metadata artifact, then the
consumer against those artifacts. Pinned rustc builds the same graph as rlibs. A
provider is an input: one that fails to build is a harness error, never an outcome.
"""
from collections import namedtuple
import re
import subprocess

PROVIDER_KEYS = {"name", "path", "crate_name", "disambiguator", "externs"}
NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
ALIAS = re.compile(r"[A-Za-z_][A-Za-z_0-9]{0,63}")
Graph = namedtuple("Graph", "arguments units artifacts")


def bindings(stage, externs, known):
    if (not isinstance(externs, dict)
            or any(not isinstance(alias, str) or ALIAS.fullmatch(alias) is None or alias == "_"
                   or target not in known for alias, target in externs.items())):
        raise ValueError(f"{stage}: externs must bind valid aliases to earlier provider crates")


def validate(root, stage, case, mode):
    """Check one case's crate graph; return the provider sources it declares."""
    if "crates" not in case:
        if "externs" in case:
            raise ValueError(f"{stage}: externs require declared provider crates: {case['path']}")
        return set()
    from case_inputs import declared_file
    from course import library_arguments
    crates = case["crates"]
    if mode != "--emit-lowir" or not isinstance(crates, list) or not crates:
        raise ValueError(f"{stage}: provider crates are a nonempty list on an --emit-lowir case: {case['path']}")
    known = []
    for provider in crates:
        if not isinstance(provider, dict) or set(provider) != PROVIDER_KEYS:
            raise ValueError(f"{stage}: a provider crate declares exactly {sorted(PROVIDER_KEYS)}")
        name = provider["name"]
        if not isinstance(name, str) or NAME.fullmatch(name) is None or name in known:
            raise ValueError(f"{stage}: provider crate names must be unique lowercase identifiers")
        declared_file(root, stage, provider["path"], ".rs", "provider crate")
        library_arguments(identity(provider), "--emit-lowir")
        bindings(stage, provider["externs"], known)
        known.append(name)
    bindings(stage, case.get("externs"), known)
    unused = set(known) - closure(case, case["externs"])
    if unused:
        raise ValueError(f"{stage}: provider crates the consumer never reaches: {sorted(unused)}")
    return {provider["path"] for provider in crates}


def identity(provider):
    """The provider as the library case whose identity arguments it shares."""
    return {"rustc": {"kind": "library"},
            "library": {"crate_name": provider["crate_name"], "disambiguator": provider["disambiguator"]}}


def closure(case, externs):
    """Every provider reachable from these bindings."""
    providers = {provider["name"]: provider for provider in case["crates"]}
    reached, pending = set(), list(externs.values())
    while pending:
        name = pending.pop()
        if name not in reached:
            reached.add(name)
            pending.extend(providers[name]["externs"].values())
    return reached


def transitive(case, externs):
    """Providers reached only through another provider, in declaration order."""
    reached = closure(case, externs) - set(externs.values())
    return [provider["name"] for provider in case["crates"] if provider["name"] in reached]


def provider_sources(case):
    return [provider["path"] for provider in case.get("crates", [])]


def candidate_arguments(case, externs, artifacts):
    arguments = []
    for alias, name in externs.items():
        arguments += ["--extern", f"{alias}={artifacts[name][0]}"]
    for name in transitive(case, externs):
        arguments += ["--dependency-metadata", str(artifacts[name][0])]
    return arguments


def candidate_graph(root, stage, case, app, folder, timeout, env=None):
    """Compile every provider with the candidate; return the consumer's arguments and LowIR units."""
    from course import library_arguments
    folder.mkdir(parents=True, exist_ok=True)
    artifacts = {}
    for provider in case["crates"]:
        metadata, lowir = folder / f"{provider['name']}.rgm", folder / f"{provider['name']}.lowir"
        command = [*app, "--emit-lowir", "--edition=2024", *library_arguments(identity(provider), "--emit-lowir"),
                   *candidate_arguments(case, provider["externs"], artifacts),
                   "--metadata-output", str(metadata), f"{stage}/{provider['path']}"]
        try:
            result = subprocess.run(command, capture_output=True, timeout=timeout, cwd=root, env=env)
        except subprocess.TimeoutExpired as error:
            raise ValueError(f"provider crate {provider['name']} timed out") from error
        if result.returncode or result.stderr or not metadata.is_file():
            raise ValueError(f"provider crate {provider['name']} did not build ({result.returncode}): "
                             f"{result.stderr.decode(errors='replace')}")
        lowir.write_bytes(result.stdout)
        artifacts[provider["name"]] = (metadata, lowir)
    return Graph(candidate_arguments(case, case["externs"], artifacts),
                 [artifacts[provider["name"]][1] for provider in case["crates"]], artifacts)


def rustc_graph(root, stage, case, folder, flags, env=None):
    """Build every provider as an rlib with pinned rustc; return the consumer's arguments."""
    folder.mkdir(parents=True, exist_ok=True)
    libraries = {}
    search = ["-L", f"dependency={folder}"]

    def externs(names):
        return [argument for alias, name in names.items() for argument in ("--extern", f"{alias}={libraries[name]}")]

    for provider in case["crates"]:
        output = folder / f"lib{provider['crate_name']}-{provider['name']}.rlib"
        command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu", *flags, "--cap-lints", "allow",
                   "--crate-type=rlib", "--crate-name", provider["crate_name"],
                   "-C", f"metadata={provider['disambiguator']}", *search, *externs(provider["externs"]),
                   str(root / stage / provider["path"]), "-o", str(output)]
        result = subprocess.run(command, capture_output=True, timeout=60, env=env)
        if result.returncode:
            raise ValueError(f"pinned rustc rejects provider crate {provider['name']}: "
                             f"{result.stderr.decode(errors='replace')}")
        libraries[provider["name"]] = output
    return [*search, *externs(case["externs"])]
