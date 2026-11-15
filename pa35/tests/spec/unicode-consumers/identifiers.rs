// The way a compiler front end uses both crates: split text into identifier
// candidates, validate them with XID classes and intern them in NFC.
use std::borrow::Cow;
use std::collections::BTreeMap;

use unicode_normalization::UnicodeNormalization;

fn is_start(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_start(c)
}

fn is_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c)
}

fn canonical(spelling: &str) -> Cow<'_, str> {
    if spelling.is_ascii() || unicode_normalization::is_nfc(spelling) {
        Cow::Borrowed(spelling)
    } else {
        Cow::Owned(spelling.nfc().collect::<String>())
    }
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Identifier(Cow<'a, str>),
    Number(&'a str),
    Punctuation(char),
    Invalid(&'a str),
}

fn lex(source: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut rest = source;
    while let Some(first) = rest.chars().next() {
        if first.is_whitespace() {
            rest = &rest[first.len_utf8()..];
        } else if is_start(first) {
            let end = rest.char_indices().find(|(_, c)| !is_continue(*c)).map_or(rest.len(), |(index, _)| index);
            tokens.push(Token::Identifier(canonical(&rest[..end])));
            rest = &rest[end..];
        } else if first.is_ascii_digit() {
            let end = rest.char_indices().find(|(_, c)| !is_continue(*c)).map_or(rest.len(), |(index, _)| index);
            tokens.push(Token::Number(&rest[..end]));
            rest = &rest[end..];
        } else if is_continue(first) || !first.is_ascii() {
            let end = rest.char_indices().find(|(_, c)| c.is_whitespace() || c.is_ascii_punctuation()).map_or(rest.len(), |(index, _)| index);
            tokens.push(Token::Invalid(&rest[..end]));
            rest = &rest[end..];
        } else {
            tokens.push(Token::Punctuation(first));
            rest = &rest[1..];
        }
    }
    tokens
}

fn valid_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters.next().is_some_and(is_start) && characters.all(is_continue)
}

fn main() {
    let source = "let café = naïve_count + 東京2 * _tmp; fn Ångström(e\u{301}t\u{e9}: u8) { ﬁx = 1; } 9lives \u{301}bad ２ x·y";
    let tokens = lex(source);
    for token in &tokens {
        println!("{:?}", token);
    }
    let mut interned: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for token in &tokens {
        if let Token::Identifier(name) = token {
            let entry = interned.entry(name.to_string()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += usize::from(matches!(name, Cow::Owned(_)));
        }
    }
    println!("interned {:?}", interned);
    let decomposed = "cafe\u{301}";
    let precomposed = "caf\u{e9}";
    println!(
        "same identifier {} {} {:?} {:?}",
        decomposed == precomposed,
        canonical(decomposed) == canonical(precomposed),
        matches!(canonical(decomposed), Cow::Owned(_)),
        matches!(canonical(precomposed), Cow::Borrowed(_))
    );
    for candidate in ["x", "_", "__init__", "été", "e\u{301}", "\u{301}e", "2x", "x2", "a-b", "", "ℌello", "ｆｕｌｌ", "a\u{200d}b", "🦀", "r#type", "Ⅷ", "ﬁ"] {
        let nfc = canonical(candidate);
        let nfkc: String = candidate.nfkc().collect();
        println!("{:?} valid {} nfc-valid {} nfkc {:?} nfkc-valid {}", candidate, valid_identifier(candidate), valid_identifier(&nfc), nfkc, valid_identifier(&nfkc));
    }
    let mut lengths: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for word in "Ünïcödé wörds größe ÅNGSTRÖM naïve façade ǅungla".split(' ') {
        let canonical_word: String = word.nfd().collect::<String>().nfc().collect();
        lengths.entry(canonical_word.chars().count()).or_default().push(canonical_word);
    }
    println!("lengths {:?}", lengths);
}
