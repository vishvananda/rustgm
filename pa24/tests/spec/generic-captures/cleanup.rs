fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut COUNT:u8=0;struct Token;impl Drop for Token{fn drop(&mut self){unsafe{COUNT+=1;}}}fn take<T>(x:T){let f=move||{let _v=x;};f()}fn main(){take(Token);check(unsafe{COUNT}==1);}
