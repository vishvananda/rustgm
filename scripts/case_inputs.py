"""Declared inputs of one course case other than its primary source.

- `auxiliary`: module sources the primary reads through its own directory (file input).
- `prelude`: one source whose bytes precede the primary's; the compiler reads the
  joined bytes as a single translation unit on standard input.
- `native.providers`: host C translation units linked into both native executables.
- `crates`: provider crates the case is compiled against; see `crate_graph`.

Every such file is declared in the manifest and hashed in the reference receipt.
"""
from crate_graph import provider_sources, validate as validate_graph


def declared_file(root, stage, name, suffix, role):
    from course import check_relative_path
    check_relative_path(name, stage)
    tests = (root / stage / "tests").resolve()
    path = (root / stage / name).resolve()
    if not name.endswith(suffix) or not path.is_relative_to(tests) or not path.is_file():
        raise ValueError(f"{stage}: invalid {role} path {name}")


def validate(root, stage, case, primaries, mode):
    """Check one case's prelude, host providers and crate graph; return the extra Rust sources it declares."""
    sources = validate_graph(root, stage, case, mode)
    if "prelude" in case:
        prelude = case["prelude"]
        declared_file(root, stage, prelude, ".rs", "prelude")
        if prelude in primaries:
            raise ValueError(f"{stage}: a prelude cannot also be a primary fixture: {prelude}")
        if case.get("input", "stdin") != "stdin":
            raise ValueError(f"{stage}: a prelude requires stdin input: {case['path']}")
        if case.get("rustc", {}).get("kind", "program") not in ("program", "library"):
            raise ValueError(f"{stage}: a prelude requires a complete program or library: {case['path']}")
        sources.add(prelude)
    native = case.get("native")
    if isinstance(native, dict) and "providers" in native:
        providers = native["providers"]
        if (not isinstance(providers, list) or not providers
                or any(not isinstance(provider, str) for provider in providers)
                or len(set(providers)) != len(providers)):
            raise ValueError(f"{stage}: native providers must be a nonempty list of unique paths")
        for provider in providers:
            declared_file(root, stage, provider, ".c", "host provider")
    return sources


def support_files(case):
    """Every declared input other than the primary, once each, in declaration order."""
    files = list(case.get("auxiliary", []))
    if "prelude" in case:
        files.append(case["prelude"])
    native = case.get("native")
    if isinstance(native, dict):
        files.extend(native.get("providers", []))
    files.extend(provider_sources(case))
    return list(dict.fromkeys(files))


def source_bytes(root, stage, case):
    """The translation unit the compilers read: the prelude, when declared, then the fixture."""
    data = (root / stage / case["path"]).read_bytes()
    if "prelude" in case:
        return (root / stage / case["prelude"]).read_bytes() + data
    return data
