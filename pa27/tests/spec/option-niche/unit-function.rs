fn f(){}union U{o:Option<fn()>,p:fn()}const N:Option<fn()>=unsafe{(U{p:f}).o};fn main(){check(match N{Some(f)=>{f();true},None=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
