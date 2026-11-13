fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a,T:Copy>(x:&'a T)->T{let c=|pair|copy(pair);c((x,&7u8))}fn copy<'a,T:Copy>(pair:(&'a T,&u8))->T{*pair.0}
fn main(){check(f(&7u8)==7);}
