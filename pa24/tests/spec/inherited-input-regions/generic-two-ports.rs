fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'b,'c,'d,'e,T>(x:&'c &'b T,y:&'e &'d T)->(&'b T,&'d T){let c=|&s,&t|(s,t);c(x,y)}
fn main(){let x=7u8;let y=9u8;let p=&x;let q=&y;let(a,b)=f(&p,&q);check(*a==7&&*b==9);}
