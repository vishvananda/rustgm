fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[u8;2]=[2,7];fn f(x:[u8;2])->u8{match x{C=>1,_=>2}}fn main(){check(f([2,7])==1&&f([7,2])==2&&f([2,2])==2&&f([7,7])==2);}