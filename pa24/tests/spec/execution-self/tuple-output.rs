fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<'a,'b,F:Fn()->(&'a u8,&'b u8)>(f:F)->F{f}fn main(){let a=3u8;let b=7u8;let r=&a;let s=&b;let f=infer(move||(r,s));let (x,y)=f();check(*x==3&&*y==7);}
