fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T:Copy>(x:T)->T{x}fn main(){let p:&dyn Fn(u16)->u16=&id::<u16>;check(p(700)==700);}
