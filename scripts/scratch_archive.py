"""Bounded streaming chunks and disk-backed evidence for one closed root."""
import hashlib
import itertools
import json
import os
import shutil
import sqlite3
import stat
import tarfile
import time

from storage import active_artifacts, cleanup_roots
from storage_archives import digest, fingerprint

DIRECTORY_FLAGS = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW


def identity(info):
    return [*fingerprint(info), info.st_mode, info.st_uid, info.st_gid, info.st_nlink]


def walk(folder):
    """Preorder with no-follow directory handles; memory is bounded by depth."""
    expected = folder.lstat()
    descriptor = os.open(folder, DIRECTORY_FLAGS)
    if identity(os.fstat(descriptor)) != identity(expected):
        os.close(descriptor)
        raise ValueError("Archive directory changed before traversal")
    stack = []
    try:
        stack.append((os.scandir(descriptor), descriptor, folder))
        yield folder, expected, None
        while stack:
            iterator, parent_fd, parent = stack[-1]
            try:
                entry = next(iterator)
            except StopIteration:
                iterator.close()
                os.close(parent_fd)
                stack.pop()
                continue
            path = parent / entry.name
            info = entry.stat(follow_symlinks=False)
            yield path, info, parent_fd
            if stat.S_ISDIR(info.st_mode):
                if len(stack) >= 256:
                    raise ValueError("Archive directory depth exceeds 256")
                child = os.open(entry.name, DIRECTORY_FLAGS, dir_fd=parent_fd)
                if identity(os.fstat(child)) != identity(info):
                    os.close(child)
                    raise ValueError("Archive directory changed during traversal")
                stack.append((os.scandir(child), child, path))
    finally:
        for iterator, descriptor, _ in stack:
            iterator.close()
            os.close(descriptor)


def connect(path):
    database = sqlite3.connect(path)
    database.execute("PRAGMA cache_size=-2048")
    database.execute("PRAGMA mmap_size=0")
    database.execute("PRAGMA temp_store=FILE")
    database.execute("CREATE TABLE entries (seq INTEGER PRIMARY KEY, path BLOB UNIQUE, "
                     "payload TEXT, verified INTEGER DEFAULT 0)")
    database.execute("CREATE TABLE links (device TEXT, inode TEXT, path BLOB, digest TEXT, "
                     "PRIMARY KEY(device,inode))")
    return database


def file_digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def save(path, value):
    with path.open("w") as output:
        json.dump(value, output, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


class Hashed:
    def __init__(self, source):
        self.source = source
        self.hash = hashlib.sha256()

    def read(self, count):
        block = self.source.read(count)
        self.hash.update(block)
        return block


def add_member(output, database, root, path, info, parent_fd):
    if path.name in {".keep-artifacts", ".git"}:
        raise ValueError("Archive root acquired a pin or worktree")
    name = path.relative_to(root).as_posix()
    row = {"path": name, "fingerprint": identity(info)}
    member = tarfile.TarInfo(name)
    member.mode, member.uid, member.gid = stat.S_IMODE(info.st_mode), info.st_uid, info.st_gid
    seconds, nanos = divmod(abs(info.st_mtime_ns), 1_000_000_000)
    member.mtime = info.st_mtime_ns // 1_000_000_000
    sign = "-" if info.st_mtime_ns < 0 else ""
    member.pax_headers = {"mtime": f"{sign}{seconds}.{nanos:09d}"}
    row["mtime"] = member.pax_headers["mtime"]
    if stat.S_ISDIR(info.st_mode):
        row["kind"], member.type = "directory", tarfile.DIRTYPE
        output.addfile(member)
    elif stat.S_ISLNK(info.st_mode):
        row["kind"], member.type = "symlink", tarfile.SYMTYPE
        member.linkname = os.readlink(path.name, dir_fd=parent_fd)
        row["target"] = member.linkname
        output.addfile(member)
    elif stat.S_ISREG(info.st_mode):
        add_regular(output, database, path, info, parent_fd, member, row)
    else:
        raise ValueError(f"Unsupported special file in archive root: {path}")
    # addfile caches TarInfo even on Python 3.14 with stream=True. No gettarinfo
    # call is used: its inode dictionary would also retain every source entry.
    output.members.clear()
    if identity(path.lstat()) != row["fingerprint"]:
        raise ValueError(f"Source changed while archiving: {path}")
    return row


def add_regular(output, database, path, info, parent_fd, member, row):
    previous = None
    key = str(info.st_dev), str(info.st_ino)
    if info.st_nlink > 1:
        previous = database.execute("SELECT path,digest FROM links WHERE device=? AND inode=?",
                                    key).fetchone()
    descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent_fd)
    with os.fdopen(descriptor, "rb") as source:
        if identity(os.fstat(source.fileno())) != row["fingerprint"]:
            raise ValueError("Source file identity changed before reading")
        if previous:
            size, checksum = digest(source)
            if size != info.st_size or checksum != previous[1]:
                raise ValueError("Hardlink bytes changed before archiving")
            row["kind"], member.type = "hardlink", tarfile.LNKTYPE
            row["target"] = os.fsdecode(previous[0])
            member.linkname = row["target"]
            output.addfile(member)
        else:
            row["kind"], member.type, member.size = "file", tarfile.REGTYPE, info.st_size
            hashed = Hashed(source)
            output.addfile(member, hashed)
            checksum = hashed.hash.hexdigest()
        if identity(os.fstat(source.fileno())) != row["fingerprint"]:
            raise ValueError("Source file changed while reading")
    row["sha256"] = checksum
    if info.st_nlink > 1 and previous is None:
        database.execute("INSERT INTO links VALUES (?,?,?,?)",
                         (*key, os.fsencode(row["path"]), checksum))


