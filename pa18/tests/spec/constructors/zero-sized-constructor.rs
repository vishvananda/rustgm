fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:usize>();fn main(){let f:fn()->S<3>=S::<3>;let g:fn()->S<5>=S::<5>;let a=f();let b=g();check(true);}
