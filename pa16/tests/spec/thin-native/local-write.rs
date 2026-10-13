fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add(x:&mut u64,y:&u64){*x+=*y}fn main(){let mut x=40;let y=2;add(&mut x,&y);check(x==42);}
