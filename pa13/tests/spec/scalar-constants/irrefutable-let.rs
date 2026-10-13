fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const YES:bool=true;const NO:bool=false;fn main(){let (YES|NO)=true;check(true);}