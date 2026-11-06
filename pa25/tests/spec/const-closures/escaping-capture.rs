#![feature(const_closures, const_trait_impl, const_destruct)]
pub const fn make<'a>(value:&'a u8)->impl Fn()->&'static u8+'a {
    const move || value
}
