fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pass<U>(f:fn(U))->fn(U){f}
fn take(x:(&u8,&u8)){check(*x.0==19);check(*x.1==73)}
fn relay<'a,'b>(x:&'a u8,y:&'b u8){let f=pass::<(_,_)>(take);f((x,y));}
fn main(){let x=19;let y=73;relay(&x,&y);}

