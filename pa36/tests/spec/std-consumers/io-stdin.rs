use std::io::{self, BufRead, BufReader, Cursor, ErrorKind, Read, Write};

fn checksum(mut reader: impl Read) -> io::Result<(usize, u32)> {
    let mut buffer = [0u8; 7];
    let mut total = 0;
    let mut sum = 0u32;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok((total, sum));
        }
        total += read;
        sum = buffer[..read].iter().fold(sum, |acc, byte| acc.wrapping_mul(31).wrapping_add(u32::from(*byte)));
    }
}

struct Upper<W: Write>(W);

impl<W: Write> Write for Upper<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write_all(&bytes.to_ascii_uppercase())?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut first = String::new();
    let read = stdin.lock().read_line(&mut first)?;
    let numbers: Vec<i64> = first.split_whitespace().filter_map(|field| field.parse().ok()).collect();
    println!("first line {:?} {} numbers {:?} sum {}", first, read, numbers, numbers.iter().sum::<i64>());
    let mut second = String::new();
    stdin.read_line(&mut second)?;
    println!("second line {:?} words {}", second.trim_end(), second.split_whitespace().count());
    let mut rest = Vec::new();
    for line in stdin.lock().lines() {
        rest.push(line?);
    }
    println!("rest {:?}", rest);
    let mut after = String::new();
    println!("at end {} {:?}", stdin.read_line(&mut after)?, after);
    let mut remaining = Vec::new();
    println!("read_to_end {}", io::stdin().read_to_end(&mut remaining)?);

    let mut cursor = Cursor::new(b"alpha\nbeta\r\ngamma".to_vec());
    let mut line = String::new();
    while cursor.read_line(&mut line)? > 0 {
        print!("[{}]", line.trim_end());
        line.clear();
    }
    println!(" position {}", cursor.position());
    cursor.set_position(6);
    let mut four = [0u8; 4];
    cursor.read_exact(&mut four)?;
    println!("read_exact {:?} {:?}", std::str::from_utf8(&four), cursor.read_exact(&mut [0u8; 64]).map_err(|error| error.kind()));
    let split: Vec<String> = Cursor::new("a,b,,c").split(b',').map(|field| field.map(|bytes| String::from_utf8_lossy(&bytes).into_owned())).collect::<io::Result<_>>()?;
    println!("split {:?}", split);

    let data: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
    println!("checksum {:?} {:?}", checksum(&data[..])?, checksum(BufReader::with_capacity(16, &data[..]))?);
    println!("take {:?} chain {:?}", checksum((&data[..]).take(10))?, checksum((&b"ab"[..]).chain(&b"cd"[..]))?);
    let bytes: io::Result<Vec<u8>> = (&data[250..260]).bytes().collect();
    println!("bytes {:?}", bytes?);
    let mut text = String::new();
    println!("read_to_string {:?} {}", (&b"caf\xc3\xa9"[..]).read_to_string(&mut text), text);
    let invalid = (&b"\xff\xfe"[..]).read_to_string(&mut String::new()).map_err(|error| error.kind());
    println!("invalid {:?}", invalid);
    println!("empty {:?} repeat {:?}", checksum(io::empty())?, checksum(io::repeat(7).take(20))?);

    let mut sink: Vec<u8> = Vec::new();
    write!(sink, "{}-{:03}", "vec", 7)?;
    sink.write_all(b" bytes")?;
    let copied = io::copy(&mut &data[..100], &mut sink)?;
    println!("vec writer {} {} {:?}", copied, sink.len(), String::from_utf8_lossy(&sink[..13]));
    let mut fixed = [0u8; 8];
    let mut slice: &mut [u8] = &mut fixed;
    let outcome = slice.write_all(b"too long for it").map_err(|error| error.kind());
    println!("slice writer {:?} {:?}", outcome, fixed);
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_all(b"hello world")?;
    cursor.set_position(6);
    cursor.write_all(b"there!")?;
    println!("cursor writer {:?}", String::from_utf8(cursor.into_inner()));
    let mut upper = Upper(io::stdout());
    writeln!(upper, "written through an adapter: {}", "mixed Case")?;
    upper.flush()?;
    println!("discarded {}", io::copy(&mut &data[..], &mut io::sink())?);

    let custom = io::Error::new(ErrorKind::InvalidData, "custom failure");
    println!("error {} | {:?} | {:?}", custom, custom.kind(), custom.get_ref().map(|inner| inner.to_string()));
    let from_os = io::Error::from_raw_os_error(2);
    println!("os error {} | {:?} | {:?}", from_os, from_os.kind(), from_os.raw_os_error());
    let simple = io::Error::from(ErrorKind::UnexpectedEof);
    println!("simple {} | {:?} | {}", simple, simple, ErrorKind::PermissionDenied);
    let other = io::Error::other(format!("formatted {}", 5));
    println!("other {} {:?}", other, other.kind());
    Ok(())
}