def verify_member(saved, member, row, database, seq):
    kind = row["kind"]
    if kind == "file":
        if not member.isreg() or member.size != row["fingerprint"][2]:
            raise ValueError("Archive regular-file shape changed")
        with saved.extractfile(member) as content:
            if digest(content) != (member.size, row["sha256"]):
                raise ValueError("Archive regular-file bytes changed")
    elif kind == "hardlink":
        target = database.execute("SELECT seq,payload,verified FROM entries WHERE path=?",
                                  (os.fsencode(row["target"]),)).fetchone()
        if (not member.islnk() or member.linkname != row["target"] or target is None
                or target[0] >= seq or not target[2]):
            raise ValueError("Archive hardlink lost its earlier verified target")
        original = json.loads(target[1])
        if original["kind"] != "file" or original["sha256"] != row["sha256"]:
            raise ValueError("Archive hardlink bytes differ from its verified target")
    elif kind == "symlink":
        if not member.issym() or member.linkname != row["target"]:
            raise ValueError("Archive symlink target changed")
    elif kind != "directory" or not member.isdir():
        raise ValueError("Archive member type changed")


def verify_chunk(path, database, first, last):
    # r|gz reads one member at a time. Clearing members also bounds Python 3.12,
    # whose TarFile has no stream=True switch; never call getmembers/getmember.
    with tarfile.open(path, "r|gz") as saved:
        for seq, payload in database.execute(
                "SELECT seq,payload FROM entries WHERE seq>=? AND seq<=? ORDER BY seq", (first, last)):
            row = json.loads(payload)
            member = saved.next()
            if member is None or member.name.rstrip("/") != row["path"]:
                raise ValueError("Archive member name/order differs from its source index")
            if (member.mode, member.uid, member.gid, member.pax_headers.get("mtime")) != (
                    stat.S_IMODE(row["fingerprint"][5]), row["fingerprint"][6],
                    row["fingerprint"][7], row["mtime"]):
                raise ValueError("Archive member metadata differs from its source index")
            verify_member(saved, member, row, database, seq)
            database.execute("UPDATE entries SET verified=1 WHERE seq=?", (seq,))
            saved.members.clear()
        if saved.next() is not None:
            raise ValueError("Archive has unindexed trailing members")
    database.commit()


def verify_source(root, folder, database):
    if cleanup_roots(root, [folder]) != [folder] or active_artifacts(root):
        raise ValueError("Archive source became pinned or active")
    count = 0
    for path, info, parent_fd in walk(folder):
        stored = database.execute("SELECT payload,verified FROM entries WHERE path=?",
                                  (os.fsencode(path.relative_to(root).as_posix()),)).fetchone()
        if stored is None or not stored[1]:
            raise ValueError("Archive source has a new or unverified entry")
        row = json.loads(stored[0])
        if path.name in {".keep-artifacts", ".git"} or identity(info) != row["fingerprint"]:
            raise ValueError("Archive source changed before removal")
        if row["kind"] == "symlink" and os.readlink(path.name, dir_fd=parent_fd) != row["target"]:
            raise ValueError("Archive source symlink changed before removal")
        count += 1
    if count != database.execute("SELECT count(*) FROM entries").fetchone()[0]:
        raise ValueError("Archive source lost indexed entries before removal")


