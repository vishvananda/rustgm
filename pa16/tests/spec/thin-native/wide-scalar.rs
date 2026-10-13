fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&u128)->u128{*x}fn main(){let x=340282366920938463463374607431768211455u128;check(f(&x)==x);let b=true;check(*&b);let c='🦀';check(*&c==c);}
