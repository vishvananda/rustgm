fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:Fn(u8)->u8{}impl<F:Fn(u8)->u8>Apply for F{}fn run(_: &dyn Apply){}fn main(){let n=4u16;let f=|x|{let _:u8=x;n};run(&f);}
