fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn sum(x0:i128,x1:i128,x2:i128,x3:i128,x4:i128,x5:i128,x6:i128,x7:i128,x8:i128,x9:i128,x10:i128,x11:i128)->i128{x0+x1+x2+x3+x4+x5+x6+x7+x8+x9+x10+x11} fn main(){check(sum(0,1,2,3,4,5,6,7,8,9,10,11)==66);}
