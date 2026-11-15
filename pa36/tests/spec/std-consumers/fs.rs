use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

fn names(directory: &Path) -> io::Result<Vec<String>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        found.push(format!("{}{}", entry.file_name().to_string_lossy(), if kind.is_dir() { "/" } else { "" }));
    }
    found.sort();
    Ok(found)
}

fn main() -> io::Result<()> {
    let root = Path::new("consumer-scratch");
    println!("exists {} {:?}", root.exists(), fs::metadata(root).map_err(|error| error.kind()).err());
    fs::create_dir(root)?;
    println!("create_dir {} {:?}", root.is_dir(), fs::create_dir(root).map_err(|error| error.kind()));
    fs::create_dir_all(root.join("a/b/c"))?;

    let greeting = root.join("greeting.txt");
    fs::write(&greeting, "hello, file\nsecond line\n")?;
    let text = fs::read_to_string(&greeting)?;
    println!("read_to_string {:?} {}", text, text.lines().count());
    let metadata = fs::metadata(&greeting)?;
    println!("metadata {} {} {} {}", metadata.len(), metadata.is_file(), metadata.is_dir(), metadata.permissions().readonly());

    let mut file = File::create(root.join("numbers.bin"))?;
    for value in 0u32..100 {
        file.write_all(&(value * value).to_le_bytes())?;
    }
    file.sync_all()?;
    drop(file);
    let bytes = fs::read(root.join("numbers.bin"))?;
    let decoded: Vec<u32> = bytes.chunks_exact(4).map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();
    println!("binary {} {} {:?}", bytes.len(), decoded.iter().sum::<u32>(), &decoded[97..]);

    let mut append = OpenOptions::new().append(true).open(&greeting)?;
    writeln!(append, "appended {}", 3)?;
    drop(append);
    let reader = BufReader::new(File::open(&greeting)?);
    for (number, line) in reader.lines().enumerate() {
        println!("line {}: {}", number + 1, line?);
    }
    println!("create_new {:?}", OpenOptions::new().write(true).create_new(true).open(&greeting).map(|_| ()).map_err(|error| error.kind()));

    let mut random = OpenOptions::new().read(true).write(true).open(root.join("numbers.bin"))?;
    let mut word = [0u8; 4];
    random.seek(SeekFrom::Start(40))?;
    random.read_exact(&mut word)?;
    let at_ten = u32::from_le_bytes(word);
    let position = random.seek(SeekFrom::End(-4))?;
    random.read_exact(&mut word)?;
    println!("seek {} {} {} {}", at_ten, position, u32::from_le_bytes(word), random.stream_position()?);
    random.seek(SeekFrom::Current(-8))?;
    random.write_all(&7u32.to_le_bytes())?;
    random.rewind()?;
    let mut all = Vec::new();
    let read = random.read_to_end(&mut all)?;
    println!("rewrite {} {:?} {:?}", read, &all[392..396], random.read(&mut word)?);
    random.set_len(16)?;
    println!("truncate {}", random.metadata()?.len());
    drop(random);

    let mut writer = BufWriter::new(File::create(root.join("a/b/table.csv"))?);
    for row in 1..=200 {
        writeln!(writer, "{},{},{}", row, row * row, if row % 2 == 0 { "even" } else { "odd" })?;
    }
    writer.flush()?;
    drop(writer);
    let table = BufReader::with_capacity(64, File::open(root.join("a/b/table.csv"))?);
    let mut total = 0u64;
    let mut evens = 0;
    for line in table.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();
        total += fields[1].parse::<u64>().unwrap_or(0);
        evens += usize::from(fields[2] == "even");
    }
    println!("table {} {} {}", total, evens, fs::metadata(root.join("a/b/table.csv"))?.len());

    let copied = fs::copy(&greeting, root.join("a/copy.txt"))?;
    fs::rename(root.join("a/copy.txt"), root.join("a/b/renamed.txt"))?;
    println!("copy {} {} {}", copied, root.join("a/copy.txt").exists(), fs::read_to_string(root.join("a/b/renamed.txt"))? == fs::read_to_string(&greeting)?);
    println!("read_dir {:?} {:?} {:?}", names(root)?, names(&root.join("a"))?, names(&root.join("a/b"))?);
    let canonical = fs::canonicalize(root.join("a/b/../b/./renamed.txt"))?;
    println!("canonical {} {:?} {:?}", canonical.is_absolute(), canonical.file_name(), canonical.parent().and_then(Path::file_name));

    let missing = File::open(root.join("missing.txt")).map(|_| ());
    match &missing {
        Ok(()) => println!("unexpectedly opened"),
        Err(error) => println!("missing {:?} {} {:?}", error.kind(), error, error.raw_os_error()),
    }
    println!("errors {:?} {:?}", fs::read_to_string(root).map_err(|error| error.kind()), fs::remove_dir(root).map_err(|error| error.kind()));
    println!("errors {:?} {:?}", fs::read_dir(&greeting).map(|_| ()).map_err(|error| error.kind()), fs::remove_file(root.join("a")).map_err(|error| error.kind() == ErrorKind::NotFound));
    fs::write(root.join("invalid.txt"), [0xff, 0xfe, b'a'])?;
    println!("invalid {:?} {:?}", fs::read_to_string(root.join("invalid.txt")).map_err(|error| error.kind()), fs::read(root.join("invalid.txt"))?);
    let empty = File::create(root.join("empty"))?;
    println!("empty {} {:?}", empty.metadata()?.len(), fs::read_to_string(root.join("empty")));

    fs::remove_file(root.join("a/b/renamed.txt"))?;
    fs::remove_dir(root.join("a/b/c"))?;
    println!("remove_dir {:?}", fs::remove_dir(root.join("a/b")).map_err(|error| error.kind()));
    fs::remove_dir_all(root)?;
    println!("removed {} {:?}", !root.exists(), fs::remove_dir_all(root).map_err(|error| error.kind()));
    Ok(())
}
