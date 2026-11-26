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
import os
import re
import subprocess

PROVIDER_KEYS = {"name", "path", "crate_name", "disambiguator", "externs"}
OPTIONAL_PROVIDER_KEYS = {"source_core", "source_profile", "edition", "features", "cfgs", "environment", "auxiliary", "cap_lints", "rustc_oracle"}
SOURCE_PROFILES = ("module-bodies-v13", "module-body-operations-v14", "provided-contracts-v1")
FORGE_KINDS = ("corrupt_version", "swap_roles", "foreign_role", "redirect_role")
NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
ALIAS = re.compile(r"[A-Za-z_][A-Za-z_0-9]{0,63}")
Graph = namedtuple("Graph", "arguments units artifacts")


def validate_source_profile(stage, profile, *, source_core=True):
    allowed = SOURCE_PROFILES if source_core else ("bootstrap-bodies-v15",)
    if not isinstance(profile, str) or profile not in allowed:
        raise ValueError(f"{stage}: source_profile must be one of {allowed}")


def validate_forge(stage, case, known):
    if "forge" not in case:
        return
    forge = case["forge"]
    if case.get("exit_status") != 1:
        raise ValueError(f"{stage}: forged metadata cases must declare exit_status 1")
    if not isinstance(forge, dict) or forge.get("provider") not in known:
        raise ValueError(f"{stage}: forge must target a declared provider crate")
    kind = forge.get("kind")
    if kind == "corrupt_version":
        if set(forge) != {"provider", "kind"}:
            raise ValueError(f"{stage}: corrupt_version forge takes only provider and kind")
    elif kind == "swap_roles":
        if (set(forge) != {"provider", "kind", "tag", "first", "second"}
                or forge["tag"] not in (254, 255)
                or not isinstance(forge["first"], str) or not forge["first"]
                or not isinstance(forge["second"], str) or not forge["second"]
                or forge["first"] == forge["second"]):
            raise ValueError(f"{stage}: invalid swap_roles forge specification")
    elif kind == "foreign_role":
        if (set(forge) != {"provider", "kind", "tag", "spelling"}
                or forge["tag"] not in (254, 255)
                or not isinstance(forge["spelling"], str) or not forge["spelling"]):
            raise ValueError(f"{stage}: invalid foreign_role forge specification")
    elif kind == "redirect_role":
        if (set(forge) != {"provider", "kind", "tag", "target", "source_tag", "source"}
                or forge["tag"] not in (254, 255) or forge["source_tag"] not in (254, 255)
                or not isinstance(forge["target"], str) or not forge["target"]
                or not isinstance(forge["source"], str) or not forge["source"]):
            raise ValueError(f"{stage}: invalid redirect_role forge specification")
    else:
        raise ValueError(f"{stage}: unsupported forge kind: {kind}")


def bindings(stage, externs, known):
    if (not isinstance(externs, dict)
            or any(not isinstance(alias, str) or ALIAS.fullmatch(alias) is None or alias == "_"
                   or target not in known for alias, target in externs.items())):
        raise ValueError(f"{stage}: externs must bind valid aliases to earlier provider crates")


