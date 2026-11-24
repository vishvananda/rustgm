#[repr(C)]
pub struct Ops {
    pub message: fn(u8) -> u8,
    pub retry: fn(u8) -> bool,
}
impl Ops {
    const DEFAULT: &'static Ops = &Ops {
        message: |_| 34,
        retry: |_| false,
    };
}
pub const fn current() -> &'static Ops { Ops::DEFAULT }
pub fn invoke(value: u8) -> u8 {
    let ops = current();
    if (ops.retry)(value) { 1 } else { (ops.message)(value) }
}
