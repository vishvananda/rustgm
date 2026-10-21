fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn mix<F:FnOnce()->u8,G:FnOnce()->u8>(f:F,g:G)->u8{let first=f();let second=g();if first==7{second}else{0}}fn main(){let n=7u8;let m=9u8;let left=||n;let right=||m;check(mix(left,right)==9);}
