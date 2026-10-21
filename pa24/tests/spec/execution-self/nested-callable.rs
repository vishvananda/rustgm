fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(x:&u8)->&u8{x}fn infer<'a,F:Fn()->(&'a u8,fn(&u8)->&u8)>(f:F)->F{f}fn main(){let n=7u8;let r=&n;let p:fn(&u8)->&u8=id;let f=infer(move||(r,p));let (x,g)=f();check(*g(x)==7);}
