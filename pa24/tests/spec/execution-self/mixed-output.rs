fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<'a,F:for<'b>Fn(&'b u8)->(&'b u8,&'a u8)>(f:F)->F{f}fn main(){let n=7u8;let r=&n;let f=infer(move|x|(x,r));let q=3;let (x,y)=f(&q);check(*x==3&&*y==7);}
