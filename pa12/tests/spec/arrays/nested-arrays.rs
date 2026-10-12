fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
fn id(x:[[u16;3];2])->[[u16;3];2]{x} fn main(){let mut a=id([[1,2,3],[4,5,6]]);a[1][2]=70;check(a[1][2]==70 && a[0][2]==3);}
