fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<T:Copy>(n:T)->u8{let f=move|x:&u8|{let y=n;let _=y;*x};let p:&dyn Fn(&u8)->u8=&f;p(&7)}fn main(){check(run(1u16)==7);}
