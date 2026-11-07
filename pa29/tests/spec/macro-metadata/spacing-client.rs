use spacing_provider::spelled;
fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    check(match spelled!(0) { "a += b" => true, _ => false });
    check(match spelled!(1) { "a + = b" => true, _ => false });
    check(match spelled!(2) { "a += b" => true, _ => false });
    check(match spelled!(3) { "x < - y, x <- y" => true, _ => false });
    check(match spelled!(4) { "f(a)[b] {c}{ d }" => true, _ => false });
    check(match spelled!(5) { "& 'a ('b') r#\"s /* t */\"#" => true, _ => false });
    check(match spelled!(6) { "a - - b, a - - b" => true, _ => false });
    check(match spelled!(7) { "#[a] # ! [b] # ! [c]" => true, _ => false });
    check(match spelled!(8) { "a :: b, a :: b, a :: b" => true, _ => false });
}
