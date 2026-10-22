unsafe extern "C"{fn sum(n:u32,...)->i64;}
extern "C" fn f(){}fn main(){let _=unsafe{sum(1,f)};}
