const fn make<T:Copy>(x:T)->impl Copy {x}
const A:u8={let x=make(17u8);unsafe{*(&raw const x as *const u8)}};
const B:u8={let x=make(31u16);unsafe{*(&raw const x as *const u8)}};
fn main(){if A!=17||B!=31{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
