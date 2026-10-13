fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let x=7u8;let v=match x{mut n if n>0=>{n+=1;n},_=>99};check(v==8&&x==7);}