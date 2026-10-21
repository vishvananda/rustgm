fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}static mut COUNT:u8=0;struct Token;impl Drop for Token{fn drop(&mut self){unsafe{COUNT+=1;}}}fn take<T>(x:T){let f=move||{let _=&x;};once(f)}fn main(){take(Token);check(unsafe{COUNT}==1);}
