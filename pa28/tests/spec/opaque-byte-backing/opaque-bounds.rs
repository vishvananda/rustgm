const fn make()->impl Copy {7u8}
const N:u8={let x=make();unsafe{*(&raw const x as *const u8).add(1)}};
fn main(){let _=N;}
