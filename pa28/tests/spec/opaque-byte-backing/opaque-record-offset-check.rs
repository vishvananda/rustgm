#[derive(Clone,Copy)] #[repr(C)] struct Pair{a:u8,b:u8}
const fn make()->impl Copy {Pair{a:13,b:29}}
const N:u8={let x=make();unsafe{*(&raw const x as *const u8).add(1)}};
fn main(){if N!=29{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
