fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&(u8,u8))->u8{let mut n=0;let v=match x{(a,_)|(_,a) if {n+=1;*a==2}=>*a,_=>0};n+v}fn main(){check(f(&(1,2))==4);}
