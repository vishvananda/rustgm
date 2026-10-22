const fn make()->impl Copy{[7u8;1]}
const N:u8={let mut x=make();let p=&raw mut x as *mut [u8;2];unsafe{(*p)[1]}};
fn main(){let _=N;}
