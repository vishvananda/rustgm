fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn copy<T:Copy>(pair:(&T,&u8,&u8))->T{*pair.0}fn f<'a,T:Copy>(x:&'a T)->T{let c=|pair:(&T,&u8,_)|copy(pair);c((x,&7u8,&9u8))}fn main(){check(f(&7u8)==7);}
