fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [[a,..,b],rest@..]=[[1u64,2,3],[4,5,6],[7,8,9]];check(a==1&&b==3&&rest[0][2]==6&&rest[1][0]==7);}