def validate(root, stage, case, mode, primaries=()):
    """Check one case's crate graph; return the provider sources it declares."""
    if "crates" not in case:
        for key in ("externs", "source_client", "source_client_mismatch", "forge"):
            if key in case:
                raise ValueError(f"{stage}: {key} requires declared provider crates: {case['path']}")
        return set()
    from case_inputs import declared_file
    from course import case_edition, check_relative_path, library_arguments, validate_environment
    crates = case["crates"]
    if mode != "--emit-lowir" or not isinstance(crates, list) or not crates:
        raise ValueError(f"{stage}: provider crates are a nonempty list on an --emit-lowir case: {case['path']}")
    known = []
    source_cores = set()
    provider_aux = set()
    tests = (root / stage / "tests").resolve()
    for provider in crates:
        if (not isinstance(provider, dict)
                or not (PROVIDER_KEYS <= set(provider) <= PROVIDER_KEYS | OPTIONAL_PROVIDER_KEYS)):
            raise ValueError(f"{stage}: a provider crate declares {sorted(PROVIDER_KEYS)} "
                             f"(optional {sorted(OPTIONAL_PROVIDER_KEYS)})")
        name = provider["name"]
        if not isinstance(name, str) or NAME.fullmatch(name) is None or name in known:
            raise ValueError(f"{stage}: provider crate names must be unique lowercase identifiers")
        source_core = provider.get("source_core", False)
        if not isinstance(source_core, bool):
            raise ValueError(f"{stage}: provider source_core must be an explicit boolean")
        if "source_profile" in provider:
            validate_source_profile(stage, provider["source_profile"], source_core=source_core)
        if source_core:
            source_cores.add(name)
        case_edition(provider, stage)
        for key in ("features", "cfgs"):
            if key in provider:
                values = provider[key]
                if (not isinstance(values, list) or not values
                        or any(not isinstance(item, str) or not item or "\0" in item for item in values)
                        or len(set(values)) != len(values)):
                    raise ValueError(f"{stage}: provider {key} must be a nonempty list of unique strings")
        if "environment" in provider:
            validate_environment(stage, provider["environment"])
        if "cap_lints" in provider and provider["cap_lints"] != "allow":
            raise ValueError(f"{stage}: provider cap_lints must be 'allow'")
        if "rustc_oracle" in provider:
            if not isinstance(provider["rustc_oracle"], bool) or (provider["rustc_oracle"] and not source_core):
                raise ValueError(f"{stage}: provider rustc_oracle must be a boolean and requires source_core")
        declared_file(root, stage, provider["path"], ".rs", "provider crate")
        if "auxiliary" in provider:
            aux_files = provider["auxiliary"]
            if (not isinstance(aux_files, list) or not aux_files
                    or any(not isinstance(item, str) for item in aux_files)
                    or len(set(aux_files)) != len(aux_files)):
                raise ValueError(f"{stage}: provider auxiliary must be a nonempty list of unique paths")
            parent = (root / stage / provider["path"]).parent.resolve()
            for item in aux_files:
                check_relative_path(item, stage)
                child = (root / stage / item).resolve()
                if (item == provider["path"] or item in primaries
                        or not child.is_relative_to(parent) or not child.is_relative_to(tests)
                        or not child.is_file()):
                    raise ValueError(f"{stage}: invalid provider auxiliary path {item}")
                provider_aux.add(item)
        library_arguments(identity(provider), "--emit-lowir")
        bindings(stage, provider["externs"], known)
        known.append(name)
    bindings(stage, case.get("externs"), known)
    if "source_client" in case:
        if not isinstance(case["source_client"], str) or case["source_client"] not in source_cores:
            raise ValueError(f"{stage}: source_client must name a source_core provider crate")
    if "source_client_mismatch" in case:
        if (not isinstance(case["source_client_mismatch"], bool) or "source_client" not in case
                or (case["source_client_mismatch"] and case.get("exit_status") != 1)):
            raise ValueError(f"{stage}: source_client_mismatch requires source_client and exit_status 1")
    validate_forge(stage, case, known)
    unused = set(known) - closure(case, case["externs"])
    if unused:
        raise ValueError(f"{stage}: provider crates the consumer never reaches: {sorted(unused)}")
    return {provider["path"] for provider in crates} | provider_aux


