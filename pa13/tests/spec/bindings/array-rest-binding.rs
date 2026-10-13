fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [a,middle@..,b]=[2u64,3,5,7];check(a==2&&middle[0]==3&&middle[1]==5&&b==7);}