// unicode-ident: identifier start and continue classes, sampled and swept.
use unicode_ident::{is_xid_continue, is_xid_start};

fn class(c: char) -> &'static str {
    match (is_xid_start(c), is_xid_continue(c)) {
        (true, true) => "start",
        (false, true) => "continue",
        (false, false) => "neither",
        (true, false) => "start only",
    }
}

fn main() {
    for c in ['a', 'Z', '_', '0', '9', ' ', '-', '$', 'é', 'ß', 'Σ', 'д', '中', 'あ', '한', 'ا', '😀', '\u{301}', '\u{200d}', '٣', '²', 'ǅ', 'ⅷ', '℮', '·', '\u{37a}', '\u{e33}', '\0', '\u{10ffff}'] {
        println!("{:?} U+{:04X} {}", c, c as u32, class(c));
    }
    let ascii_start: String = (0u8..128).map(char::from).filter(|c| is_xid_start(*c)).collect();
    let ascii_continue: String = (0u8..128).map(char::from).filter(|c| is_xid_continue(*c) && !is_xid_start(*c)).collect();
    println!("ascii start {:?}", ascii_start);
    println!("ascii continue only {:?}", ascii_continue);

    let mut starts = 0u32;
    let mut continues = 0u32;
    let mut start_only = 0u32;
    let mut checksum = 0u64;
    let mut runs = 0u32;
    let mut previous = false;
    let mut last_start = None;
    for code in 0..=0x10ffffu32 {
        let Some(c) = char::from_u32(code) else {
            previous = false;
            continue;
        };
        let start = is_xid_start(c);
        let cont = is_xid_continue(c);
        starts += u32::from(start);
        continues += u32::from(cont);
        start_only += u32::from(start && !cont);
        if cont {
            checksum = checksum.wrapping_mul(0x100_0000_01b3).wrapping_add(u64::from(code) * 2 + u64::from(start));
        }
        if cont && !previous {
            runs += 1;
        }
        if start {
            last_start = Some(c);
        }
        previous = cont;
    }
    println!("sweep starts {} continues {} start-only {} runs {}", starts, continues, start_only, runs);
    println!("checksum {:016x} last start {:?}", checksum, last_start.map(|c| c as u32));
    for (name, range) in [("latin-1", 0x80..0x100u32), ("greek", 0x370..0x400), ("cjk", 0x4e00..0xa000), ("emoji", 0x1f600..0x1f650), ("astral letters", 0x20000..0x2a6e0)] {
        let total = range.clone().count();
        let start = range.clone().filter_map(char::from_u32).filter(|c| is_xid_start(*c)).count();
        let cont = range.filter_map(char::from_u32).filter(|c| is_xid_continue(*c)).count();
        println!("{} {} {} {}", name, total, start, cont);
    }
}
