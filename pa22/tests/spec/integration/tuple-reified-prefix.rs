fn read(p: &(u16, [u8])) -> u16 {
    p.0
}
fn main() {
    let _p: fn(&(u16, [u8])) -> u16 = read;
}
