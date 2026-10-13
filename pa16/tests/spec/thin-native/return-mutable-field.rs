fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn update<'a>(p:&'a mut(u64,u64),q:&(u64,u64))->&'a mut u64{let(a,b)=p;*a+=q.0;*b+=q.1;a}fn main(){let mut p=(1,2);let q=(3,4);let r=update(&mut p,&q);let n=*r;check(n+p.1==10);}
