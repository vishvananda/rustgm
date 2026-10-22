fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:usize=7;fn main(){let x:[u8;match 7usize{N=>1,_=>2}]=[9];check(x[0]==9);}