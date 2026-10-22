const fn make()->impl Copy {37u32}
const N:u8={let x=make();unsafe{*(&raw const x as *const u8)}};
fn main(){if N!=37{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
