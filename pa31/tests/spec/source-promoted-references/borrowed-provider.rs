#[repr(C)]
pub struct Ops {
    pub message: fn(u8, &mut Arguments<'_>) -> Opt<u8>,
    pub retry: fn(u8) -> bool,
}
impl Ops {
    const DEFAULT: &'static Ops = &Ops {
        message: |_, _| Opt::Some(34),
        retry: |_| false,
    };
}
pub const fn current() -> &'static Ops { Ops::DEFAULT }
