fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:for<'a>Fn(&'a u8)->u8{}impl<F:for<'a>Fn(&'a u8)->u8>Apply for F{}fn run(f:&dyn Apply)->u8{f(&0)}fn main(){let n=7;let a=|_:&u8|n;let m=8;let b=|_:&u8|m;check(run(&a)==7);check(run(&b)==8);check(run(&a)==7);}
