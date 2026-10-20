fn pointer(p: &(u8, (u16, [u8]))) -> *const [u8] {
    &raw const p.1.1
}
fn main() {
    let _p: fn(&(u8, (u16, [u8]))) -> *const [u8] = pointer;
}
