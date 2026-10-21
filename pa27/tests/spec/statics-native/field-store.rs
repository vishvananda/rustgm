fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{x:u8,y:u8}static mut X:S=S{x:3,y:5};fn main(){unsafe{X.x=9;check(X.x==9);check(X.y==5);}}