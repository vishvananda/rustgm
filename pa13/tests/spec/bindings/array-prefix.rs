fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [prefix@..,z]=[2u8,3,5];check(prefix[0]==2&&prefix[1]==3&&z==5);}