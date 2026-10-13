fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed(2))]struct P{a:u8,b:u16}fn f(x:&mut u16){*x+=1}fn main(){let mut p=P{a:3,b:41};f(&mut p.b);check(p.b==42 && p.a==3);}
