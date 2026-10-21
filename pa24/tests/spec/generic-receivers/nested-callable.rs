fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn id(x:&u8)->&u8{x}fn take<T:Copy>(x:T){let f=move|p:fn(&u8)->&u8|{let _v=x;let n=7;check(*p(&n)==7);};fn run<F:FnOnce(fn(&u8)->&u8)>(f:F){f(id)}run(f)}fn main(){take(3u8);}
