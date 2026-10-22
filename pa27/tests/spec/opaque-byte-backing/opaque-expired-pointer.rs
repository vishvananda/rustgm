const fn make()->impl Copy{let x=7u8;&raw const x}
const N:u8={let h=make();let p=unsafe{*(&raw const h as *const *const u8)};unsafe{*p}};
fn main(){let _=N;}
