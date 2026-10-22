const fn hide<'a>(r:&'a u8)->impl Copy+use<'a>{r}
const N:u8={let mut x=7u8;let h=hide(&x);let p=unsafe{*(&raw const h as *const *mut u8)};unsafe{*p=9;}x};
fn main(){let _=N;}
