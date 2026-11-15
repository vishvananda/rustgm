use std::io::{self, BufWriter, LineWriter, Write};

struct Celsius(f64);

impl std::fmt::Display for Celsius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}°C", self.0)
    }
}

fn main() -> io::Result<()> {
    println!("plain line");
    print!("no newline, ");
    print!("{} {:>5} {:?}\n", 1, "pad", Some('x'));
    println!();
    println!("{0} {0:#x} {0:#b} {name}", 255, name = "named");
    let text = format!("{:08.3}", 3.14159);
    println!("{text} {} {}", text.len(), Celsius(21.55));
    println!("{:?} {:?}", vec![(1, "a"), (2, "b")], String::from("owned"));
    eprintln!("this line goes to standard error");
    eprint!("so does this, ");
    eprintln!("without interleaving into standard output");

    let stdout = io::stdout();
    let mut lock = stdout.lock();
    writeln!(lock, "locked {}", 1)?;
    lock.write_all(b"raw bytes\n")?;
    write!(lock, "partial")?;
    lock.flush()?;
    drop(lock);
    println!(" line completed");

    let mut buffered = BufWriter::new(io::stdout());
    for index in 0..5 {
        writeln!(buffered, "buffered {index}")?;
    }
    println!("printed before the buffer is flushed");
    buffered.flush()?;
    let mut lines = LineWriter::new(io::stdout());
    lines.write_all(b"line writer ")?;
    println!("<direct>");
    lines.write_all(b"second half\n")?;
    drop(lines);

    let long = "0123456789".repeat(500);
    println!("{}", long);
    println!("{}", long.len());
    let mut vectored = io::stdout();
    let written = vectored.write_vectored(&[io::IoSlice::new(b"vec"), io::IoSlice::new(b"tored\n")])?;
    println!("vectored wrote {}", written > 0);
    for row in 1..=3 {
        for column in 1..=3 {
            print!("{:>4}", row * column);
        }
        println!();
    }
    io::stdout().write_all("unicode: ∑ 日本語 😀\n".as_bytes())?;
    print!("final line without a newline");
    Ok(())
}
