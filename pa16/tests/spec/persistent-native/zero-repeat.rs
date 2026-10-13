fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:[[(u64,u8);1000];1000]=[[(0,0);1000];1000];fn main(){check(A[999][999].0==0 && A[1][2].1==0);}
