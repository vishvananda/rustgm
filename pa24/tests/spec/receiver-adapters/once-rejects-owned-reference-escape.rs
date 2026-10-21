fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D(u8);fn call<F:FnOnce()->&'static u8>(f:F)->&'static u8{f()}fn main(){let d=D(7);let f=move||&d.0;check(*call(f)==7);}
