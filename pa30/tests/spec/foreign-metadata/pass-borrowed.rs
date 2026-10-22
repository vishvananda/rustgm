fn main(){let n=42u8;let f:for<'a>extern "C" fn(&'a u8)->&'a u8=one::identity;check(*f(&n)==42);let mut m=41u8;check(*one::update(&mut m)==42);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

