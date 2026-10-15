fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'unused,'a>S<'a>{fn get(self)->&'a u8{self.0}}fn main(){let x=7;check(*S::get(S(&x))==7);}