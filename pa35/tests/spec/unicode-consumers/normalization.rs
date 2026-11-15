// unicode-normalization: the four forms, quick checks and character queries.
use unicode_normalization::char::{canonical_combining_class, compose, decompose_canonical, decompose_compatible, is_combining_mark};
use unicode_normalization::{
    IsNormalized, UnicodeNormalization, is_nfc, is_nfc_quick, is_nfd, is_nfd_quick, is_nfkc, is_nfkd,
};

fn codes(text: &str) -> String {
    text.chars().map(|c| format!("{:04X}", c as u32)).collect::<Vec<_>>().join(" ")
}

fn forms(label: &str, text: &str) {
    let nfc: String = text.nfc().collect();
    let nfd: String = text.nfd().collect();
    let nfkc: String = text.nfkc().collect();
    let nfkd: String = text.nfkd().collect();
    println!("{label}: {:?} [{}]", text, codes(text));
    println!("  nfc  {:?} [{}] {}", nfc, codes(&nfc), is_nfc(text));
    println!("  nfd  {:?} [{}] {}", nfd, codes(&nfd), is_nfd(text));
    println!("  nfkc {:?} [{}] {}", nfkc, codes(&nfkc), is_nfkc(text));
    println!("  nfkd {:?} [{}] {}", nfkd, codes(&nfkd), is_nfkd(text));
    let stable = nfc.nfc().eq(nfc.chars()) && nfd.nfd().eq(nfd.chars()) && nfd.nfc().eq(nfc.chars()) && nfc.nfd().eq(nfd.chars());
    println!("  round trips {}", stable);
}

fn quick(result: IsNormalized) -> &'static str {
    match result {
        IsNormalized::Yes => "yes",
        IsNormalized::No => "no",
        IsNormalized::Maybe => "maybe",
    }
}

fn main() {
    forms("ascii", "plain ASCII text");
    forms("precomposed", "é Å ñ ü");
    forms("decomposed", "e\u{301} A\u{30a} n\u{303}");
    forms("ordering", "q\u{307}\u{323} a\u{315}\u{300}\u{5ae}\u{300}b");
    forms("hangul", "한글 \u{1112}\u{1161}\u{11ab} 각");
    forms("compatibility", "ﬁ ² ㎏ Ⅷ ｶ ﾞ ℌ ①");
    forms("singletons", "Ω K Å \u{212b} \u{2126}");
    forms("exclusions", "\u{958} \u{fb1d} \u{2adc}");
    forms("mixed", "Åström ﬁancé 한국어 x\u{303}\u{304}");
    forms("empty", "");

    for text in ["abc", "é", "e\u{301}", "\u{1100}\u{1161}", "ﬁ", "a\u{300}\u{301}", "\u{301}"] {
        println!("quick {:?} nfc {} nfd {}", text, quick(is_nfc_quick(text.chars())), quick(is_nfd_quick(text.chars())));
    }
    for c in ['a', '\u{301}', '\u{323}', '\u{5ae}', 'é', '\u{1161}', '\u{20dd}'] {
        println!("char U+{:04X} class {} mark {}", c as u32, canonical_combining_class(c), is_combining_mark(c));
    }
    println!(
        "compose {:?} {:?} {:?} {:?}",
        compose('e', '\u{301}'),
        compose('a', 'b'),
        compose('\u{1100}', '\u{1161}'),
        compose('\u{ac00}', '\u{11a8}')
    );
    for c in ['é', 'ǆ', '각', 'ﬃ', 'ẛ'] {
        let mut canonical = String::new();
        let mut compatible = String::new();
        decompose_canonical(c, |part| canonical.push(part));
        decompose_compatible(c, |part| compatible.push(part));
        println!("decompose {:?} [{}] [{}]", c, codes(&canonical), codes(&compatible));
    }

    let mut changed_nfc = 0u32;
    let mut changed_nfd = 0u32;
    let mut changed_nfkc = 0u32;
    let mut longest = (0usize, 0u32);
    let mut checksum = 0u64;
    for code in 0..0x30000u32 {
        let Some(c) = char::from_u32(code) else { continue };
        let mut buffer = [0u8; 4];
        let text: &str = c.encode_utf8(&mut buffer);
        changed_nfc += u32::from(!text.nfc().eq(text.chars()));
        changed_nfkc += u32::from(!text.nfkc().eq(text.chars()));
        let mut length = 0;
        for part in text.nfd() {
            length += 1;
            checksum = checksum.wrapping_mul(0x100_0000_01b3).wrapping_add(u64::from(part as u32));
        }
        changed_nfd += u32::from(length != 1 || !text.nfd().eq(text.chars()));
        if length > longest.0 {
            longest = (length, code);
        }
    }
    println!("sweep nfc {} nfd {} nfkc {} longest {:?} checksum {:016x}", changed_nfc, changed_nfd, changed_nfkc, longest, checksum);
    let stream: String = "a\u{300}".repeat(3).chars().cycle().take(40).stream_safe().collect();
    let marks: String = std::iter::once('a').chain(std::iter::repeat('\u{300}').take(40)).stream_safe().collect();
    println!("stream safe {} {} {:?}", stream.chars().count(), marks.chars().count(), marks.chars().position(|c| c == '\u{34f}'));
    let cjk: String = "\u{2f800}\u{2f801}".cjk_compat_variants().collect();
    println!("cjk variants [{}]", codes(&cjk));
}
