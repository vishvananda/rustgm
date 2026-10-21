fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut X:[u8;2]=[3,5];fn main(){unsafe{X[1]=9;check(X[0]==3);check(X[1]==9);}}