fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'b,'c,T>(x:&'c &'b (T,u8))->&'b (T,u8){*x}fn main(){let x=(7u8,9u8);let q=&x;let p=&q;check(f(&p).0==7);}
