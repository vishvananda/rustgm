fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:Fn(u8)->u8{}fn run(_: &dyn Apply){}fn main(){let n=4;let f=|x|x+n;run(&f);}
