fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let mut c=0;let v=match 1u8{n@(1|_) if {c+=1;c==2}=>n,_=>99};check(v==1&&c==2);}