static X:u8=41;const fn make()->impl Copy{&X}
const N:u8={let x=make();unsafe{**(&raw const x as *const &u8)}};
fn main(){if N!=41{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
