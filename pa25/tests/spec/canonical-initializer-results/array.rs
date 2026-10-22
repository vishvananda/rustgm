fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a>S<'a>{const C:[&'a u8;2]={let n=&7;[n,&8]};}fn main(){check(match [&7,&8]{S::<'static>::C=>true,_=>false});check(!match [&9,&8]{S::<'static>::C=>true,_=>false});}
