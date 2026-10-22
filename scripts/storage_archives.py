"""Losslessly archive large rustc type diagnostics before releasing their blocks."""
import gzip
import hashlib
import os
import re
import stat
import tempfile


def large_diagnostic(path, info):
    return (stat.S_ISREG(info.st_mode) and info.st_size >= 1024 ** 2
            and re.fullmatch(r".+\.long-type-\d+\.txt", path.name) is not None)


def fingerprint(info):
    return info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns


def digest(stream):
    result = hashlib.sha256()
    size = 0
    while block := stream.read(1024 ** 2):
        result.update(block)
        size += len(block)
    return size, result.hexdigest()


def restored_digest(path):
    try:
        with gzip.open(path, "rb") as restored:
            return digest(restored)
    except (EOFError, OSError) as error:
        raise ValueError(f"Cannot verify diagnostic archive: {path}") from error


def archive_diagnostic(path, info):
    """Create and verify an archive; the caller records it before removing input.

    Existing archives must contain exactly the same bytes. Source changes and
    symlinks fail closed. Temporary storage is at most one compressed file.
    """
    archive = path.with_name(path.name + ".gz")
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".diagnostic-",
                                     suffix=".tmp") as temporary:
        with path.open("rb") as source:
            if fingerprint(os.fstat(source.fileno())) != fingerprint(info):
                raise ValueError(f"Diagnostic changed before archiving: {path}")
            original = hashlib.sha256()
            with gzip.GzipFile(filename="", mode="wb", fileobj=temporary,
                               compresslevel=1, mtime=0) as output:
                while block := source.read(1024 ** 2):
                    original.update(block)
                    output.write(block)
        temporary.flush()
        expected = info.st_size, original.hexdigest()
        if restored_digest(temporary.name) != expected:
            raise ValueError(f"Diagnostic archive verification failed: {path}")
        if fingerprint(path.lstat()) != fingerprint(info):
            raise ValueError(f"Diagnostic changed during archiving: {path}")
        os.fsync(temporary.fileno())
        try:
            os.link(temporary.name, archive)
        except FileExistsError:
            if not stat.S_ISREG(archive.lstat().st_mode):
                raise ValueError(f"Diagnostic archive is not a regular file: {archive}") from None
            if restored_digest(archive) != expected:
                raise ValueError(f"Existing diagnostic archive differs: {archive}") from None
    return archive, expected[1]