def write_chunk(root, destination, stem, number, entries, database, first, limit, byte_limit):
    path = destination / f"{stem}-{number:06d}.tar.gz"
    index = path.with_suffix(".members.jsonl")
    count = size = 0
    index_digest = hashlib.sha256()
    with tarfile.open(path, "x:gz", compresslevel=1) as output, index.open("x") as manifest:
        for source, info, parent_fd in entries:
            row = add_member(output, database, root, source, info, parent_fd)
            payload = json.dumps(row, separators=(",", ":"))
            database.execute("INSERT INTO entries(seq,path,payload) VALUES (?,?,?)",
                             (first + count, os.fsencode(row["path"]), payload))
            manifest.write(payload + "\n")
            index_digest.update((payload + "\n").encode("utf-8"))
            count += 1
            size += info.st_size if row["kind"] == "file" else 0
            if count % 256 == 0:
                database.commit()
            if count >= limit or size >= byte_limit:
                break
        manifest.flush()
        os.fsync(manifest.fileno())
    database.commit()
    verify_chunk(path, database, first, first + count - 1)
    if file_digest(index) != index_digest.hexdigest():
        raise ValueError("Archive member index changed after writing")
    return count, {"archive": str(path.relative_to(root)), "sha256": file_digest(path),
                   "members": count, "payload_bytes": size,
                   "members_index": str(index.relative_to(root)), "index_sha256": index_digest.hexdigest(),
                   "archive_fingerprint": identity(path.lstat()),
                   "index_fingerprint": identity(index.lstat()),
                   "restore": ["tar", "-xzf", str(path.relative_to(root)), "-C", "."]}


def verify_parts(root, parts_path, expected):
    if file_digest(parts_path) != expected:
        raise ValueError("Archive restore index changed after writing")
    with parts_path.open() as parts:
        for line in parts:
            part = json.loads(line)
            if (identity((root / part["archive"]).lstat()) != part["archive_fingerprint"]
                    or identity((root / part["members_index"]).lstat()) != part["index_fingerprint"]):
                raise ValueError("Verified archive or member index changed before removal")


def archive_root(root, folder, chunk_members, chunk_bytes):
    destination = root / "obj/archives"
    if destination.is_symlink():
        raise ValueError("Archive directory cannot be a symlink")
    destination.mkdir(exist_ok=True)
    stem = f"scratch-{time.time_ns()}"
    receipt_path = destination / f"{stem}.json"
    parts_path = destination / f"{stem}.parts.jsonl"
    spool = destination / f"{stem}.work.sqlite"
    receipt = {"state": "writing", "directories": [str(folder.relative_to(root))],
               "parts_index": str(parts_path.relative_to(root)), "parts": 0,
               "restore": "Run each parts_index restore command in line order, in the checkout. "
                          "Earlier chunks contain hardlink targets used by later chunks."}
    save(receipt_path, receipt)
    database = connect(spool)
    entries = walk(folder)
    parts_digest = hashlib.sha256()
    try:
        with parts_path.open("x") as parts:
            first = 0
            while True:
                entry = next(entries, None)
                if entry is None:
                    break
                count, part = write_chunk(root, destination, stem, receipt["parts"],
                                          itertools.chain((entry,), entries), database, first,
                                          chunk_members, chunk_bytes)
                line = json.dumps(part) + "\n"
                parts.write(line)
                parts_digest.update(line.encode("utf-8"))
                parts.flush()
                os.fsync(parts.fileno())
                receipt["parts"] += 1
                first += count
                save(receipt_path, receipt)
        verify_source(root, folder, database)
        verify_parts(root, parts_path, parts_digest.hexdigest())
        receipt.update(state="verified", members=first, parts_index_sha256=parts_digest.hexdigest())
        save(receipt_path, receipt)
        # Closed roots must stay inactive. The checkout lock and final mutation
        # walk retain the existing no-concurrent-writer archival contract.
        shutil.rmtree(folder)
        receipt["state"] = "archived"
        save(receipt_path, receipt)
    except BaseException as error:
        receipt.update(state="failed", error=str(error))
        save(receipt_path, receipt)
        raise
    finally:
        entries.close()
        database.close()
    spool.unlink()
    print(f"Archived closed root in {receipt['parts']} verified chunks: {receipt_path}", flush=True)
