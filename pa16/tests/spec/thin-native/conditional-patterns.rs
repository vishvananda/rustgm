fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&(u8,u8))->u8{let mut n=0;if let (a,b)=x && *a==1{n+=*a+*b;}let (a,b)=x else{loop{}};n+=*a+*b;while let (a,b)=x && true{n+=*a+*b;break;}n}fn main(){check(f(&(1,2))==9);}