def identity(provider):
    """The provider as the library case whose identity arguments it shares."""
    return {"rustc": {"kind": "library"},
            "source_provider": provider.get("source_core", False),
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
    return [path for provider in case.get("crates", [])
            for path in [provider["path"], *provider.get("auxiliary", [])]]


def provider_cfg_arguments(provider):
    arguments = []
    for feature in provider.get("features", []):
        arguments += ["--cfg", f'feature="{feature}"']
    for cfg in provider.get("cfgs", []):
        arguments += ["--cfg", cfg]
    return arguments


def provider_environment(provider, base_env):
    if "environment" not in provider:
        return base_env
    merged = dict(os.environ if base_env is None else base_env)
    for key, value in provider["environment"].items():
        if value is None:
            merged.pop(key, None)
        else:
            merged[key] = value
    return merged


def candidate_arguments(case, externs, artifacts):
    arguments = []
    for alias, name in externs.items():
        arguments += ["--extern", f"{alias}={artifacts[name][0]}"]
    for name in transitive(case, externs):
        arguments += ["--dependency-metadata", str(artifacts[name][0])]
    return arguments


def _role_address(packet, tag, spelling):
    word = spelling.encode()
    marker = bytes((tag,)) + len(word).to_bytes(4, "little") + word + bytes(4)
    offset = packet.find(marker)
    if offset < 0 or packet.find(marker, offset + 1) != -1:
        raise ValueError(f"expected one provider role binding for {spelling}")
    return offset + len(marker)


def apply_forge(artifact, forge, destination):
    packet = bytearray(artifact.read_bytes())
    kind = forge["kind"]
    if kind == "corrupt_version":
        if len(packet) < 15:
            raise ValueError("metadata artifact is too short to corrupt its version")
        packet[13:15] = b"\xff\xff"
    elif kind == "swap_roles":
        first = _role_address(packet, forge["tag"], forge["first"])
        second = _role_address(packet, forge["tag"], forge["second"])
        packet[first:first + 4], packet[second:second + 4] = (
            packet[second:second + 4], packet[first:first + 4]
        )
    elif kind == "foreign_role":
        offset = _role_address(packet, forge["tag"], forge["spelling"]) - 4
        packet[offset:offset + 4] = (1).to_bytes(4, "little")
    elif kind == "redirect_role":
        target = _role_address(packet, forge["tag"], forge["target"])
        source = _role_address(packet, forge["source_tag"], forge["source"])
        packet[target:target + 4] = packet[source:source + 4]
    else:
        raise ValueError(f"unsupported forge kind: {kind}")
    destination.write_bytes(packet)
    return destination


def consumer_arguments(case, artifacts):
    arguments = []
    if "source_client" in case:
        provider = next(entry for entry in case["crates"] if entry["name"] == case["source_client"])
        disambiguator = provider["disambiguator"]
        if case.get("source_client_mismatch"):
            disambiguator += "_mismatch"
        arguments += ["--source-client", f"{provider['crate_name']}@{disambiguator}"]
        if "library" not in case:
            client_name = "client" if provider["crate_name"] != "client" else "consumer"
            arguments += ["--crate-name", client_name,
                          "--crate-disambiguator", f"{provider['disambiguator']}_client"]
    arguments += candidate_arguments(case, case["externs"], artifacts)
    return arguments


def candidate_graph(root, stage, case, app, folder, timeout, env=None):
    """Compile every provider with the candidate; return the consumer's arguments and LowIR units."""
    from course import library_arguments
    folder.mkdir(parents=True, exist_ok=True)
    artifacts = {}
    for provider in case["crates"]:
        metadata, lowir = folder / f"{provider['name']}.rgm", folder / f"{provider['name']}.lowir"
        edition = provider.get("edition", "2024")
        lints = ["--cap-lints=allow"] if provider.get("cap_lints") == "allow" else []
        profile = ["--source-profile", provider["source_profile"]] if "source_profile" in provider else []
        command = [*app, "--emit-lowir", f"--edition={edition}", *lints, *provider_cfg_arguments(provider),
                   *library_arguments(identity(provider), "--emit-lowir"),
                   *profile, *candidate_arguments(case, provider["externs"], artifacts),
                   "--metadata-output", str(metadata), f"{stage}/{provider['path']}"]
        try:
            result = subprocess.run(command, capture_output=True, timeout=timeout, cwd=root,
                                    env=provider_environment(provider, env))
        except subprocess.TimeoutExpired as error:
            raise ValueError(f"provider crate {provider['name']} timed out") from error
        if result.returncode or result.stderr or not metadata.is_file():
            raise ValueError(f"provider crate {provider['name']} did not build ({result.returncode}): "
                             f"{result.stderr.decode(errors='replace')}")
        lowir.write_bytes(result.stdout)
        artifacts[provider["name"]] = (metadata, lowir)
    if "forge" in case:
        forge = case["forge"]
        target = forge["provider"]
        forged = apply_forge(artifacts[target][0], forge, folder / f"{target}-forged.rgm")
        artifacts[target] = (forged, artifacts[target][1])
    return Graph(consumer_arguments(case, artifacts),
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
        edition = provider.get("edition", "2024")
        oracle_cfg = ["--cfg", "rustc_oracle"] if provider.get("rustc_oracle") else []
        command = ["rustc", f"--edition={edition}", "--target", "x86_64-unknown-linux-gnu", *flags, "--cap-lints", "allow",
                   "--crate-type=rlib", "--crate-name", provider["crate_name"],
                   "-C", f"metadata={provider['disambiguator']}", *oracle_cfg, *provider_cfg_arguments(provider),
                   *search, *externs(provider["externs"]),
                   str(root / stage / provider["path"]), "-o", str(output)]
        result = subprocess.run(command, capture_output=True, timeout=60, env=provider_environment(provider, env))
        if result.returncode:
            raise ValueError(f"pinned rustc rejects provider crate {provider['name']}: "
                             f"{result.stderr.decode(errors='replace')}")
        libraries[provider["name"]] = output
    oracle = ["--cfg", "rustc_oracle"] if any(provider.get("rustc_oracle") for provider in case["crates"]) else []
    return [*search, *externs(case["externs"]), *oracle]
